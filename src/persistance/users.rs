use crate::errors::SnitchError;

type Result<T> = std::result::Result<T, SnitchError>;

#[derive(Debug)]
pub struct Users {
    pub users: Vec<String>,
}

impl Users {
    pub fn user_exists(&self, user: &String) -> bool {
        self.users.contains(user)
    }

    pub fn add_user(&mut self, user: String) -> Result<String> {
        if self.user_exists(&user) {
            return Err(SnitchError {});
        }
        self.users.push(user.clone());
        Ok(user)
    }

    pub fn delete_user(&mut self, user: String) -> Result<String> {
        if !self.user_exists(&user) {
            return Err(SnitchError {});
        }
        self.users.drain_filter(|iter_user| iter_user == &user);
        Ok(user)
    }

    pub fn get_users(&self) -> Result<Vec<String>> {
        // this is a stupid call
        // self.users.get
        Ok(self.users.clone())
    }

    pub fn get_user_by_id(&self, user: String) -> Result<String> {
        if !self.user_exists(&user) {
            return Err(SnitchError {});
        }
        // this is a stupid call
        // self.users.get
        Ok(user)
    }
}
