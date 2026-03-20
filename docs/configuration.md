# Photonic Configuration

This document describes all configuration options for Photonic, including environment variables,
configuration files, and deployment examples.

## Table of Contents

- [Overview](#overview)
- [Environment Variables](#environment-variables)
- [Configuration File](#configuration-file-optional)
- [Storage Configuration](#storage-configuration)
- [Path Pattern Configuration](#path-pattern-configuration)
- [Processing Configuration](#processing-configuration)
- [Database Configuration](#database-configuration)
- [Authentication Configuration](#authentication-configuration)
- [Logging and Tracing](#logging-and-tracing)
- [Deployment Examples](#deployment-examples)

---

## Overview

### Configuration Priority

Configuration is loaded in this order (later sources override earlier ones):

1. Default values (hardcoded)
2. Configuration file (`/etc/photonic/config.toml` or `PHOTONIC_CONFIG`)
3. Environment variables
4. Command-line arguments

### Configuration Validation

All configuration is validated on startup. The application will fail to start if:

- Required variables are missing
- Values are invalid (e.g., negative numbers, invalid URLs)
- Storage paths don't exist or aren't writable
- Database connection fails

---

## Environment Variables

### Server Configuration

#### `SERVER_HOST`

- **Description:** IP address to bind to
- **Type:** String
- **Default:** `0.0.0.0` (all interfaces)
- **Example:** `127.0.0.1` (localhost only)

#### `SERVER_PORT`

- **Description:** Port to listen on
- **Type:** Integer
- **Default:** `8080`
- **Range:** 1-65535
- **Example:** `3000`

#### `SERVER_WORKERS`

- **Description:** Number of worker threads
- **Type:** Integer
- **Default:** Number of CPU cores
- **Range:** 1-256
- **Example:** `4`

---

### Authentication Configuration

#### `JWT_ISSUER`

- **Description:** Expected JWT issuer (iss claim)
- **Type:** URL
- **Required:** Yes
- **Example:** `https://idp.example.com`

#### `JWT_AUDIENCE`

- **Description:** Expected JWT audience (aud claim)
- **Type:** String
- **Required:** Yes
- **Example:** `photonic-api`

#### `JWT_JWKS_URL`

- **Description:** URL to fetch JWKS (public keys)
- **Type:** URL
- **Required:** Yes
- **Example:** `https://idp.example.com/.well-known/jwks.json`

#### `JWT_VALIDATE_EXPIRY`

- **Description:** Validate JWT expiration
- **Type:** Boolean
- **Default:** `true`
- **Example:** `false` (dev only)

#### `JWT_LEEWAY_SECONDS`

- **Description:** Clock skew tolerance for exp/nbf
- **Type:** Integer
- **Default:** `60`
- **Range:** 0-300
- **Example:** `120`

---

### Database Configuration

#### `DATABASE_URL`

- **Description:** PostgreSQL connection string
- **Type:** Connection String
- **Required:** Yes
- **Format:** `postgresql://user:password@host:port/database`
- **Example:** `postgresql://photonic:secret@localhost:5432/photonic`

#### `DATABASE_MAX_CONNECTIONS`

- **Description:** Maximum connection pool size
- **Type:** Integer
- **Default:** `10`
- **Range:** 1-100
- **Example:** `20`

#### `DATABASE_MIN_CONNECTIONS`

- **Description:** Minimum connection pool size
- **Type:** Integer
- **Default:** `2`
- **Range:** 0-max_connections
- **Example:** `5`

#### `DATABASE_ACQUIRE_TIMEOUT_SECONDS`

- **Description:** Timeout for acquiring connection
- **Type:** Integer
- **Default:** `30`
- **Range:** 1-300
- **Example:** `60`

#### `DATABASE_IDLE_TIMEOUT_SECONDS`

- **Description:** Idle connection timeout
- **Type:** Integer
- **Default:** `600` (10 minutes)
- **Range:** 60-3600
- **Example:** `300`

#### `DATABASE_MAX_LIFETIME_SECONDS`

- **Description:** Maximum connection lifetime
- **Type:** Integer
- **Default:** `1800` (30 minutes)
- **Range:** 300-7200
- **Example:** `3600`

#### `DATABASE_RUN_MIGRATIONS`

- **Description:** Automatically run migrations on startup
- **Type:** Boolean
- **Default:** `true`
- **Example:** `false` (production - use separate migration job)

---

### Storage Configuration

#### `STORAGE_BASE_PATH`

- **Description:** Base directory for all file storage
- **Type:** Path
- **Required:** Yes
- **Example:** `/var/photonic/storage`

#### `STORAGE_TEMP_PATH`

- **Description:** Temporary storage directory (fast/SSD)
- **Type:** Path
- **Default:** `{STORAGE_BASE_PATH}/temporary`
- **Example:** `/mnt/ssd/photonic/temp`

#### `STORAGE_PERMANENT_PATH`

- **Description:** Permanent storage directory
- **Type:** Path
- **Default:** `{STORAGE_BASE_PATH}/permanent`
- **Example:** `/mnt/hdd/photonic/permanent`

#### `STORAGE_CACHE_PATH`

- **Description:** Cache storage for variants
- **Type:** Path
- **Default:** `{STORAGE_BASE_PATH}/cache`
- **Example:** `/mnt/ssd/photonic/cache`

#### `STORAGE_PROVIDER`

- **Description:** Storage backend to use
- **Type:** Enum
- **Default:** `filesystem`
- **Values:** `filesystem`, `s3` (future)
- **Example:** `filesystem`

#### `STORAGE_ENSURE_DIRECTORIES`

- **Description:** Create storage directories on startup
- **Type:** Boolean
- **Default:** `true`
- **Example:** `false` (if pre-provisioned)

---

### S3 Storage Configuration (Future)

#### `S3_ENDPOINT`

- **Description:** S3-compatible endpoint URL
- **Type:** URL
- **Required:** If STORAGE_PROVIDER=s3
- **Example:** `https://s3.amazonaws.com`

#### `S3_REGION`

- **Description:** AWS region
- **Type:** String
- **Required:** If STORAGE_PROVIDER=s3
- **Example:** `us-east-1`

#### `S3_BUCKET`

- **Description:** S3 bucket name
- **Type:** String
- **Required:** If STORAGE_PROVIDER=s3
- **Example:** `photonic-media`

#### `S3_ACCESS_KEY_ID`

- **Description:** AWS access key ID
- **Type:** String
- **Required:** If STORAGE_PROVIDER=s3
- **Example:** `AKIAIOSFODNN7EXAMPLE`

#### `S3_SECRET_ACCESS_KEY`

- **Description:** AWS secret access key
- **Type:** String (sensitive)
- **Required:** If STORAGE_PROVIDER=s3
- **Example:** `wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY`

---

### Path Pattern Configuration

#### `STORAGE_PATH_PATTERN`

- **Description:** Template for organizing files in permanent storage
- **Type:** String
- **Default:** `{year}/{month}/{camera_make}/{filename}`
- **Variables:**
    - `{year}` - 4-digit year
    - `{month}` - 2-digit month (01-12)
    - `{day}` - 2-digit day (01-31)
    - `{camera_make}` - Camera manufacturer
    - `{camera_model}` - Camera model
    - `{lens_model}` - Lens model
    - `{filename}` - Original filename
    - `{extension}` - File extension
    - `{user_id}` - User UUID

**Example Patterns:**

```bash
# Organize by date and camera
STORAGE_PATH_PATTERN="{year}/{month}/{camera_make}/{filename}"
# Result: 2024/12/Canon/IMG_1234.jpg

# Organize by user and date
STORAGE_PATH_PATTERN="{user_id}/{year}/{month}/{filename}"
# Result: 550e8400-e29b-41d4-a716-446655440000/2024/12/IMG_1234.jpg

# Organize by camera and date
STORAGE_PATH_PATTERN="{camera_make}/{camera_model}/{year}-{month}/{filename}"
# Result: Canon/EOS_R5/2024-12/IMG_1234.jpg

# Flat structure with date prefix
STORAGE_PATH_PATTERN="{year}{month}{day}_{filename}"
# Result: 20241216_IMG_1234.jpg
```

#### `STORAGE_PATH_FALLBACK_DATE`

- **Description:** What to use if taken_at is missing
- **Type:** Enum
- **Default:** `upload_time`
- **Values:** `upload_time`, `file_modified`, `current_time`
- **Example:** `file_modified`

#### `STORAGE_PATH_FALLBACK_CAMERA`

- **Description:** String to use if camera info missing
- **Type:** String
- **Default:** `Unknown`
- **Example:** `NoCamera`

#### `STORAGE_PATH_SANITIZE`

- **Description:** Sanitize path components for filesystem safety
- **Type:** Boolean
- **Default:** `true`
- **Example:** `false` (if pre-sanitized)

---

### Processing Configuration

#### `MAX_UPLOAD_SIZE_BYTES`

- **Description:** Maximum file size for uploads
- **Type:** Integer (bytes)
- **Default:** `52428800` (50 MB)
- **Range:** 1024-10737418240 (1KB - 10GB)
- **Example:** `104857600` (100 MB)

#### `EXIFTOOL_PATH`

- **Description:** Path to exiftool binary
- **Type:** Path
- **Default:** `exiftool` (from PATH)
- **Example:** `/usr/local/bin/exiftool`

#### `EXIFTOOL_TIMEOUT_SECONDS`

- **Description:** Timeout for exiftool execution
- **Type:** Integer
- **Default:** `30`
- **Range:** 5-300
- **Example:** `60`

#### `THUMBNAIL_SIZE`

- **Description:** Thumbnail dimensions (square)
- **Type:** Integer (pixels)
- **Default:** `200`
- **Range:** 50-500
- **Example:** `300`

#### `THUMBNAIL_QUALITY`

- **Description:** JPEG quality for thumbnails
- **Type:** Integer (percentage)
- **Default:** `85`
- **Range:** 1-100
- **Example:** `90`

#### `PREVIEW_SIZE`

- **Description:** Preview max dimension (longest side)
- **Type:** Integer (pixels)
- **Default:** `1024`
- **Range:** 500-4096
- **Example:** `1920`

#### `PREVIEW_QUALITY`

- **Description:** JPEG quality for previews
- **Type:** Integer (percentage)
- **Default:** `90`
- **Range:** 1-100
- **Example:** `92`

#### `LOWRES_SIZE`

- **Description:** Low-res max dimension for RAW files
- **Type:** Integer (pixels)
- **Default:** `2048`
- **Range:** 1024-4096
- **Example:** `2560`

#### `LOWRES_QUALITY`

- **Description:** JPEG quality for low-res
- **Type:** Integer (percentage)
- **Default:** `92`
- **Range:** 1-100
- **Example:** `95`

#### `PROCESSING_TIMEOUT_SECONDS`

- **Description:** Global timeout for processing pipeline
- **Type:** Integer
- **Default:** `300` (5 minutes)
- **Range:** 60-3600
- **Example:** `600`

#### `PROCESSING_MAX_RETRIES`

- **Description:** Maximum retry attempts for failed processing
- **Type:** Integer
- **Default:** `3`
- **Range:** 0-10
- **Example:** `5`

#### `PROCESSING_RETRY_DELAY_MS`

- **Description:** Initial retry delay (exponential backoff)
- **Type:** Integer (milliseconds)
- **Default:** `1000` (1 second)
- **Range:** 100-10000
- **Example:** `2000`

---

### Event Bus Configuration

#### `EVENT_BUS_TYPE`

- **Description:** Event bus implementation
- **Type:** Enum
- **Default:** `in_memory`
- **Values:** `in_memory`, `nats` (future), `kafka` (future)
- **Example:** `in_memory`

#### `EVENT_BUS_BUFFER_SIZE`

- **Description:** Event channel buffer size
- **Type:** Integer
- **Default:** `1000`
- **Range:** 100-10000
- **Example:** `5000`

#### `NATS_URL` (Future)

- **Description:** NATS server URL
- **Type:** URL
- **Required:** If EVENT_BUS_TYPE=nats
- **Example:** `nats://localhost:4222`

---

### Quota Configuration

#### `QUOTA_RESERVATION_TIMEOUT_SECONDS`

- **Description:** How long quota reservations are valid
- **Type:** Integer
- **Default:** `3600` (1 hour)
- **Range:** 300-86400
- **Example:** `7200` (2 hours)

#### `QUOTA_CLEANUP_INTERVAL_SECONDS`

- **Description:** How often to cleanup expired reservations
- **Type:** Integer
- **Default:** `300` (5 minutes)
- **Range:** 60-3600
- **Example:** `600`

---

### Logging and Tracing

#### `RUST_LOG`

- **Description:** Log level and filtering
- **Type:** String
- **Default:** `info`
- **Format:** `target=level,target=level` or just `level`
- **Levels:** `error`, `warn`, `info`, `debug`, `trace`

**Examples:**

```bash
# Everything at info level
RUST_LOG=info

# Photonic at debug, everything else at warn
RUST_LOG=warn,infrastructure=debug

# Detailed logging for specific module
RUST_LOG=info,infrastructure::domain::medium=trace

# SQL query logging
RUST_LOG=info,sqlx=debug
```

#### `LOG_FORMAT`

- **Description:** Log output format
- **Type:** Enum
- **Default:** `json`
- **Values:** `json`, `pretty`, `compact`
- **Example:** `pretty` (dev)

#### `LOG_INCLUDE_TIMESTAMPS`

- **Description:** Include timestamps in logs
- **Type:** Boolean
- **Default:** `true`
- **Example:** `false` (if using external logger)

#### `OTEL_EXPORTER_OTLP_ENDPOINT`

- **Description:** OpenTelemetry collector endpoint
- **Type:** URL
- **Default:** Not set (tracing disabled)
- **Example:** `http://localhost:4317`

#### `OTEL_SERVICE_NAME`

- **Description:** Service name for tracing
- **Type:** String
- **Default:** `photonic`
- **Example:** `photonic-api-prod`

#### `OTEL_TRACES_SAMPLER`

- **Description:** Trace sampling strategy
- **Type:** Enum
- **Default:** `parentbased_always_on`
- **Values:** `always_on`, `always_off`, `traceidratio`, `parentbased_always_on`
- **Example:** `traceidratio`

#### `OTEL_TRACES_SAMPLER_ARG`

- **Description:** Sampler argument (e.g., ratio)
- **Type:** Float
- **Default:** `1.0` (100%)
- **Range:** 0.0-1.0
- **Example:** `0.1` (10% sampling)

---

### Feature Flags

#### `FEATURE_IMAGE_RECOGNITION`

- **Description:** Enable image recognition (future)
- **Type:** Boolean
- **Default:** `false`
- **Example:** `true`

#### `FEATURE_VIDEO_PROCESSING`

- **Description:** Enable video processing
- **Type:** Boolean
- **Default:** `false`
- **Example:** `true`

#### `FEATURE_LIVE_PHOTOS`

- **Description:** Enable live photo support
- **Type:** Boolean
- **Default:** `false`
- **Example:** `true`

---

## Configuration File (Optional)

Instead of environment variables, use a TOML configuration file:

**Location:** `/etc/photonic/config.toml` or set via `PHOTONIC_CONFIG`

**Example:**

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[jwt]
issuer = "https://idp.example.com"
audience = "photonic-api"
jwks_url = "https://idp.example.com/.well-known/jwks.json"
validate_expiry = true
leeway_seconds = 60

[database]
url = "postgresql://photonic:secret@localhost:5432/photonic"
max_connections = 20
min_connections = 5
acquire_timeout_seconds = 30
idle_timeout_seconds = 600
max_lifetime_seconds = 1800
run_migrations = true

[storage]
base_path = "/var/photonic/storage"
temp_path = "/mnt/ssd/photonic/temp"
permanent_path = "/mnt/hdd/photonic/permanent"
cache_path = "/mnt/ssd/photonic/cache"
provider = "filesystem"
ensure_directories = true

[storage.path_pattern]
pattern = "{year}/{month}/{camera_make}/{filename}"
fallback_date = "upload_time"
fallback_camera = "Unknown"
sanitize = true

[processing]
max_upload_size_bytes = 52428800
exiftool_path = "/usr/bin/exiftool"
exiftool_timeout_seconds = 30
thumbnail_size = 200
thumbnail_quality = 85
preview_size = 1024
preview_quality = 90
lowres_size = 2048
lowres_quality = 92
timeout_seconds = 300
max_retries = 3
retry_delay_ms = 1000

[quota]
reservation_timeout_seconds = 3600
cleanup_interval_seconds = 300

[event_bus]
type = "in_memory"
buffer_size = 1000

[logging]
level = "info"
format = "json"
include_timestamps = true

[tracing]
otlp_endpoint = "http://localhost:4317"
service_name = "photonic-api-prod"
sampler = "parentbased_always_on"
sampler_arg = 1.0

[features]
image_recognition = false
video_processing = false
live_photos = false
```

---

## Deployment Examples

### Development (.env)

```bash
# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# JWT
JWT_ISSUER=https://idp.dev.example.com
JWT_AUDIENCE=infrastructure-dev
JWT_JWKS_URL=https://idp.dev.example.com/.well-known/jwks.json
JWT_VALIDATE_EXPIRY=false

# Database
DATABASE_URL=postgresql://infrastructure:devpass@localhost:5432/photonic_dev
DATABASE_MAX_CONNECTIONS=5
DATABASE_RUN_MIGRATIONS=true

# Storage
STORAGE_BASE_PATH=./storage
STORAGE_PATH_PATTERN={year}/{month}/{filename}

# Processing
MAX_UPLOAD_SIZE_BYTES=104857600
EXIFTOOL_PATH=/usr/local/bin/exiftool
THUMBNAIL_SIZE=150

# Logging
RUST_LOG=debug,infrastructure=trace
LOG_FORMAT=pretty
OTEL_EXPORTER_OTLP_ENDPOINT=

# Features
FEATURE_VIDEO_PROCESSING=false
```

---

### Production (.env)

```bash
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
SERVER_WORKERS=8

# JWT
JWT_ISSUER=https://idp.example.com
JWT_AUDIENCE=infrastructure-api
JWT_JWKS_URL=https://idp.example.com/.well-known/jwks.json
JWT_VALIDATE_EXPIRY=true
JWT_LEEWAY_SECONDS=60

# Database
DATABASE_URL=postgresql://infrastructure:${DB_PASSWORD}@db.internal:5432/infrastructure
DATABASE_MAX_CONNECTIONS=50
DATABASE_MIN_CONNECTIONS=10
DATABASE_ACQUIRE_TIMEOUT_SECONDS=30
DATABASE_IDLE_TIMEOUT_SECONDS=600
DATABASE_MAX_LIFETIME_SECONDS=1800
DATABASE_RUN_MIGRATIONS=false

# Storage
STORAGE_BASE_PATH=/mnt/infrastructure
STORAGE_TEMP_PATH=/mnt/ssd/infrastructure/temp
STORAGE_PERMANENT_PATH=/mnt/hdd/infrastructure/permanent
STORAGE_CACHE_PATH=/mnt/ssd/infrastructure/cache
STORAGE_PATH_PATTERN={year}/{month}/{camera_make}/{filename}
STORAGE_ENSURE_DIRECTORIES=true

# Processing
MAX_UPLOAD_SIZE_BYTES=52428800
EXIFTOOL_PATH=/usr/bin/exiftool
EXIFTOOL_TIMEOUT_SECONDS=60
THUMBNAIL_SIZE=200
THUMBNAIL_QUALITY=85
PREVIEW_SIZE=1920
PREVIEW_QUALITY=92
LOWRES_SIZE=2560
LOWRES_QUALITY=95
PROCESSING_TIMEOUT_SECONDS=600
PROCESSING_MAX_RETRIES=3

# Quota
QUOTA_RESERVATION_TIMEOUT_SECONDS=3600
QUOTA_CLEANUP_INTERVAL_SECONDS=300

# Event Bus
EVENT_BUS_TYPE=in_memory
EVENT_BUS_BUFFER_SIZE=5000

# Logging
RUST_LOG=info,infrastructure=debug
LOG_FORMAT=json
LOG_INCLUDE_TIMESTAMPS=true

# Tracing
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
OTEL_SERVICE_NAME=infrastructure-api-prod
OTEL_TRACES_SAMPLER=parentbased_always_on
OTEL_TRACES_SAMPLER_ARG=1.0

# Features
FEATURE_IMAGE_RECOGNITION=false
FEATURE_VIDEO_PROCESSING=true
FEATURE_LIVE_PHOTOS=false
```

---

### Docker Compose

```yaml
version: '3.8'

services:
  photonic:
    image: photonic:latest
    ports:
      - "8080:8080"
    environment:
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - JWT_ISSUER=https://idp.example.com
      - JWT_AUDIENCE=infrastructure-api
      - JWT_JWKS_URL=https://idp.example.com/.well-known/jwks.json
      - DATABASE_URL=postgresql://infrastructure:secret@postgres:5432/infrastructure
      - STORAGE_BASE_PATH=/storage
      - STORAGE_PATH_PATTERN={year}/{month}/{camera_make}/{filename}
      - RUST_LOG=info
    volumes:
      - infrastructure-storage:/storage
    depends_on:
      - postgres

  postgres:
    image: postgres:16
    environment:
      - POSTGRES_USER=infrastructure
      - POSTGRES_PASSWORD=secret
      - POSTGRES_DB=infrastructure
    volumes:
      - postgres-data:/var/lib/postgresql/data

volumes:
  photonic-storage:
  postgres-data:
```

---

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: photonic-config
data:
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "8080"
  SERVER_WORKERS: "8"
  JWT_ISSUER: "https://idp.example.com"
  JWT_AUDIENCE: "infrastructure-api"
  JWT_JWKS_URL: "https://idp.example.com/.well-known/jwks.json"
  STORAGE_BASE_PATH: "/storage"
  STORAGE_PATH_PATTERN: "{year}/{month}/{camera_make}/{filename}"
  MAX_UPLOAD_SIZE_BYTES: "52428800"
  RUST_LOG: "info,infrastructure=debug"
  LOG_FORMAT: "json"
  OTEL_EXPORTER_OTLP_ENDPOINT: "http://otel-collector:4317"
  OTEL_SERVICE_NAME: "infrastructure-api"

---
apiVersion: v1
kind: Secret
metadata:
  name: photonic-secrets
type: Opaque
stringData:
  DATABASE_URL: "postgresql://infrastructure:secret@postgres-service:5432/infrastructure"
```

---

## Validation and Testing

### Validate Configuration

```bash
# Dry-run to validate configuration
infrastructure --config-check

# Show effective configuration (merged from all sources)
infrastructure --show-config
```

### Test Database Connection

```bash
# Test database connection and run migrations
infrastructure --test-db
```

### Test Storage Paths

```bash
# Verify storage paths are writable
infrastructure --test-storage
```

---

## Security Best Practices

### Secrets Management

**DO NOT** commit secrets to version control:

- Use `.env` files (add to `.gitignore`)
- Use environment variables in production
- Use secret management systems (HashiCorp Vault, AWS Secrets Manager, etc.)
- Use Kubernetes Secrets for K8s deployments

### File Permissions

```bash
# Configuration file
chmod 600 /etc/infrastructure/config.toml

# Storage directories
chown -R infrastructure:infrastructure /var/infrastructure/storage
chmod 750 /var/infrastructure/storage
```

### Database Security

- Use strong passwords
- Enable SSL/TLS for database connections
- Use separate read-only database user for queries
- Regularly rotate credentials

---

## Summary

### Required Configuration

Minimum required environment variables:

```bash
JWT_ISSUER=https://idp.example.com
JWT_AUDIENCE=infrastructure-api
JWT_JWKS_URL=https://idp.example.com/.well-known/jwks.json
DATABASE_URL=postgresql://user:pass@host:5432/db
STORAGE_BASE_PATH=/var/infrastructure/storage
```

### Recommended for Production

Additional recommended configuration:

```bash
SERVER_WORKERS=8
DATABASE_MAX_CONNECTIONS=50
STORAGE_PATH_PATTERN={year}/{month}/{camera_make}/{filename}
MAX_UPLOAD_SIZE_BYTES=52428800
PROCESSING_TIMEOUT_SECONDS=600
RUST_LOG=info,infrastructure=debug
OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317
```

All configuration options are validated on startup, and the application will fail fast with clear
error messages if misconfigured.