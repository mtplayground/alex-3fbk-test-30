use std::env;
use std::fmt;

use crate::error::{ConfigError, Result};

const DATABASE_URL_ENV: &str = "DATABASE_URL";
const HOST_ENV: &str = "HOST";
const JWT_SECRET_ENV: &str = "JWT_SECRET";
const PORT_ENV: &str = "PORT";
const PUBLIC_BASE_URL_ENV: &str = "PUBLIC_BASE_URL";
const REDIS_KEY_PREFIX_ENV: &str = "REDIS_KEY_PREFIX";
const REDIS_URL_ENV: &str = "REDIS_URL";
const S3_ACCESS_KEY_ID_ENV: &str = "S3_ACCESS_KEY_ID";
const S3_BUCKET_ENV: &str = "S3_BUCKET";
const S3_ENDPOINT_ENV: &str = "S3_ENDPOINT";
const S3_REGION_ENV: &str = "S3_REGION";
const S3_SECRET_ACCESS_KEY_ENV: &str = "S3_SECRET_ACCESS_KEY";
const SERVICE_NAME_ENV: &str = "SERVICE_NAME";
const SMTP_FROM_ENV: &str = "SMTP_FROM";
const SMTP_HOST_ENV: &str = "SMTP_HOST";
const SMTP_PASSWORD_ENV: &str = "SMTP_PASSWORD";
const SMTP_PORT_ENV: &str = "SMTP_PORT";
const SMTP_USERNAME_ENV: &str = "SMTP_USERNAME";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRole {
    Api,
    Worker,
}

