# Event Storage Solutions: Comprehensive Comparison

## Executive Summary

| Solution                       | Best For                    | Complexity | Event Sourcing | Rust Support    | Cost      | Recommendation          |
|--------------------------------|-----------------------------|------------|----------------|-----------------|-----------|-------------------------|
| **PostgreSQL + Domain Events** | Starting out, simple apps   | ⭐ Low      | Partial        | ⭐⭐⭐⭐⭐ Native    | $ Low     | ✅ **START HERE**        |
| **EventStoreDB**               | True event sourcing         | ⭐⭐ Medium  | ⭐⭐⭐⭐⭐ Full     | ⭐⭐⭐⭐ Good       | $$ Medium | Upgrade later           |
| **NATS JetStream**             | Lightweight event streaming | ⭐⭐ Medium  | ⭐⭐⭐ Good       | ⭐⭐⭐⭐⭐ Excellent | $ Low     | Good alternative        |
| **Apache Kafka**               | High-scale, distributed     | ⭐⭐⭐⭐⭐ High | ⭐⭐ Limited     | ⭐⭐⭐ OK          | $$$ High  | Overkill for single app |
| **Redis Streams**              | Fast, simple events         | ⭐⭐ Medium  | ⭐⭐ Limited     | ⭐⭐⭐⭐ Good       | $ Low     | Not recommended for ES  |

---

## Detailed Comparison

### 1. PostgreSQL + Domain Events

#### Overview

Store both current state AND domain events in PostgreSQL. Events are primarily for audit trails and
inter-aggregate communication, not as the source of truth.

#### Architecture

```
Command → Aggregate → State + Events
                         ↓
                    [Transaction]
                         ↓
           ┌─────────────┴─────────────┐
           ↓                           ↓
    State Tables                  Events Table
    (albums, media)              (domain_events)
           ↓                           ↓
    Read Models ←────────────── Event Bus (in-memory)
                                        ↓
                                  Event Handlers
```

#### Implementation Details

**Schema Design**:

```sql
-- State table (source of truth for reads)
CREATE TABLE albums
(
    id          UUID PRIMARY KEY,
    user_id     UUID      NOT NULL,
    title       TEXT      NOT NULL,
    description TEXT,
    version     BIGINT    NOT NULL DEFAULT 0, -- Optimistic locking
    created_at  TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Events table (audit trail + event bus source)
CREATE TABLE domain_events
(
    id             BIGSERIAL PRIMARY KEY,
    aggregate_id   UUID      NOT NULL,
    aggregate_type TEXT      NOT NULL,
    event_type     TEXT      NOT NULL,
    event_data     JSONB     NOT NULL,
    metadata       JSONB,
    version        BIGINT    NOT NULL, -- Per-aggregate version
    created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMP,          -- NULL = not published yet
    UNIQUE (aggregate_id, version)
);

CREATE INDEX idx_events_aggregate ON domain_events (aggregate_id, version);
CREATE INDEX idx_events_unpublished ON domain_events (published_at) WHERE published_at IS NULL;
CREATE INDEX idx_events_type ON domain_events (event_type, created_at);

-- Read model (CQRS)
CREATE TABLE album_list_view
(
    id            UUID PRIMARY KEY,
    user_id       UUID      NOT NULL,
    title         TEXT      NOT NULL,
    thumbnail_url TEXT,
    media_count   INT DEFAULT 0,
    last_updated  TIMESTAMP NOT NULL,
    INDEX         idx_user_albums(user_id, last_updated DESC)
);
```

**Rust Implementation**:

```rust
pub struct PostgresAlbumRepository {
    pool: PgPool,
    event_bus: Arc<dyn EventBus>,
}

impl AlbumRepository for PostgresAlbumRepository {
    async fn save(&self, album: &mut Album) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        let events = album.take_uncommitted_events();

        // 1. Save state (traditional)
        sqlx::query(
            "INSERT INTO albums (id, user_id, title, description, version, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())
             ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                version = EXCLUDED.version,
                updated_at = NOW()
             WHERE albums.version = $5 - 1" - -Optimistic locking
        )
            .bind(&album.id)
            .bind(&album.user_id)
            .bind(&album.title)
            .bind(&album.description)
            .bind(album.version as i64)
            .bind(&album.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                if e.to_string().contains("no rows") {
                    Error::ConcurrencyConflict
                } else {
                    Error::Database(e)
                }
            })?;

        // 2. Save events (audit trail)
        for (i, event) in events.iter().enumerate() {
            sqlx::query(
                "INSERT INTO domain_events
                 (aggregate_id, aggregate_type, event_type, event_data, metadata, version, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW())"
            )
                .bind(&album.id)
                .bind("Album")
                .bind(event.event_type())
                .bind(serde_json::to_value(event)?)
                .bind(serde_json::json!({
                "user_id": album.user_id,
                "correlation_id": uuid::Uuid::new_v4()
            }))
                .bind((album.version - events.len() as u64 + i as u64) as i64)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        // 3. Publish to event bus (after commit)
        for event in events {
            self.event_bus.publish(event).await?;
        }

        Ok(())
    }

    async fn find_by_id(&self, id: &AlbumId) -> Result<Option<Album>> {
        // Load from state table (fast)
        let row = sqlx::query_as::<_, AlbumRow>(
            "SELECT * FROM albums WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.into_domain()))
    }

    // Optional: Reconstruct from events (for debugging/audit)
    async fn reconstruct_from_events(&self, id: &AlbumId) -> Result<Option<Album>> {
        let events: Vec<DomainEvent> = sqlx::query_as(
            "SELECT event_data FROM domain_events
             WHERE aggregate_id = $1 AND aggregate_type = 'Album'
             ORDER BY version ASC"
        )
            .bind(id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row: EventRow| serde_json::from_value(row.event_data))
            .collect::<Result<Vec<_>, _>>()?;

        if events.is_empty() {
            return Ok(None);
        }

        Ok(Some(Album::from_events(events)?))
    }
}
```

