
//! A Rust library for sending Firebase Cloud Messaging (FCM) notifications.
//!
//! This crate provides a simple interface to send push notifications using Firebase Cloud Messaging (FCM).
//! It handles authentication with Google OAuth2 and constructs the necessary payloads for FCM requests.
//!
//! # Example
//! ```rust
//! use fcm_notification_service::{FcmNotificationService, NotificationPayload};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let fcm_service = FcmNotificationService::new("service_account.json")?;
//!     let notification = NotificationPayload {
//!         token: "device-token-here",
//!         title: "New Like",
//!         body: "Someone liked your post!",
//!         data: None,
//!     };
//!     fcm_service.send_notification(&notification).await?;
//!     Ok(())
//! }
//! ```

use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{error::Error, fs};
use thiserror::Error;

/// Represents a Firebase service account, loaded from a JSON file.
///
/// This struct is used to store the credentials required to authenticate with Google OAuth2
/// and send FCM notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccount {
    #[serde(rename = "type")]
    pub account_type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String,
    pub universe_domain: String,
}

/// Represents the payload for an FCM notification.
///
/// This struct is used to define the content of the notification, including the target device token,
/// the title, the body, and optional additional data.
#[derive(Debug, Serialize)]
pub struct NotificationPayload<'a> {
    /// The device token of the target device.
    pub token: &'a str,
    /// The title of the notification.
    pub title: &'a str,
    /// The body of the notification.
    pub body: &'a str,
    /// Optional additional data to include in the notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Represents errors that can occur while using the `FcmNotificationService`.
///
/// This enum provides a unified error type for all operations, including file I/O, JSON parsing,
/// JWT encoding, HTTP requests, and FCM-specific errors.
#[derive(Debug, Error)]
pub enum FcmError {
    #[error("Failed to read service account file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse service account JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Failed to encode JWT: {0}")]
    JwtEncodeError(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to send HTTP request: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Access token not found in response")]
    AccessTokenNotFound,
    #[error("Failed to send notification: {0}")]
    NotificationError(String),
}

/// The main service for sending FCM notifications.
///
/// This struct provides methods to authenticate with Google OAuth2 and send notifications
/// using the Firebase Cloud Messaging API.
#[derive(Clone)]
pub struct FcmNotificationService {
    service_account: ServiceAccount,
    client: Client,
}

impl FcmNotificationService {
    /// Creates a new `FcmNotificationService` instance.
    ///
    /// # Arguments
    /// * `config_path` - The path to the Firebase service account JSON file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or the JSON cannot be parsed.
    pub fn new(config_path: &str) -> Result<Self, FcmError> {
        let config_file = fs::read_to_string(config_path)?;
        let service_account: ServiceAccount = serde_json::from_str(&config_file)?;

        Ok(Self {
            service_account,
            client: Client::new(),
        })
    }

    /// Generates an OAuth2 access token using the service account credentials.
    ///
    /// This method creates a JWT (JSON Web Token) and exchanges it for an access token
    /// using the Google OAuth2 token endpoint.
    ///
    /// # Errors
    /// Returns an error if the JWT cannot be encoded or the HTTP request fails.
    async fn get_access_token(&self) -> Result<String, FcmError> {
        #[derive(Serialize)]
        struct Claims {
            iss: String,
            scope: String,
            aud: String,
            exp: i64,
            iat: i64,
        }

        let now = Utc::now();
        let claims = Claims {
            iss: self.service_account.client_email.clone(),
            scope: "https://www.googleapis.com/auth/firebase.messaging".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(),
            exp: (now + chrono::Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
        };

        let encoding_key = EncodingKey::from_rsa_pem(self.service_account.private_key.as_bytes())?;
        let jwt = encode(
            &Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &encoding_key,
        )?;

        let params = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ];

        let response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let access_token = response["access_token"]
            .as_str()
            .ok_or(FcmError::AccessTokenNotFound)?
            .to_string();

        Ok(access_token)
    }

    /// Sends an FCM notification to the specified device.
    ///
    /// # Arguments
    /// * `notification` - The notification payload containing the device token, title, body, and optional data.
    ///
    /// # Errors
    /// Returns an error if the access token cannot be retrieved or the HTTP request fails.
    pub async fn send_notification(
        &self,
        notification: &NotificationPayload<'_>,
    ) -> Result<(), FcmError> {
        let access_token = self.get_access_token().await?;

        let notification_payload = json!({
            "message": {
                "token": notification.token,
                "notification": {
                    "title": notification.title,
                    "body": notification.body
                },
                "data": notification.data
            }
        });

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.service_account.project_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .json(&notification_payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("Notification sent successfully");
            Ok(())
        } else {
            Err(FcmError::NotificationError(response.text().await?))
        }
    }
}
