use crate::model::user::UserID;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct NotificationFilter {
    pub(crate) last_notifications: HashMap<UserID, DateTime<Utc>>,
}

impl NotificationFilter {
    pub(crate) fn new() -> Self {
        Self {
            last_notifications: HashMap::new(),
        }
    }

    pub(crate) async fn notify_user(&mut self, key: &UserID) -> bool {
        self.cleanup().await;
        if self.last_notifications.contains_key(&key) {
            return false;
        }
        self.last_notifications.insert(key.clone(), Utc::now());
        true
    }

    async fn cleanup(&mut self) {
        let mut now = Utc::now();
        self.last_notifications
            .retain(|_, timeout| timeout > &mut now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanup() {
        let mut handler = NotificationFilter::new();
        handler.cleanup().await;
    }
}