#### Pros & Cons

**✅ Advantages**:

- **Simple**: One database, familiar SQL patterns
- **ACID Transactions**: State + events saved atomically
- **Fast Reads**: Direct table queries, no event replay
- **Mature Tooling**: pgAdmin, SQL clients, migrations
- **Cost Effective**: No additional infrastructure
- **Rust Support**: sqlx is excellent
- **Gradual Adoption**: Start simple, add event sourcing later
- **Backup/Restore**: Standard PostgreSQL tools
- **Audit Trail**: Events table provides history

**❌ Disadvantages**:

- **Not Pure Event Sourcing**: State is source of truth, not events
- **Storage Overhead**: Duplicate data (state + events)
- **Event Replay**: Not optimized for reconstructing from events
- **Scalability**: PostgreSQL write limits (~10k writes/sec)
- **No Event Streaming**: Need additional layer for real-time subscriptions
- **Event Ordering**: Across aggregates requires careful timestamp handling

#### When to Use

- ✅ Starting a new project
- ✅ Single application deployments
- ✅ Need audit trail but not full event sourcing
- ✅ Team familiar with SQL
- ✅ Budget constraints
- ✅ Want to iterate quickly

#### Performance Characteristics

- **Writes**: ~5-10k events/sec (single instance)
- **Reads**: Fast (indexed state tables)
- **Event Replay**: Slow (not optimized for this)
- **Storage**: 2x (state + events)

#### Cost Analysis

- **Infrastructure**: $0 (uses existing PostgreSQL)
- **Development**: Low (familiar patterns)
- **Operations**: Low (standard PostgreSQL ops)
- **Total**: $ (Cheapest option)

---

### 2. EventStoreDB

#### Overview

Purpose-built database specifically designed for event sourcing. Events are the source of truth,
state is derived.

#### Architecture

```
Command → Aggregate → Events
                        ↓
                  [EventStoreDB]
                        ↓
            ┌───────────┴───────────┐
            ↓                       ↓
    Event Streams              Subscriptions
    (append-only)                   ↓
            ↓                  Event Handlers
    Projections ←───────────────────┘
    (PostgreSQL)
```

#### Implementation Details

**Setup**:

```yaml
# docker-compose.yml
services:
  eventstore:
    image: eventstore/eventstore:23.10.0-bookworm-slim
    environment:
      - EVENTSTORE_CLUSTER_SIZE=1
      - EVENTSTORE_RUN_PROJECTIONS=All
      - EVENTSTORE_START_STANDARD_PROJECTIONS=true
      - EVENTSTORE_INSECURE=true  # Use HTTPS in production
      - EVENTSTORE_ENABLE_ATOM_PUB_OVER_HTTP=true
    ports:
      - "2113:2113"  # HTTP
      - "1113:1113"  # TCP
    volumes:
      - eventstore-data:/var/lib/eventstore
      - eventstore-logs:/var/log/eventstore
    networks:
      - infrastructure
    healthcheck:
      test: [ "CMD-SHELL", "curl -f http://localhost:2113/health/live || exit 1" ]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  eventstore-data:
  eventstore-logs:
```

**Rust Implementation**:

