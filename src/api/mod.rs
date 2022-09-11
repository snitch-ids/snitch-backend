pub(crate) mod authentication;
pub mod messages;
pub mod users;
use tokio::sync::Mutex;

use crate::persistance::redis::RedisDatabaseService;

pub struct AppStateWithCounter {
    pub messages: Mutex<RedisDatabaseService>,
}