impl ServiceRole {
    pub const fn default_service_name(self) -> &'static str {
        match self {
            Self::Api => "zeroclaw-api",
            Self::Worker => "zeroclaw-worker",
        }
    }

    pub const fn default_port(self) -> u16 {
        match self {
            Self::Api => 8080,
            Self::Worker => 0,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Config {
    service_name: String,
    role: ServiceRole,
    host: String,
    port: u16,
    database_url: String,
    redis_url: String,
    redis_key_prefix: String,
    s3: S3Config,
    jwt: JwtConfig,
    smtp: SmtpConfig,
    public_base_url: String,
}

impl Config {
    pub fn from_env(role: ServiceRole) -> Result<Self> {
        let service_name = match optional_env(SERVICE_NAME_ENV)? {
            Some(value) => value,
            None => role.default_service_name().to_owned(),
        };

        let host = match optional_env(HOST_ENV)? {
            Some(value) => value,
            None => "0.0.0.0".to_owned(),
        };

        let port = match optional_env(PORT_ENV)? {
            Some(value) => parse_port(PORT_ENV, value)?,
            None => role.default_port(),
        };

        let redis_key_prefix = match optional_env(REDIS_KEY_PREFIX_ENV)? {
            Some(value) => value,
            None => "zeroclaw".to_owned(),
        };

        Ok(Self {
            service_name,
            role,
            host,
            port,
            database_url: required_url(DATABASE_URL_ENV)?,
            redis_url: required_url(REDIS_URL_ENV)?,
            redis_key_prefix,
            s3: S3Config::from_env()?,
            jwt: JwtConfig::from_env()?,
            smtp: SmtpConfig::from_env()?,
            public_base_url: required_url(PUBLIC_BASE_URL_ENV)?,
        })
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub const fn role(&self) -> ServiceRole {
        self.role
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    pub fn redis_key_prefix(&self) -> &str {
        &self.redis_key_prefix
    }

    pub const fn s3(&self) -> &S3Config {
        &self.s3
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

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Config")
            .field("service_name", &self.service_name)
            .field("role", &self.role)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database_url", &"<configured>")
            .field("redis_url", &"<configured>")
            .field("redis_key_prefix", &self.redis_key_prefix)
            .field("s3", &self.s3)
            .field("jwt", &self.jwt)
            .field("smtp", &self.smtp)
            .field("public_base_url", &self.public_base_url)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct S3Config {
    endpoint: String,
    bucket: String,
    access_key_id: String,
    secret_access_key: String,
    region: String,
}

impl S3Config {
    fn from_env() -> Result<Self> {
        let region = match optional_env(S3_REGION_ENV)? {
            Some(value) => value,
            None => "us-east-1".to_owned(),
        };

        Ok(Self {
            endpoint: required_url(S3_ENDPOINT_ENV)?,
            bucket: required_env(S3_BUCKET_ENV)?,
            access_key_id: required_env(S3_ACCESS_KEY_ID_ENV)?,
            secret_access_key: required_env(S3_SECRET_ACCESS_KEY_ENV)?,
            region,
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    pub fn secret_access_key(&self) -> &str {
        &self.secret_access_key
    }

    pub fn region(&self) -> &str {
        &self.region
    }
}

impl fmt::Debug for S3Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("S3Config")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"<redacted>")
            .field("region", &self.region)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct JwtConfig {
    secret: String,
}

impl JwtConfig {
    pub fn from_secret(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    fn from_env() -> Result<Self> {
        Ok(Self {
            secret: required_env(JWT_SECRET_ENV)?,
        })
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }
}

impl fmt::Debug for JwtConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JwtConfig")
            .field("secret", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    from: String,
}

impl SmtpConfig {
    fn from_env() -> Result<Self> {
        let port = match optional_env(SMTP_PORT_ENV)? {
            Some(value) => parse_port(SMTP_PORT_ENV, value)?,
            None => 587,
        };

        Ok(Self {
            host: required_env(SMTP_HOST_ENV)?,
            port,
            username: required_env(SMTP_USERNAME_ENV)?,
            password: required_env(SMTP_PASSWORD_ENV)?,
            from: required_env(SMTP_FROM_ENV)?,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn from(&self) -> &str {
        &self.from
    }
}

impl fmt::Debug for SmtpConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SmtpConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .field("from", &self.from)
            .finish()
    }
}

fn required_env(name: &'static str) -> Result<String> {
    let value = env::var(name).map_err(|error| ConfigError::from_var_error(name, error))?;

    if value.trim().is_empty() {
        return Err(ConfigError::EmptyEnv { name });
    }

    Ok(value)
}

fn required_url(name: &'static str) -> Result<String> {
    let value = required_env(name)?;

    if is_url_like(&value) {
        return Ok(value);
    }

    Err(ConfigError::InvalidUrl { name, value })
}

fn is_url_like(value: &str) -> bool {
    let Some((scheme, rest)) = value.split_once("://") else {
        return false;
    };

    let has_valid_scheme = !scheme.is_empty()
        && scheme.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        });

    has_valid_scheme && !rest.trim().is_empty()
}

fn optional_env(name: &'static str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) if value.trim().is_empty() => Err(ConfigError::EmptyEnv { name }),
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(ConfigError::from_var_error(name, error)),
    }
}

fn parse_port(name: &'static str, value: String) -> Result<u16> {
    value
        .parse::<u16>()
        .map_err(|source| ConfigError::InvalidPort {
            name,
            value,
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_shape_validator_accepts_common_runtime_urls() {
        assert!(is_url_like(
            "postgres://user:password@localhost:5432/zeroclaw"
        ));
        assert!(is_url_like("redis://localhost:6379"));
        assert!(is_url_like("http://localhost:9000"));
        assert!(is_url_like("https://example.com"));
    }

    #[test]
    fn url_shape_validator_rejects_missing_or_empty_parts() {
        assert!(!is_url_like("localhost:5432"));
        assert!(!is_url_like("://localhost:5432"));
        assert!(!is_url_like("postgres://"));
    }

    #[test]
    fn parse_port_rejects_non_numeric_values() {
        let result = parse_port(PORT_ENV, "not-a-port".to_owned());

        assert!(matches!(result, Err(ConfigError::InvalidPort { .. })));
    }
}