```rust
use eventstore::{Client, EventData, ReadStreamOptions, StreamPosition, ExpectedRevision};

pub struct EventStoreDbAdapter {
    client: Client,
    projections: Arc<ProjectionManager>,
}

impl EventStoreDbAdapter {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let settings = connection_string.parse()?;
        let client = Client::new(settings)?;

        Ok(Self {
            client,
            projections: Arc::new(ProjectionManager::new()),
        })
    }
}

impl EventStore for EventStoreDbAdapter {
    async fn append_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        events: Vec<DomainEvent>,
        expected_version: ExpectedVersion,
    ) -> Result<()> {
        let stream_name = format!("{}-{}", aggregate_type, aggregate_id);

        let event_data: Vec<EventData> = events
            .into_iter()
            .map(|event| {
                EventData::json(
                    event.event_type(),
                    event
                )
                    .expect("Failed to serialize event")
                    .metadata_as_json(serde_json::json!({
                    "aggregate_id": aggregate_id,
                    "aggregate_type": aggregate_type,
                    "timestamp": chrono::Utc::now(),
                }))
            })
            .collect();

        let expected_revision = match expected_version {
            ExpectedVersion::NoStream => ExpectedRevision::NoStream,
            ExpectedVersion::Exact(v) => ExpectedRevision::Exact(v),
            ExpectedVersion::Any => ExpectedRevision::Any,
        };

        self.client
            .append_to_stream(stream_name, &expected_revision, event_data)
            .await?;

        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> Result<Vec<DomainEvent>> {
        let stream_name = format!("{}-{}", aggregate_type, aggregate_id);

        let mut stream = self.client
            .read_stream(
                stream_name,
                &ReadStreamOptions::default()
                    .position(StreamPosition::Start)
            )
            .await?;

        let mut events = Vec::new();

        while let Some(event) = stream.next().await? {
            let resolved = event.get_original_event();
            let event_data: DomainEvent = resolved.as_json()?;
            events.push(event_data);
        }

        Ok(events)
    }

    async fn subscribe_to_all<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(DomainEvent) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
    {
        let mut subscription = self.client
            .subscribe_to_all()
            .await?;

        while let Some(event) = subscription.next().await? {
            let resolved = event.event.expect("No event");

            if resolved.event_type.starts_with("$") {
                continue; // Skip system events
            }

            let domain_event: DomainEvent = resolved.as_json()?;
            handler(domain_event).await?;
        }

        Ok(())
    }
}

// Aggregate Repository
pub struct EventSourcedAlbumRepository {
    event_store: Arc<dyn EventStore>,
}

impl AlbumRepository for EventSourcedAlbumRepository {
    async fn save(&self, album: &mut Album) -> Result<()> {
        let events = album.take_uncommitted_events();
        let expected_version = if album.version == events.len() as u64 {
            ExpectedVersion::NoStream
        } else {
            ExpectedVersion::Exact(album.version - events.len() as u64)
        };

        self.event_store
            .append_events(
                &album.id.to_string(),
                "Album",
                events,
                expected_version,
            )
            .await?;

        Ok(())
    }

    async fn find_by_id(&self, id: &AlbumId) -> Result<Option<Album>> {
        let events = self.event_store
            .load_events(&id.to_string(), "Album")
            .await?;

        if events.is_empty() {
            return Ok(None);
        }

        Ok(Some(Album::from_events(events)?))
    }
}

// Projection Handler (runs asynchronously)
pub struct AlbumProjectionHandler {
    pool: PgPool,
}

impl AlbumProjectionHandler {
    pub async fn handle(&self, event: DomainEvent) -> Result<()> {
        match event {
            DomainEvent::AlbumCreated(e) => {
                sqlx::query(
                    "INSERT INTO album_list_view
                     (id, user_id, title, thumbnail_url, media_count, last_updated)
                     VALUES ($1, $2, $3, NULL, 0, NOW())"
                )
                    .bind(&e.album_id)
                    .bind(&e.user_id)
                    .bind(&e.title)
                    .execute(&self.pool)
                    .await?;
            }
            DomainEvent::MediumAddedToAlbum(e) => {
                sqlx::query(
                    "UPDATE album_list_view
                     SET media_count = media_count + 1,
                         last_updated = NOW()
                     WHERE id = $1"
                )
                    .bind(&e.album_id)
                    .execute(&self.pool)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}
```

#### Pros & Cons

**✅ Advantages**:

- **True Event Sourcing**: Events are source of truth
- **Time Travel**: Query state at any point in history
- **Optimized for ES**: Append-only, optimistic concurrency built-in
- **Event Streaming**: Built-in subscriptions (catch-up, persistent)
- **Projections**: Native projection system
- **Event Versioning**: Handle event schema evolution
- **Audit Trail**: Complete, immutable history
- **Scalability**: Handles 15k+ writes/sec
- **Idempotency**: Built-in deduplication
- **Tooling**: Web UI, event browser, stream viewer

**❌ Disadvantages**:

- **Additional Infrastructure**: Another database to manage
- **Learning Curve**: Event sourcing concepts
- **Event Replay Cost**: CPU intensive for large streams
- **Storage Growth**: Events accumulate (snapshots help)
- **Complexity**: More moving parts
- **Eventual Consistency**: Projections lag behind events
- **Schema Evolution**: Requires upcasting strategies
- **Operational Overhead**: Monitoring, backups, clustering

#### When to Use

- ✅ Need complete audit trail with time travel
- ✅ Complex domain with many state transitions
- ✅ Regulatory compliance (financial, healthcare)
- ✅ Debugging requires event replay
- ✅ High write throughput (>5k/sec)
- ✅ Team experienced with event sourcing
- ✅ Want to analyze event patterns

#### Performance Characteristics

