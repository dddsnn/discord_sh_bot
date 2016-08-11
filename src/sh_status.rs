use std::collections::{HashMap, HashSet};
use discord::model::{UserId, OnlineStatus};
use model::{Tier, StatusReport, UserData, Want, Timeframe};
use common::Retain;
use time;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

#[derive(PartialEq, Debug)]
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

const SERIALIZATION_VERSION: u32 = 1;

impl Encodable for ShStatus {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(2, |s| {
            try!(s.emit_seq_elt(0, |s| s.emit_u32(SERIALIZATION_VERSION)));
            s.emit_seq_elt(1, |s| {
                s.emit_map(self.users_data.len(), |s| {
                    for (i, (k, ref v)) in self.users_data.iter().enumerate() {
                        try!(s.emit_map_elt_key(i, |s| {
                            let UserId(id) = *k;
                            s.emit_u64(id)
                        }));
                        try!(s.emit_map_elt_val(i, |s| v.encode(s)));
                    }
                    Ok(())
                })
            })
        })
    }
}

impl Decodable for ShStatus {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_seq(|d, _| {
            let version = try!(d.read_seq_elt(0, |d| d.read_u32()));
            if version != SERIALIZATION_VERSION {
                return Err(d.error(&format!("Invalid serialization version {}.", version)));
            }
            let users_data = try!(d.read_seq_elt(1, |d| {
                d.read_map(|d, len| {
                    let mut users_data = HashMap::new();
                    for i in 0..len {
                        let user_id =
                            try!(d.read_map_elt_key(i, |d| Ok(UserId(try!(d.read_u64())))));
                        let user_data =
                            try!(d.read_map_elt_val(i, |d| Ok(try!(UserData::decode(d)))));
                        users_data.insert(user_id, user_data);
                    }
                    Ok(users_data)
                })
            }));
            Ok(ShStatus { users_data: users_data })
        })
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::ShStatus;
    use model::{UserData, Want, Timeframe, Tier};
    use discord::model::{UserId, OnlineStatus};
    use std::collections::{HashMap, HashSet};
    use rustc_serialize::json::{encode, decode};
    use time;

    #[test]
    fn sh_status_empty() {
        let sh_status = ShStatus { users_data: HashMap::new() };
        let encoded = encode(&sh_status).unwrap();
        let decoded = decode::<ShStatus>(&encoded).unwrap();
        assert_eq!(sh_status, decoded);
    }

    #[test]
    fn sh_status_simple() {
        let empty_user_data = UserData {
            status: OnlineStatus::Online,
            time_wants: HashMap::new(),
        };
        let sh_status = ShStatus {
            users_data: {
                let mut users_data = HashMap::new();
                users_data.insert(UserId(0), empty_user_data);
                users_data
            },
        };
        let encoded = encode(&sh_status).unwrap();
        let decoded = decode::<ShStatus>(&encoded).unwrap();
        assert_eq!(sh_status, decoded);
    }

    #[test]
    fn sh_status_complex() {
        let user_ids = vec![UserId(0), UserId(1), UserId(1357)];
        let statuses = vec![OnlineStatus::Online, OnlineStatus::Offline, OnlineStatus::Idle];
        let time_wants2 = {
            let mut time_wants = HashMap::new();
            let wants = HashSet::new();
            let until = time::at(time::Timespec::new(12345678, 2345));
            time_wants.insert(Timeframe::Timespan { until: until }, wants);
            time_wants
        };
        let time_wants3 = {
            let mut time_wants = HashMap::new();
            let wants1 = HashSet::new();
            let mut wants2_vec = vec![Want { tier: Tier::Tier8 }];
            let wants2 = wants2_vec.drain(..).collect::<HashSet<Want>>();
            let mut wants3_vec = vec![Want { tier: Tier::Tier10 }, Want { tier: Tier::Tier6 }];
            let wants3 = wants3_vec.drain(..).collect::<HashSet<Want>>();
            let until = time::at(time::Timespec::new(12345678, 2345));
            time_wants.insert(Timeframe::Always, wants1);
            time_wants.insert(Timeframe::UntilLogout, wants2);
            time_wants.insert(Timeframe::Timespan { until: until }, wants3);
            time_wants
        };
        let time_wantss = vec![HashMap::new(), time_wants2, time_wants3];

        let sh_status = {
            let mut users_data = HashMap::new();
            for ((user_id, status), time_wants) in user_ids.iter().zip(statuses).zip(time_wantss) {
                let user_data = UserData {
                    status: status,
                    time_wants: time_wants,
                };
                users_data.insert(*user_id, user_data);
            }
            ShStatus { users_data: users_data }
        };
        let encoded = encode(&sh_status).unwrap();
        let decoded = decode::<ShStatus>(&encoded).unwrap();
        assert_eq!(sh_status, decoded);
    }
}
