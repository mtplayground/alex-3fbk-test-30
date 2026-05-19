use ::redis::aio::{ConnectionManager, PubSub};
use ::redis::{AsyncCommands, ErrorKind, RedisError, RedisResult, ToRedisArgs};

use crate::Config;

const SEPARATOR: &str = ":";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisNamespace {
    prefix: String,
}

impl RedisNamespace {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: clean_segment(&prefix.into()),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self::new(config.redis_key_prefix())
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn key<I, S>(&self, parts: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.join(parts)
    }

    pub fn channel<I, S>(&self, parts: I) -> RedisChannel
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        RedisChannel::new(self.join(parts))
    }

    fn join<I, S>(&self, parts: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut segments = Vec::new();

        if !self.prefix.is_empty() {
            segments.push(self.prefix.clone());
        }

        for part in parts {
            let cleaned = clean_segment(part.as_ref());
            if !cleaned.is_empty() {
                segments.push(cleaned);
            }
        }

        segments.join(SEPARATOR)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedisChannel {
    name: String,
}

impl RedisChannel {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone)]
pub struct RedisClient {
    client: ::redis::Client,
    namespace: RedisNamespace,
}

impl RedisClient {
    pub fn new(config: &Config) -> RedisResult<Self> {
        Ok(Self {
            client: ::redis::Client::open(config.redis_url())?,
            namespace: RedisNamespace::from_config(config),
        })
    }

    pub fn namespace(&self) -> &RedisNamespace {
        &self.namespace
    }

    pub async fn connection_manager(&self) -> RedisResult<ConnectionManager> {
        self.client.get_connection_manager().await
    }

    pub async fn publish<P>(
        &self,
        manager: &mut ConnectionManager,
        channel: &RedisChannel,
        payload: P,
    ) -> RedisResult<usize>
    where
        P: ToRedisArgs + Send + Sync,
    {
        manager.publish(channel.as_str(), payload).await
    }

    pub async fn subscribe(&self, channels: &[RedisChannel]) -> RedisResult<PubSub> {
        let mut pubsub = self.client.get_async_pubsub().await?;

        for channel in channels {
            pubsub.subscribe(channel.as_str()).await?;
        }

        Ok(pubsub)
    }
}

pub async fn health_check(manager: &mut ConnectionManager) -> RedisResult<()> {
    let response: String = ::redis::cmd("PING").query_async(manager).await?;

    if response == "PONG" {
        return Ok(());
    }

    Err(RedisError::from((
        ErrorKind::ResponseError,
        "unexpected Redis PING response",
    )))
}

fn clean_segment(segment: &str) -> String {
    segment.trim().trim_matches(':').to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_joins_keys_with_single_separator() {
        let namespace = RedisNamespace::new(":zeroclaw:");

        assert_eq!(
            namespace.key(["feed", ":home:", "user-123"]),
            "zeroclaw:feed:home:user-123"
        );
    }

    #[test]
    fn namespace_skips_empty_segments() {
        let namespace = RedisNamespace::new("zeroclaw");

        assert_eq!(namespace.key(["", "posts", "::"]), "zeroclaw:posts");
    }

    #[test]
    fn namespace_allows_empty_prefix() {
        let namespace = RedisNamespace::new("");

        assert_eq!(namespace.key(["posts", "123"]), "posts:123");
    }

    #[test]
    fn namespace_builds_channels() {
        let namespace = RedisNamespace::new("zeroclaw");
        let channel = namespace.channel(["dm", "user-123"]);

        assert_eq!(channel.as_str(), "zeroclaw:dm:user-123");
    }
}
