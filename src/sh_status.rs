use std::collections::HashMap;
use discord::model::UserId;

pub struct ShStatus {
    user_data: HashMap<UserId, UserData>,
}

struct UserData {
    wants_sh: bool,
}

impl ShStatus {
    pub fn new() -> Self {
        ShStatus { user_data: HashMap::new() }
    }

    pub fn user_wants_sh(&mut self, user_id: UserId) {
        self.user_data.insert(user_id, UserData { wants_sh: true });
    }

    pub fn num_users_wanting_sh(&self) -> u32 {
        self.user_data.values().fold(0, |acc, data| acc + (data.wants_sh as u32))
    }
}
