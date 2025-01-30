# FCM Notification Service in Rust

This repository provides a Rust implementation for sending push notifications using Firebase Cloud Messaging (FCM). It uses a service account JSON file for authentication and communicates with the FCM API to send notifications.

## Prerequisites

#### 1. Service Account JSON File

Create a service account in your Firebase project and download the JSON key file. Save it in your project's root directory with the name service_account.json.
The JSON file should have the following structure:

```json
{
  "type": "service_account",
  "project_id": "your-project-id",
  "private_key_id": "your-private-key-id",
  "private_key": "your-private-key",
  "client_email": "your-client-email",
  "client_id": "your-client-id",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "your-cert-url",
  "universe_domain": "googleapis.com"
}
```

#### 2. Platforms

This code works for platforms where Firebase Cloud Messaging is supported, such as:

* iOS
* Android
* Web

## How to Use

#### 1. Initialize the FCM Notification Service

Load the service_account.json file to create an instance of FcmNotificationService:

``` rust
let fcm_service = FcmNotificationService::new("service_account.json")?;
```

#### 2. Create and Send a Notification

Define a NotificationPayload with the required fields, such as token, title, body, and optional custom data:

```rust
let notification = NotificationPayload {
    token: "device-token-here",
    title: "New Like",
    body: "Someone liked your post!",
    data: None, // or Some(json!({"key": "value"}))
};

fcm_service.send_notification(&notification).await?;
```

## Dependencies

This implementation uses the following crates:

* chrono: For handling timestamps.
* jsonwebtoken: For creating JWT tokens.
* reqwest: For making HTTP requests.
* serde and serde_json: For serializing and deserializing JSON.
