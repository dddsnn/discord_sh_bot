use std::collections::{HashMap, HashSet};
use discord::model::UserId;
use time;

#[derive(Eq, PartialEq, Hash)]
pub enum Tier {
    Tier6,
    Tier8,
    Tier10,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum Timeframe {
    Always,
    UntilLogout,
    Timespan {
        until: time::Tm,
    },
}

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
    pub wants: HashSet<Want>,
}

#[derive(Eq, PartialEq, Hash)]
pub struct Want {
    pub tier: Tier,
    pub time: Timeframe,
}

// impl Eq for UserDatum{}

impl ShStatus {
    pub fn new() -> Self {
        ShStatus { user_data: HashMap::new() }
    }

    /// Returns new user data.
    pub fn set_user_wants_sh(&mut self, user_id: UserId, wants: HashSet<Want>) -> &UserData {
        let user_data = self.user_data.entry(user_id).or_insert(UserData { wants: HashSet::new() });
        for want in wants {
            user_data.wants.insert(want);
        }
        user_data
    }

    pub fn set_user_doesnt_want_sh(&mut self, user_id: UserId) {
        if let Some(user_data) = self.user_data.get_mut(&user_id) {
            user_data.wants.clear();
        }
    }

    pub fn get_current_status(&self) -> StatusReport {
        let update = |mut acc: StatusReport, user_data: &UserData| {
            if !user_data.wants.is_empty() {
                acc.num_wanting_total += 1;
            }
            let (mut wants_t6, mut wants_t8, mut wants_t10) = (false, false, false);
            for datum in user_data.wants.iter() {
                match datum.tier {
                    Tier::Tier6 => wants_t6 = true,
                    Tier::Tier8 => wants_t8 = true,
                    Tier::Tier10 => wants_t10 = true,
                }
            }
            acc.num_wanting_t6 += wants_t6 as usize;
            acc.num_wanting_t8 += wants_t8 as usize;
            acc.num_wanting_t10 += wants_t10 as usize;
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
