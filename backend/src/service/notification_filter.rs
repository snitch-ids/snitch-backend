use crate::model::user::UserID;
use chrono::Utc;
use std::collections::HashMap;

const DEAD_TIME: i64 = 30;

#[derive(Debug)]
pub(crate) struct NotificationFilter {
    pub(crate) last_notifications: HashMap<UserID, i64>,
}

impl NotificationFilter {
    pub(crate) fn new() -> Self {
        Self {
            last_notifications: HashMap::new(),
        }
    }

    pub(crate) async fn notify_user(&mut self, key: &UserID) -> bool {
        self.cleanup().await;
        if self.last_notifications.contains_key(key) {
            return false;
        }
        self.last_notifications
            .insert(key.clone(), Utc::now().timestamp() + DEAD_TIME);
        true
    }

    async fn cleanup(&mut self) {
        let mut now = Utc::now().timestamp();
        self.last_notifications
            .retain(|_, time_expired| &mut now < time_expired)
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