- **Writes**: ~15k events/sec (single node), ~100k+ (cluster)
- **Reads**: Fast subscriptions, moderate replay (depends on stream size)
- **Event Replay**: Optimized with snapshots
- **Storage**: Efficient (append-only, compressed)

#### Cost Analysis

- **Infrastructure**: $$ (Docker: ~$50-100/mo, Cloud: $200-500/mo)
- **Development**: Medium (learning curve)
- **Operations**: Medium (monitoring, backups, upgrades)
- **Total**: $$ (Medium cost)

---

### 3. NATS JetStream

#### Overview

Lightweight, cloud-native messaging system with persistence. Good middle ground between simplicity
and capability.

#### Architecture

```
Command → Aggregate → Events
                        ↓
                  [NATS JetStream]
                        ↓
            ┌───────────┴───────────┐
            ↓                       ↓
        Streams                 Consumers
    (per aggregate type)             ↓
            ↓                  Event Handlers
    Key-Value Store                  ↓
    (snapshots)              Projections (PostgreSQL)
```

#### Implementation Details

**Setup**:

```yaml
# docker-compose.yml
services:
  nats:
    image: nats:2.10-alpine
    command:
      - "-js"  # Enable JetStream
      - "-sd=/data"  # Storage directory
      - "-m=8222"  # Monitoring port
    ports:
      - "4222:4222"  # Client connections
      - "8222:8222"  # HTTP monitoring
    volumes:
      - nats-data:/data
    networks:
      - infrastructure
    healthcheck:
      test: [ "CMD", "wget", "--spider", "-q", "http://localhost:8222/healthz" ]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  nats-data:
```

**Rust Implementation**:

```rust
use async_nats::{Client, jetstream};
use futures::StreamExt;

pub struct NatsEventStore {
    client: Client,
    jetstream: jetstream::Context,
}

impl NatsEventStore {
    pub async fn new(url: &str) -> Result<Self> {
        let client = async_nats::connect(url).await?;
        let jetstream = jetstream::new(client.clone());

        // Create stream for album events
        jetstream
            .create_stream(jetstream::stream::Config {
                name: "ALBUM_EVENTS".to_string(),
                subjects: vec!["album.>".to_string()],
                retention: jetstream::stream::RetentionPolicy::Limits,
                max_age: std::time::Duration::from_secs(365 * 24 * 60 * 60), // 1 year
                storage: jetstream::stream::StorageType::File,
                ..Default::default()
            })
            .await?;

        Ok(Self { client, jetstream })
    }
}

impl EventStore for NatsEventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        events: Vec<DomainEvent>,
        expected_version: ExpectedVersion,
    ) -> Result<()> {
        for (i, event) in events.iter().enumerate() {
            let subject = format!(
                "{}.{}.{}",
                aggregate_type.to_lowercase(),
                aggregate_id,
                event.event_type()
            );

            let payload = serde_json::to_vec(&event)?;

            let headers = async_nats::HeaderMap::from_iter([
                ("Nats-Msg-Id".to_string(), format!("{}-{}", aggregate_id, i)), // Idempotency
                ("aggregate_id".to_string(), aggregate_id.to_string()),
                ("aggregate_type".to_string(), aggregate_type.to_string()),
                ("event_type".to_string(), event.event_type()),
                ("timestamp".to_string(), chrono::Utc::now().to_rfc3339()),
            ]);

            let message = async_nats::jetstream::context::Publish::build()
                .payload(payload.into())
                .headers(headers);

            self.jetstream
                .send_publish(subject, message)
                .await?
                .await?;
        }

        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> Result<Vec<DomainEvent>> {
        let subject = format!("{}.{}.>", aggregate_type.to_lowercase(), aggregate_id);

        let stream = self.jetstream
            .get_stream("ALBUM_EVENTS")
            .await?;

        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                filter_subject: subject,
                deliver_policy: jetstream::consumer::DeliverPolicy::All,
                ack_policy: jetstream::consumer::AckPolicy::None,
                ..Default::default()
            })
            .await?;

        let mut events = Vec::new();
        let mut messages = consumer.messages().await?;

        while let Some(Ok(message)) = messages.next().await {
            let event: DomainEvent = serde_json::from_slice(&message.payload)?;
            events.push(event);
        }

        Ok(events)
    }

    // Subscribe to all events
    pub async fn subscribe<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(DomainEvent) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
    {
        let stream = self.jetstream.get_stream("ALBUM_EVENTS").await?;

        let consumer = stream
            .create_consumer(jetstream::consumer::pull::Config {
                durable_name: Some("projection-consumer".to_string()),
                deliver_policy: jetstream::consumer::DeliverPolicy::All,
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                ack_wait: std::time::Duration::from_secs(30),
                ..Default::default()
            })
            .await?;

        let mut messages = consumer.messages().await?;

        while let Some(Ok(message)) = messages.next().await {
            let event: DomainEvent = serde_json::from_slice(&message.payload)?;

            if let Err(e) = handler(event).await {
                eprintln!("Error handling event: {}", e);
                // Don't ack, will retry
            } else {
                message.ack().await?;
            }
        }

        Ok(())
    }
}

// Snapshot support using NATS KV
pub struct NatsSnapshotStore {
    kv: jetstream::kv::Store,
}

impl NatsSnapshotStore {
    pub async fn new(jetstream: &jetstream::Context) -> Result<Self> {
        let kv = jetstream
            .create_key_value(jetstream::kv::Config {
                bucket: "SNAPSHOTS".to_string(),
                history: 5,
                ..Default::default()
            })
            .await?;

        Ok(Self { kv })
    }

    pub async fn save_snapshot(&self, aggregate_id: &str, snapshot: &Album) -> Result<()> {
        let key = format!("album-{}", aggregate_id);
        let value = serde_json::to_vec(snapshot)?;

        self.kv.put(&key, value.into()).await?;

        Ok(())
    }

    pub async fn load_snapshot(&self, aggregate_id: &str) -> Result<Option<Album>> {
        let key = format!("album-{}", aggregate_id);

        match self.kv.get(&key).await {
            Ok(Some(entry)) => {
                let snapshot = serde_json::from_slice(&entry)?;
                Ok(Some(snapshot))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
```

