//! Helper functions allowing you to avoid writing boilerplate code for common operations, such as
//! parsing JSON or reading files.

// Copyright (c) 2016 Google Inc (lewinb@google.com).
//
// Refer to the project root for licensing information.
use crate::authorized_user::AuthorizedUserSecret;
use crate::external_account::ExternalAccountSecret;
use crate::types::{ApplicationSecret, ConsoleApplicationSecret};

#[cfg(feature = "service-account")]
use crate::service_account::ServiceAccountKey;

use std::io;
use std::path::Path;

/// Read an application secret from a file.
pub async fn read_application_secret<P: AsRef<Path>>(path: P) -> io::Result<ApplicationSecret> {
    parse_application_secret(tokio::fs::read(path).await?)
}

/// Read an application secret from a JSON string.
pub fn parse_application_secret<S: AsRef<[u8]>>(secret: S) -> io::Result<ApplicationSecret> {
    let decoded: ConsoleApplicationSecret =
        serde_json::from_slice(secret.as_ref()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Bad application secret: {}", e),
            )
        })?;

    if let Some(web) = decoded.web {
        Ok(web)
    } else if let Some(installed) = decoded.installed {
        Ok(installed)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unknown application secret format",
        ))
    }
}

/// Read a service account key from a JSON file. You can download the JSON keys from the Google
/// Cloud Console or the respective console of your service provider.
#[cfg(feature = "service-account")]
pub async fn read_service_account_key<P: AsRef<Path>>(path: P) -> io::Result<ServiceAccountKey> {
    let key = tokio::fs::read(path).await?;
    parse_service_account_key(key)
}

#[cfg(feature = "service-account")]
/// Read a service account key from a JSON string.
pub fn parse_service_account_key<S: AsRef<[u8]>>(key: S) -> io::Result<ServiceAccountKey> {
    serde_json::from_slice(key.as_ref()).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Bad service account key: {}", e),
        )
    })
}

/// Read an authorized user secret from a JSON file. You can obtain it by running on the client:
/// `gcloud auth application-default login`.
/// The file should be on Windows in: `%APPDATA%/gcloud/application_default_credentials.json`
/// for other systems: `$HOME/.config/gcloud/application_default_credentials.json`.
pub async fn read_authorized_user_secret<P: AsRef<Path>>(
    path: P,
) -> io::Result<AuthorizedUserSecret> {
    let key = tokio::fs::read(path).await?;
    serde_json::from_slice(&key).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Bad authorized user secret: {}", e),
        )
    })
}

/// Read an external account secret from a JSON file.
pub async fn read_external_account_secret<P: AsRef<Path>>(
    path: P,
) -> io::Result<ExternalAccountSecret> {
    let key = tokio::fs::read(path).await?;
    serde_json::from_slice(&key).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Bad external account secret: {}", e),
        )
    })
}

pub(crate) fn join<T>(pieces: &[T], separator: &str) -> String
where
    T: AsRef<str>,
{
    let mut iter = pieces.iter();
    let first = match iter.next() {
        Some(p) => p,
        None => return String::new(),
    };
    let num_separators = pieces.len() - 1;
    let pieces_size: usize = pieces.iter().map(|p| p.as_ref().len()).sum();
    let size = pieces_size + separator.len() * num_separators;
    let mut result = String::with_capacity(size);
    result.push_str(first.as_ref());
    for p in iter {
        result.push_str(separator);
        result.push_str(p.as_ref());
    }
    debug_assert_eq!(size, result.len());
    result
}
