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

    /// Returns whether the user wanted Stronghold before.
    pub fn user_doesnt_want_sh(&mut self, user_id: UserId) -> bool {
        let previous_status = self.user_data.insert(user_id, UserData { wants_sh: false });
        if let Some(UserData { wants_sh: true }) = previous_status {
            true
        } else {
            false
        }
    }

    pub fn num_users_wanting_sh(&self) -> u32 {
        self.user_data.values().fold(0, |acc, data| acc + (data.wants_sh as u32))
    }
}
