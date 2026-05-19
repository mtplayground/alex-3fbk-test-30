use redis::aio::ConnectionManager;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    db_pool: PgPool,
    redis_manager: ConnectionManager,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_manager: ConnectionManager) -> Self {
        Self {
            db_pool,
            redis_manager,
        }
    }

    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    pub fn redis_manager(&self) -> ConnectionManager {
        self.redis_manager.clone()
    }
}
