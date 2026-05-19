use std::error::Error;
use std::time::Duration;

use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::presigning::{PresigningConfig, PresigningConfigError};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use thiserror::Error;

use crate::S3Config;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("invalid presigning configuration")]
    InvalidPresigningConfig(#[from] PresigningConfigError),

    #[error("S3 {operation} operation failed")]
    Operation {
        operation: &'static str,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresignedUrl {
    url: String,
    method: &'static str,
    expires_in: Duration,
}

impl PresignedUrl {
    pub fn new(url: impl Into<String>, method: &'static str, expires_in: Duration) -> Self {
        Self {
            url: url.into(),
            method,
            expires_in,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub const fn method(&self) -> &'static str {
        self.method
    }

    pub const fn expires_in(&self) -> Duration {
        self.expires_in
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectSummary {
    key: String,
    size: i64,
    e_tag: Option<String>,
    last_modified: Option<String>,
}

impl ObjectSummary {
    pub fn new(
        key: impl Into<String>,
        size: i64,
        e_tag: Option<String>,
        last_modified: Option<String>,
    ) -> Self {
        Self {
            key: key.into(),
            size,
            e_tag,
            last_modified,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub const fn size(&self) -> i64 {
        self.size
    }

    pub fn e_tag(&self) -> Option<&str> {
        self.e_tag.as_deref()
    }

    pub fn last_modified(&self) -> Option<&str> {
        self.last_modified.as_deref()
    }
}

#[derive(Clone)]
pub struct ObjectStorage {
    client: Client,
    bucket: String,
}

impl ObjectStorage {
    pub fn new(config: &S3Config) -> Self {
        let credentials = Credentials::new(
            config.access_key_id(),
            config.secret_access_key(),
            None,
            None,
            "zeroclaw-config",
        );

        let s3_config = aws_sdk_s3::Config::builder()
            .region(Region::new(config.region().to_owned()))
            .credentials_provider(credentials)
            .endpoint_url(config.endpoint())
            .force_path_style(true)
            .build();

        Self {
            client: Client::from_conf(s3_config),
            bucket: config.bucket().to_owned(),
        }
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub async fn presigned_put(
        &self,
        key: &str,
        expires_in: Duration,
        content_type: Option<&str>,
    ) -> Result<PresignedUrl, StorageError> {
        let presigning_config = PresigningConfig::expires_in(expires_in)?;
        let mut request = self.client.put_object().bucket(&self.bucket).key(key);

        if let Some(content_type) = content_type {
            request = request.content_type(content_type);
        }

        let presigned = request
            .presigned(presigning_config)
            .await
            .map_err(|error| storage_error("presigned_put", error))?;

        Ok(PresignedUrl::new(
            presigned.uri().to_string(),
            "PUT",
            expires_in,
        ))
    }

    pub async fn presigned_get(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<PresignedUrl, StorageError> {
        let presigning_config = PresigningConfig::expires_in(expires_in)?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|error| storage_error("presigned_get", error))?;

        Ok(PresignedUrl::new(
            presigned.uri().to_string(),
            "GET",
            expires_in,
        ))
    }

    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| storage_error("delete", error))?;

        Ok(())
    }

    pub async fn get_bytes(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|error| storage_error("get_bytes", error))?;

        let bytes = response
            .body
            .collect()
            .await
            .map_err(|error| storage_error("get_bytes", error))?
            .into_bytes()
            .to_vec();

        Ok(bytes)
    }

    pub async fn put_bytes(
        &self,
        key: &str,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<(), StorageError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await
            .map_err(|error| storage_error("put_bytes", error))?;

        Ok(())
    }

    pub async fn list_by_prefix(&self, prefix: &str) -> Result<Vec<ObjectSummary>, StorageError> {
        let mut continuation_token = None;
        let mut objects = Vec::new();

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|error| storage_error("list_by_prefix", error))?;

            for object in response.contents() {
                let Some(key) = object.key() else {
                    continue;
                };

                objects.push(ObjectSummary::new(
                    key,
                    match object.size() {
                        Some(size) => size,
                        None => 0,
                    },
                    object.e_tag().map(ToOwned::to_owned),
                    object.last_modified().map(ToString::to_string),
                ));
            }

            continuation_token = response.next_continuation_token().map(ToOwned::to_owned);

            if continuation_token.is_none() {
                break;
            }
        }

        Ok(objects)
    }
}

fn storage_error(
    operation: &'static str,
    error: impl Error + Send + Sync + 'static,
) -> StorageError {
    StorageError::Operation {
        operation,
        source: Box::new(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presigned_url_exposes_method_url_and_expiry() {
        let expires_in = Duration::from_secs(300);
        let url = PresignedUrl::new("https://storage.example/object", "GET", expires_in);

        assert_eq!(url.url(), "https://storage.example/object");
        assert_eq!(url.method(), "GET");
        assert_eq!(url.expires_in(), expires_in);
    }

    #[test]
    fn object_summary_exposes_optional_metadata() {
        let summary = ObjectSummary::new(
            "media/original/image.jpg",
            1024,
            Some("etag".to_owned()),
            None,
        );

        assert_eq!(summary.key(), "media/original/image.jpg");
        assert_eq!(summary.size(), 1024);
        assert_eq!(summary.e_tag(), Some("etag"));
        assert_eq!(summary.last_modified(), None);
    }
}
