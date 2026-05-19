use redis::aio::ConnectionManager;
use sqlx::PgPool;
use zeroclaw_core::redis::{RedisClient, RedisNamespace};
use zeroclaw_core::storage::ObjectStorage;
use zeroclaw_core::{JwtConfig, SmtpConfig};

#[derive(Clone)]
pub struct AppState {
    db_pool: PgPool,
    redis_manager: ConnectionManager,
    redis_client: RedisClient,
    redis_namespace: RedisNamespace,
    jwt: JwtConfig,
    smtp: SmtpConfig,
    public_base_url: String,
    storage: ObjectStorage,
}

impl AppState {
    pub fn new(
        db_pool: PgPool,
        redis_manager: ConnectionManager,
        redis_client: RedisClient,
        redis_namespace: RedisNamespace,
        jwt: JwtConfig,
        smtp: SmtpConfig,
        public_base_url: String,
        storage: ObjectStorage,
    ) -> Self {
        Self {
            db_pool,
            redis_manager,
            redis_client,
            redis_namespace,
            jwt,
            smtp,
            public_base_url,
            storage,
        }
    }

    pub fn db_pool(&self) -> &PgPool {
        &self.db_pool
    }

    pub fn redis_manager(&self) -> ConnectionManager {
        self.redis_manager.clone()
    }

    pub const fn redis_client(&self) -> &RedisClient {
        &self.redis_client
    }

    pub const fn redis_namespace(&self) -> &RedisNamespace {
        &self.redis_namespace
    }

    pub const fn jwt(&self) -> &JwtConfig {
        &self.jwt
    }

    pub const fn smtp(&self) -> &SmtpConfig {
        &self.smtp
    }

    pub fn public_base_url(&self) -> &str {
        &self.public_base_url
    }

    pub const fn storage(&self) -> &ObjectStorage {
        &self.storage
    }
}
