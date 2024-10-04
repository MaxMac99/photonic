use crate::{error::Result, ksqldb::KsqlDb};
use byte_unit::Byte;
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_current_quota_usage(ksql_db: Arc<KsqlDb>, user_id: Uuid) -> Result<Byte> {
    // TODO
    Ok(Byte::from_u64(0))
}