#### Pros & Cons

**✅ Advantages**:

- **Lightweight**: Small footprint (~20MB binary)
- **Simple Operations**: Easy to deploy and manage
- **Performance**: ~1M+ msgs/sec
- **Excellent Rust Support**: async-nats is mature
- **Built-in Features**: Key-value store, object store
- **Clustering**: Easy to set up for HA
- **Cost Effective**: Low resource usage
- **Cloud Native**: CNCF project
- **Monitoring**: Built-in HTTP monitoring
- **At-Least-Once Delivery**: Consumer ack tracking

**❌ Disadvantages**:

- **Not Purpose-Built for ES**: No native event sourcing features
- **Limited Querying**: No complex event queries
- **No Time Travel**: Can't easily query past state
- **Manual Snapshots**: Need to implement yourself
- **Event Ordering**: Only guaranteed per subject
- **Less Mature ES Patterns**: Need to build patterns yourself
- **Retention Policies**: Need manual configuration
- **No Native Projections**: Must build projection infrastructure

#### When to Use

- ✅ Want lightweight messaging + events
- ✅ Need fast pub/sub with persistence
- ✅ Prefer simplicity over advanced ES features
- ✅ Building microservices (future)
- ✅ Want easy horizontal scaling
- ✅ Team familiar with messaging systems

#### Performance Characteristics

- **Writes**: ~100k-1M+ messages/sec
- **Reads**: Fast subscriptions
- **Event Replay**: Good with snapshots
- **Storage**: Efficient (compressed)

#### Cost Analysis

- **Infrastructure**: $ (Docker: ~$20-40/mo, Cloud: $50-150/mo)
- **Development**: Medium (need to build ES patterns)
- **Operations**: Low (simple to manage)
- **Total**: $ (Low to medium cost)

---

### 4. Apache Kafka

#### Overview

Distributed event streaming platform. Massive scale, high complexity. Overkill for single
application.

#### Architecture

```
Command → Aggregate → Events
                        ↓
                  [Kafka Topics]
                        ↓
            ┌───────────┴───────────────┐
            ↓                           ↓
     Partitions                    Consumers
    (ordered per key)              (Consumer Groups)
            ↓                           ↓
    Compacted Topics             Event Handlers
    (snapshots)                        ↓
                                Projections (PostgreSQL)
```

#### Implementation Details

**Setup**:

```yaml
# docker-compose.yml (minimal setup)
services:
  zookeeper:
    image: confluentinc/cp-zookeeper:7.5.0
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
    ports:
      - "2181:2181"
    volumes:
      - zookeeper-data:/var/lib/zookeeper/data
      - zookeeper-log:/var/lib/zookeeper/log

  kafka:
    image: confluentinc/cp-kafka:7.5.0
    depends_on:
      - zookeeper
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
      KAFKA_LOG_RETENTION_MS: 31536000000  # 1 year
    ports:
      - "9092:9092"
    volumes:
      - kafka-data:/var/lib/kafka/data

  schema-registry:
    image: confluentinc/cp-schema-registry:7.5.0
    depends_on:
      - kafka
    environment:
      SCHEMA_REGISTRY_HOST_NAME: schema-registry
      SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS: kafka:9092
    ports:
      - "8081:8081"

volumes:
  zookeeper-data:
  zookeeper-log:
  kafka-data:
```

**Rust Implementation**:

