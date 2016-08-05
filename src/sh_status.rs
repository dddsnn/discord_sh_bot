use std::collections::{HashMap, HashSet};
use discord::model::{UserId, OnlineStatus};
use model::{Tier, StatusReport, UserData, Want, Timeframe};
use common::Retain;
use time;

pub struct ShStatus {
    users_data: HashMap<UserId, UserData>,
}

impl ShStatus {
    pub fn new() -> Self {
        ShStatus { users_data: HashMap::new() }
    }

    /// Returns new user data.
    pub fn set_user_wants_sh(&mut self,
                             user_id: UserId,
                             time: Timeframe,
                             wants: HashSet<Want>)
                             -> &UserData {
        let user_data = self.users_data.entry(user_id).or_insert(UserData {
            // TODO set user's actual online status (may theoretically be idle (?))
            status: OnlineStatus::Online,
            time_wants: HashMap::new(),
        });
        {
            let existing_wants = user_data.time_wants.entry(time).or_insert(HashSet::new());
            for want in wants {
                existing_wants.insert(want);
            }
        }
        user_data
    }

    pub fn set_user_doesnt_want_sh(&mut self, user_id: UserId) {
        if let Some(user_data) = self.users_data.get_mut(&user_id) {
            user_data.time_wants.clear();
        }
    }

    pub fn set_user_changed_status(&mut self, user_id: UserId, status: OnlineStatus) {
        let user_data = self.users_data.entry(user_id).or_insert(UserData {
            status: status,
            time_wants: HashMap::new(),
        });
        user_data.status = status;
        if status == OnlineStatus::Offline {
            // User is now offline, delete all wants that were only valid until he logged out.
            user_data.time_wants.retain(|t| t != &Timeframe::UntilLogout);
        }
    }

    pub fn get_current_status(&mut self) -> StatusReport {
        // Clean up the current user data, e.g. remove outdated wants.
        update_users_data(self.users_data.values_mut());
        let update = |mut acc: StatusReport, user_data: &UserData| {
            if !user_data.time_wants.is_empty() {
                acc.num_wanting_total += 1;
            }
            let (mut wants_t6, mut wants_t8, mut wants_t10) = (false, false, false);
            for datum in user_data.time_wants.values().flat_map(|s| s.iter()) {
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
        self.users_data
            .values()
            .filter(|ud| ud.status == OnlineStatus::Online)
            .fold(init_status, &update)
    }
}

fn update_users_data<'a, I: Iterator<Item = &'a mut UserData>>(data: I) {
    let now = time::now();
    for d in data {
        // Only retain timespan wants that are valid beyond now.
        d.time_wants.retain(|&t| {
            if let Timeframe::Timespan { until } = t {
                return until > now;
            }
            true
        });
    }
}
