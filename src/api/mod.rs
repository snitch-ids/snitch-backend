pub mod messages;
pub mod users;
use tokio::sync::Mutex;

use crate::persistance::{mongodb::DatabaseService, users::Users};

pub struct AppStateWithCounter {
    pub users: Mutex<Users>,
    pub messages: Mutex<DatabaseService>,
}
