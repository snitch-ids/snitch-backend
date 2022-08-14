pub mod messages;
pub mod users;
use tokio::sync::Mutex;

use crate::persistance::{users::Users, redis::RedisDatabaseService};

pub struct AppStateWithCounter {
    pub users: Mutex<Users>,
    pub messages: Mutex<RedisDatabaseService>,
}
