//! Database DAO
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct InstanceOperationDao {
    _pool: MySqlPool,
}

impl InstanceOperationDao {
    pub fn new(pool: MySqlPool) -> Self {
        Self { _pool: pool }
    }
}
