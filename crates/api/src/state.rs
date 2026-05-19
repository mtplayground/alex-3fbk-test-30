use redis::aio::ConnectionManager;
use sqlx::PgPool;
use zeroclaw_core::JwtConfig;

#[derive(Clone)]
pub struct AppState {
    db_pool: PgPool,
    redis_manager: ConnectionManager,
    jwt: JwtConfig,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_manager: ConnectionManager, jwt: JwtConfig) -> Self {
        Self {
            db_pool,
            redis_manager,
            jwt,
        }
    }

    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    pub fn redis_manager(&self) -> ConnectionManager {
        self.redis_manager.clone()
    }

    pub const fn jwt(&self) -> &JwtConfig {
        &self.jwt
    }
}