```rust
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    consumer::{Consumer, StreamConsumer},
    config::ClientConfig,
    message::Message,
};

pub struct KafkaEventStore {
    producer: FutureProducer,
    consumer: StreamConsumer,
}

impl KafkaEventStore {
    pub async fn new(brokers: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")  // Strong consistency
            .set("enable.idempotence", "true")  // Exactly-once semantics
            .create()?;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", "infrastructure-projections")
            .set("enable.auto.commit", "false")  // Manual commit
            .set("auto.offset.reset", "earliest")
            .create()?;

        Ok(Self { producer, consumer })
    }
}

impl EventStore for KafkaEventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        events: Vec<DomainEvent>,
        expected_version: ExpectedVersion,
    ) -> Result<()> {
        let topic = format!("{}-events", aggregate_type.to_lowercase());

        // Note: Kafka doesn't have native optimistic concurrency
        // Would need to implement with external version tracking

        for event in events {
            let key = aggregate_id;  // Ensures ordering per aggregate
            let payload = serde_json::to_string(&event)?;

            let record = FutureRecord::to(&topic)
                .key(key)
                .payload(&payload)
                .headers(rdkafka::message::OwnedHeaders::new()
                    .insert(rdkafka::message::Header {
                        key: "event_type",
                        value: Some(event.event_type()),
                    })
                    .insert(rdkafka::message::Header {
                        key: "aggregate_type",
                        value: Some(aggregate_type),
                    }));

            self.producer
                .send(record, std::time::Duration::from_secs(5))
                .await
                .map_err(|(e, _)| Error::Kafka(e))?;
        }

        Ok(())
    }

    // Note: Loading events from Kafka is not efficient
    // Would need to use compacted topics + snapshots
    async fn load_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> Result<Vec<DomainEvent>> {
        // This is inefficient - Kafka is designed for streaming, not querying
        // In practice, you'd use snapshots + recent events

        unimplemented!("Use snapshots + recent events instead");
    }

    pub async fn subscribe<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(DomainEvent) -> BoxFuture<'static, Result<()>> + Send + Sync + 'static,
    {
        self.consumer.subscribe(&["album-events", "medium-events"])?;

        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    let payload = message.payload().expect("Message has no payload");
                    let event: DomainEvent = serde_json::from_slice(payload)?;

                    if let Err(e) = handler(event).await {
                        eprintln!("Error handling event: {}", e);
                        // Don't commit, will retry
                    } else {
                        self.consumer.commit_message(&message, rdkafka::consumer::CommitMode::Async)?;
                    }
                }
                Err(e) => {
                    eprintln!("Kafka error: {}", e);
                }
            }
        }
    }
}
```

#### Pros & Cons

**✅ Advantages**:

- **Massive Scale**: Millions of events/sec
- **Battle-Tested**: Used by LinkedIn, Netflix, Uber
- **Distributed**: Built for horizontal scaling
- **Durability**: Replicated, fault-tolerant
- **Retention**: Configurable, can keep forever
- **Ecosystem**: Rich tooling (Kafka Streams, Connect, ksqlDB)
- **Exactly-Once Semantics**: With idempotent producers
- **Compaction**: Log compaction for snapshots

**❌ Disadvantages**:

- **Extreme Complexity**: Requires Zookeeper/KRaft, schema registry
- **Resource Heavy**: High memory/CPU usage
- **Operational Burden**: Complex to tune and maintain
- **Overkill**: Way too much for single app
- **Learning Curve**: Steep
- **Cost**: Expensive infrastructure
- **Not Built for ES**: Need to build ES patterns on top
- **Query Limitations**: Not designed for random access

#### When to Use

- ✅ Building distributed microservices platform
- ✅ Need millions of events/sec
- ✅ Have dedicated platform team
- ✅ Multiple applications consuming events
- ✅ Real-time analytics requirements
- ❌ **Single application** (use something simpler)

#### Performance Characteristics

- **Writes**: ~1M+ events/sec (cluster)
- **Reads**: Fast streaming, poor random access
- **Event Replay**: Good with consumer groups
- **Storage**: Efficient with compaction

#### Cost Analysis

- **Infrastructure**: $$$ (Cloud: $500-2000+/mo)
- **Development**: High (complexity)
- **Operations**: High (requires expertise)
- **Total**: $$$ (Very expensive)

---

### 5. Redis Streams

#### Overview

Redis data structure for log-style data. Fast, simple, but limited persistence guarantees.

#### Architecture

```
Command → Aggregate → Events
                        ↓
                 [Redis Streams]
                        ↓
            ┌───────────┴───────────┐
            ↓                       ↓
     Stream per Aggregate      Consumer Groups
            ↓                       ↓
    Trimming Policy          Event Handlers
    (max length)                   ↓
                            Projections (PostgreSQL)
```

#### Implementation Details

**Rust Implementation**:

