use std::collections::HashMap;
use discord::model::UserId;

pub struct ShStatus {
    user_data: HashMap<UserId, UserData>,
}

pub struct StatusReport {
    pub num_wanting_total: usize,
    pub num_wanting_t6: usize,
    pub num_wanting_t8: usize,
    pub num_wanting_t10: usize,
}

pub struct UserData {
    pub wants_t6: bool,
    pub wants_t8: bool,
    pub wants_t10: bool,
}

impl ShStatus {
    pub fn new() -> Self {
        ShStatus { user_data: HashMap::new() }
    }

    /// Returns new user data.
    pub fn add_user_wants_sh(&mut self,
                             user_id: UserId,
                             t6: bool,
                             t8: bool,
                             t10: bool)
                             -> UserData {
        let user_data = self.user_data.entry(user_id).or_insert(UserData {
            wants_t6: false,
            wants_t8: false,
            wants_t10: false,
        });
        user_data.wants_t6 |= t6;
        user_data.wants_t8 |= t8;
        user_data.wants_t10 |= t10;
        UserData {
            wants_t6: user_data.wants_t6,
            wants_t8: user_data.wants_t8,
            wants_t10: user_data.wants_t10,
        }
    }

    pub fn add_user_doesnt_want_sh(&mut self, user_id: UserId) {
        self.user_data.insert(user_id,
                              UserData {
                                  wants_t6: false,
                                  wants_t8: false,
                                  wants_t10: false,
                              });
    }

    pub fn get_current_status(&self) -> StatusReport {
        let update = |mut acc: StatusReport, user_data: &UserData| {
            acc.num_wanting_total +=
                (user_data.wants_t6 || user_data.wants_t8 || user_data.wants_t10) as usize;
            acc.num_wanting_t6 += user_data.wants_t6 as usize;
            acc.num_wanting_t8 += user_data.wants_t8 as usize;
            acc.num_wanting_t10 += user_data.wants_t10 as usize;
            acc
        };
        let init_status = StatusReport {
            num_wanting_total: 0,
            num_wanting_t6: 0,
            num_wanting_t8: 0,
            num_wanting_t10: 0,
        };
        self.user_data.values().fold(init_status, &update)
    }
}
