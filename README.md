# fcm_notification

A Rust library for sending Firebase Cloud Messaging (FCM) notifications.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
fcm-notification = "0.1.2"
```

## Usage

```rust
use fcm_notification::{FcmNotification, NotificationPayload};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fcm = FcmNotification::new("service_account.json")?;
    let notification = NotificationPayload {
        token: "device-token-here",
        title: "New Like",
        body: "Someone liked your post!",
        data: None,
    };
    fcm.send_notification(&notification).await?;
    Ok(())
}
```

## License

This project is licensed under the MIT License.
