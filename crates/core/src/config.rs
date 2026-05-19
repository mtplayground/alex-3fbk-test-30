use std::env;

use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, Result};

const DATABASE_URL_ENV: &str = "DATABASE_URL";
const HOST_ENV: &str = "HOST";
const PORT_ENV: &str = "PORT";
const SERVICE_NAME_ENV: &str = "SERVICE_NAME";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    service_name: String,
    role: ServiceRole,
    host: String,
    port: u16,
    database_url: String,
}

impl AppConfig {
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

        Ok(Self {
            service_name,
            role,
            host,
            port,
            database_url: required_env(DATABASE_URL_ENV)?,
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

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn required_env(name: &'static str) -> Result<String> {
    let value = env::var(name).map_err(|error| ConfigError::from_var_error(name, error))?;

    if value.trim().is_empty() {
        return Err(ConfigError::EmptyEnv { name });
    }

    Ok(value)
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
