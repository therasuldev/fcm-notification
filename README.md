# fcm_notification

A Rust library for sending Firebase Cloud Messaging (FCM) notifications.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
fcm-notification = "0.1.1"
```

## Usage

```rust
use fcm_notification::{FcmNotificationService, NotificationPayload};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fcm_service = FcmNotificationService::new("service_account.json")?;
    let notification = NotificationPayload {
        token: "device-token-here",
        title: "New Like",
        body: "Someone liked your post!",
        data: None,
    };
    fcm_service.send_notification(&notification).await?;
    Ok(())
}
```

## License

This project is licensed under the MIT License.
