use pushover::{Priority, API};
use pushover::requests::message::SendMessage;
use anyhow::Result;
use dotenvy::dotenv;
use std::env;

pub struct Notifier {
    pushover_api: API,
    pushover_user_key: String,
    pushover_app_token: String,
}

impl Notifier {
    pub fn new() -> Result<Self, anyhow::Error> {
        dotenv().ok();

        let pushover_user_key = env::var("PUSHOVER_KEY").expect("PUSHOVER_KEY must be set");
        let pushover_app_token = env::var("PUSHOVER_TOKEN").expect("PUSHOVER_TOKEN must be set");

        Ok(Self {
            pushover_api: API::new(),
            pushover_user_key,
            pushover_app_token,
        })
    }

    pub async fn send_pushover_message(
        &self,
        message: &str,
        priority: &Priority,
    ) -> Result<(), anyhow::Error> {
        let mut message_obj = SendMessage::new(
            &self.pushover_app_token,
            &self.pushover_user_key,
            message,
        );
        message_obj.set_priority(priority.clone());

        self.pushover_api.send(&message_obj)
            .map_err(|e| anyhow::anyhow!("Pushover API error: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore = "requires live Pushover credentials and sends a real notification"]
    async fn test_send_pushover_message() {
        let notifier = Notifier::new().unwrap();
        let result = notifier.send_pushover_message("Testing", &Priority::Normal).await;
        assert!(result.is_ok());
    }
}