```rust
use redis::{Client, AsyncCommands, streams::{StreamReadOptions, StreamReadReply}};

pub struct RedisEventStore {
    client: Client,
}

impl RedisEventStore {
    pub async fn new(url: &str) -> Result<Self> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }
}

impl EventStore for RedisEventStore {
    async fn append_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        events: Vec<DomainEvent>,
        expected_version: ExpectedVersion,
    ) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let stream_key = format!("events:{}:{}", aggregate_type, aggregate_id);

        for event in events {
            let event_data = serde_json::to_string(&event)?;

            conn.xadd(
                &stream_key,
                "*",  // Auto-generate ID
                &[
                    ("event_type", event.event_type()),
                    ("event_data", event_data.as_str()),
                    ("timestamp", &chrono::Utc::now().to_rfc3339()),
                ]
            ).await?;
        }

        // Optional: Trim stream to max length
        conn.xtrim(&stream_key, redis::streams::StreamMaxlen::Approx(10000)).await?;

        Ok(())
    }

    async fn load_events(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
    ) -> Result<Vec<DomainEvent>> {
        let mut conn = self.client.get_async_connection().await?;
        let stream_key = format!("events:{}:{}", aggregate_type, aggregate_id);

        let reply: StreamReadReply = conn.xread(&[&stream_key], &["0"]).await?;

        let mut events = Vec::new();
        for stream_key in reply.keys {
            for stream_id in stream_key.ids {
                let event_data: String = stream_id.get("event_data").unwrap();
                let event: DomainEvent = serde_json::from_str(&event_data)?;
                events.push(event);
            }
        }

        Ok(events)
    }
}
```

#### Pros & Cons

**✅ Advantages**:

- **Very Fast**: In-memory performance
- **Simple**: Easy to understand and use
- **Lightweight**: No complex setup
- **Good Rust Support**: redis-rs is mature
- **Consumer Groups**: Built-in
- **Real-time**: Low latency

**❌ Disadvantages**:

- **Persistence**: Optional, not guaranteed (AOF/RDB)
- **Limited Storage**: Memory-bound
- **Data Loss Risk**: If Redis crashes
- **Not Built for ES**: Streams have size limits
- **No Transactions**: Across multiple streams
- **Trimming**: Must manually manage stream size
- **Cost**: Memory is expensive at scale

#### When to Use

- ⚠️ Not recommended for event sourcing source of truth
- ✅ Good for: Real-time event bus, cache layer
- ✅ Combined with persistent event store

#### Performance Characteristics

- **Writes**: ~100k+ ops/sec
- **Reads**: Very fast
- **Event Replay**: Limited by memory
- **Storage**: Memory-bound

#### Cost Analysis

- **Infrastructure**: $$ (Memory expensive)
- **Development**: Low
- **Operations**: Low
- **Total**: $$ (Medium cost due to memory)

---

## Decision Matrix

### For Photonic (Your Photo App)

| Requirement                | PostgreSQL + Domain Events | EventStoreDB        | NATS JetStream      | Kafka               | Redis            |
|----------------------------|----------------------------|---------------------|---------------------|---------------------|------------------|
| **Audit Trail**            | ✅ Good                     | ✅ Excellent         | ✅ Good              | ✅ Excellent         | ⚠️ Limited       |
| **Time Travel**            | ⚠️ Manual                  | ✅ Native            | ⚠️ Manual           | ⚠️ Manual           | ❌ No             |
| **Simplicity**             | ✅ Very Simple              | ⚠️ Medium           | ✅ Simple            | ❌ Complex           | ✅ Simple         |
| **ACID Transactions**      | ✅ Yes                      | ⚠️ Per stream       | ⚠️ Per subject      | ❌ No                | ⚠️ Limited       |
| **Query Performance**      | ✅ Excellent                | ⚠️ Need projections | ⚠️ Need projections | ⚠️ Need projections | ✅ Fast           |
| **Single App Fit**         | ✅ Perfect                  | ✅ Good              | ✅ Good              | ❌ Overkill          | ⚠️ As cache only |
| **Operational Complexity** | ✅ Low                      | ⚠️ Medium           | ✅ Low               | ❌ High              | ✅ Low            |
| **Cost**                   | ✅ $                        | ⚠️ $$               | ✅ $                 | ❌ $$$               | ⚠️ $$            |
| **Rust Ecosystem**         | ✅ Excellent                | ✅ Good              | ✅ Excellent         | ⚠️ OK               | ✅ Good           |
| **Photo Processing**       | ✅ Great                    | ✅ Good              | ✅ Good              | ⚠️ Overkill         | ⚠️ Limited       |

---

## Recommendation for Photonic

### Phase 1: Start (Current State) - **PostgreSQL + Domain Events**

**Why:**

- You already have PostgreSQL
- Simple to implement
- Fast iteration
- Low operational overhead
- Team can stay productive
- Easy to test and debug

**What you get:**

- Domain events for integration
- Audit trail in events table
- CQRS read models
- Event-driven architecture
- All DDD benefits

**Implementation:**

```rust
// Your current architecture already supports this!
// Just add the events table and event bus

// src/infrastructure/persistence/postgres/event_repository.rs
pub struct PostgresEventRepository {
    pool: PgPool,
}

impl EventRepository for PostgresEventRepository {
    async fn save_events(&self, events: &[DomainEvent]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for event in events {
            sqlx::query(
                "INSERT INTO domain_events
                 (aggregate_id, aggregate_type, event_type, event_data, version, created_at)
                 VALUES ($1, $2, $3, $4, $5, NOW())"
            )
                .bind(event.aggregate_id())
                .bind(event.aggregate_type())
                .bind(event.event_type())
                .bind(serde_json::to_value(event)?)
                .bind(event.version() as i64)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
```

---

### Phase 2: Scale (Future, if needed) - **Add EventStoreDB**

**When to upgrade:**

- Need time-travel queries
- Regulatory compliance requires immutable audit log
- Want to replay events for debugging
- Need to create new projections from history
- Write volume exceeds PostgreSQL capacity (>10k writes/sec)

**Migration path:**

1. Deploy EventStoreDB alongside PostgreSQL
2. Start writing events to both (dual-write pattern)
3. Migrate projections to read from EventStoreDB
4. Deprecate PostgreSQL events table
5. Keep PostgreSQL for read models

---

### Phase 3: Distributed (Far future) - **NATS JetStream or Kafka**

**When to upgrade:**

- Breaking into microservices
- Need cross-service event streaming
- Multiple teams/services consuming events
- Need geographic distribution

---

## Sample Implementation Timeline

### Week 1-2: PostgreSQL + Domain Events ✅

```
✅ Add domain_events table
✅ Implement EventRepository
✅ Integrate with existing repositories
✅ Add in-memory event bus
✅ Create first projection (album list view)
✅ Test with existing use cases
```

### Week 3-4: CQRS Read Models

```
✅ Identify read-heavy queries
✅ Create optimized read models
✅ Add projection handlers
✅ Performance testing
```

### Month 2-3: Monitoring & Refinement

```
✅ Add event monitoring
✅ Track projection lag
✅ Optimize slow projections
✅ Add event versioning strategy
```

### Future (only if needed): EventStoreDB

```
⏳ Evaluate need
⏳ Deploy EventStoreDB
⏳ Dual-write pattern
⏳ Migrate projections
⏳ Full cutover
```

---

## Code Examples for PostgreSQL Approach

### Migration

```sql
-- migrations/add_domain_events.sql
CREATE TABLE domain_events
(
    id             BIGSERIAL PRIMARY KEY,
    aggregate_id   UUID      NOT NULL,
    aggregate_type TEXT      NOT NULL,
    event_type     TEXT      NOT NULL,
    event_data     JSONB     NOT NULL,
    metadata       JSONB,
    version        BIGINT    NOT NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT NOW(),
    published_at   TIMESTAMP,
    UNIQUE (aggregate_id, version)
);

CREATE INDEX idx_events_aggregate ON domain_events (aggregate_id, version);
CREATE INDEX idx_events_type ON domain_events (event_type, created_at);
CREATE INDEX idx_events_unpublished ON domain_events (created_at) WHERE published_at IS NULL;

-- Read model
CREATE TABLE album_list_view
(
    id            UUID PRIMARY KEY,
    user_id       UUID      NOT NULL,
    title         TEXT      NOT NULL,
    thumbnail_url TEXT,
    media_count   INT DEFAULT 0,
    created_at    TIMESTAMP NOT NULL,
    last_updated  TIMESTAMP NOT NULL,
    INDEX         idx_user_albums(user_id, last_updated DESC),
    INDEX         idx_search(title)
);
```

### Domain Event Definition

```rust
// src/shared/event.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DomainEvent {
    AlbumCreated(AlbumCreatedEvent),
    AlbumUpdated(AlbumUpdatedEvent),
    MediumAddedToAlbum(MediumAddedToAlbumEvent),
    // ... more events
}

impl DomainEvent {
    pub fn aggregate_id(&self) -> Uuid {
        match self {
            Self::AlbumCreated(e) => e.album_id,
            Self::AlbumUpdated(e) => e.album_id,
            Self::MediumAddedToAlbum(e) => e.album_id,
        }
    }

    pub fn aggregate_type(&self) -> &'static str {
        match self {
            Self::AlbumCreated(_) | Self::AlbumUpdated(_) | Self::MediumAddedToAlbum(_) => "Album",
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Self::AlbumCreated(_) => "AlbumCreated",
            Self::AlbumUpdated(_) => "AlbumUpdated",
            Self::MediumAddedToAlbum(_) => "MediumAddedToAlbum",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumCreatedEvent {
    pub album_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

---

## Final Recommendation

**Start with PostgreSQL + Domain Events**

Then upgrade **only if you actually need it:**

```
Current State → PostgreSQL + Events → (if needed) EventStoreDB → (if needed) Kafka
   (CRUD)            (80% benefits)        (full ES)           (distributed)
```

**Why this is right for Photonic:**

1. ✅ Single application requirement
2. ✅ Photo management isn't ultra-high scale
3. ✅ Team can be productive immediately
4. ✅ Low operational complexity
5. ✅ Easy to test and debug
6. ✅ You keep all your existing PostgreSQL knowledge
7. ✅ Can always upgrade later without rewriting domain logic

The domain layer stays the same regardless of which storage you choose. That's the beauty of
Hexagonal Architecture!