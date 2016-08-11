use std::collections::{HashSet, HashMap};
use std::error::Error;
use time;
use discord::model::OnlineStatus;
use rustc_serialize::{Encodable, Encoder, Decodable, Decoder};

pub enum Request {
    None,
    Unknown,
    Help,
    Want {
        time: Timeframe,
        wants: HashSet<Want>,
    },
    DontWant,
    Status,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Tier {
    Tier6,
    Tier8,
    Tier10,
}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum Timeframe {
    Always,
    UntilLogout,
    Timespan {
        until: time::Tm,
    },
}

pub struct StatusReport {
    pub num_wanting_total: usize,
    pub num_wanting_t6: usize,
    pub num_wanting_t8: usize,
    pub num_wanting_t10: usize,
}

#[derive(PartialEq, Clone, Debug)]
pub struct UserData {
    pub status: OnlineStatus,
    pub time_wants: HashMap<Timeframe, HashSet<Want>>,
}


#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Want {
    pub tier: Tier,
}

impl Encodable for UserData {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(2, |s| {
            try!(s.emit_seq_elt(0, |s| {
                // Encode status.
                s.emit_enum("OnlineStatus", |s| {
                    match self.status {
                        OnlineStatus::Offline => s.emit_enum_variant("Offline", 0, 0, |_| Ok(())),
                        OnlineStatus::Online => s.emit_enum_variant("Online", 1, 0, |_| Ok(())),
                        OnlineStatus::Idle => s.emit_enum_variant("Idle", 2, 0, |_| Ok(())),
                    }
                })
            }));
            s.emit_seq_elt(1, |s| {
                // Encode map from timeframes to sets of wants.
                s.emit_map(self.time_wants.len(), |s| {
                    for (i, (k, ref v)) in self.time_wants.iter().enumerate() {
                        try!(s.emit_map_elt_key(i, |s| k.encode(s)));
                        try!(s.emit_map_elt_val(i, |s| {
                            // Encode set of wants.
                            s.emit_seq(v.len(), |s| {
                                for (j, w) in v.iter().enumerate() {
                                    try!(s.emit_seq_elt(j, |s| w.encode(s)));
                                }
                                Ok(())
                            })
                        }));
                    }
                    Ok(())
                })
            })
        })
    }
}

impl Decodable for UserData {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_seq(|d, _| {
            let status = try!(d.read_seq_elt(0, |d| {
                d.read_enum("OnlineStatus", |d| {
                    d.read_enum_variant(&["Offline", "Online", "Idle"], |_, i| {
                        match i {
                            0 => Ok(OnlineStatus::Offline),
                            1 => Ok(OnlineStatus::Online),
                            _ => Ok(OnlineStatus::Idle),
                        }
                    })
                })
            }));
            let time_wants = try!(d.read_seq_elt(1, |d| {
                d.read_map(|d, len| {
                    let mut time_wants = HashMap::with_capacity(len);
                    for i in 0..len {
                        let time = try!(d.read_map_elt_key(i, |d| Timeframe::decode(d)));
                        let wants = try!(d.read_map_elt_val(i, |d| {
                            d.read_seq(|d, len_set| {
                                let mut wants = HashSet::with_capacity(len_set);
                                for j in 0..len_set {
                                    let want = try!(d.read_seq_elt(j, |d| Want::decode(d)));
                                    wants.insert(want);
                                }
                                Ok(wants)
                            })
                        }));
                        time_wants.insert(time, wants);
                    }
                    Ok(time_wants)
                })
            }));
            Ok(UserData {
                status: status,
                time_wants: time_wants,
            })
        })
    }
}

impl Encodable for Timeframe {
    // We have to encode the timeframe as a string so we can use it as a key in a map (json...).
    // First the type of timeframe. Then, if it's a timespan, the seconds and nanoseconds of the
    // timespec, all separated by colons.
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            Timeframe::Timespan { until } => {
                let timespec = until.to_timespec();
                s.emit_str(&format!("Timespan:{}:{}", timespec.sec, timespec.nsec))
            }
            Timeframe::Always => s.emit_str("Always"),
            Timeframe::UntilLogout => s.emit_str("UntilLogout"),
        }
    }
}

impl Decodable for Timeframe {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        let s = try!(d.read_str());
        let mut split = s.splitn(3, ':');
        match split.next() {
            Some("Timespan") => {
                let sec = try!(if let Some(sec_str) = split.next() {
                    sec_str.parse::<i64>().map_err(|e| {
                        d.error(&format!("Error parsing seconds: {}.", e.description()))
                    })
                } else {
                    return Err(d.error("Timespan contained no seconds."));
                });
                let nsec = try!(if let Some(nsec_str) = split.next() {
                    nsec_str.parse::<i32>().map_err(|e| {
                        d.error(&format!("Error parsing nanoseconds: {}.", e.description()))
                    })
                } else {
                    return Err(d.error("Timespan contained no seconds."));
                });
                let timespec = time::Timespec::new(sec, nsec);
                let tm = time::at_utc(timespec);
                Ok(Timeframe::Timespan { until: tm })
            }
            Some("Always") => Ok(Timeframe::Always),
            Some("UntilLogout") => Ok(Timeframe::UntilLogout),
            _ => Err(d.error("Unknown timeframe type.")),
        }
    }
}

impl Encodable for Want {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.tier.encode(s)
    }
}

impl Decodable for Want {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        Tier::decode(d).map(|tier| Want { tier: tier })
    }
}

impl Encodable for Tier {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_u32(match *self {
            Tier::Tier6 => 6,
            Tier::Tier8 => 8,
            Tier::Tier10 => 10,
        })
    }
}


impl Decodable for Tier {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_u32()
            .and_then(|i| match i {
                6 => Ok(Tier::Tier6),
                8 => Ok(Tier::Tier8),
                10 => Ok(Tier::Tier10),
                other => Err(d.error(&format!("Expected tier 6, 8 or 10, got {}.", other))),
            })
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::{Tier, Want, Timeframe, UserData};
    use discord::model::OnlineStatus;
    use std::collections::{HashMap, HashSet};
    use rustc_serialize::json::{encode, decode};
    use time;

    #[test]
    fn tier() {
        let tiers = vec![Tier::Tier6, Tier::Tier8, Tier::Tier10];
        for tier in tiers {
            let encoded = encode(&tier).unwrap();
            let decoded = decode::<Tier>(&encoded).unwrap();
            assert_eq!(tier, decoded);
        }
    }

    #[test]
    fn want() {
        let tiers = vec![Tier::Tier6, Tier::Tier8, Tier::Tier10];
        for tier in tiers {
            let want = Want { tier: tier };
            let encoded = encode(&want).unwrap();
            let decoded = decode::<Want>(&encoded).unwrap();
            assert_eq!(want, decoded);
        }
    }

    #[test]
    fn timeframe() {
        let until1 = time::at(time::Timespec::new(0, 0));
        let until2 = time::at(time::Timespec::new(12345678, 2345));
        let timeframes = vec![Timeframe::Always,
                              Timeframe::UntilLogout,
                              Timeframe::Timespan { until: until1 },
                              Timeframe::Timespan { until: until2 }];
        for timeframe in timeframes {
            let encoded = encode(&timeframe).unwrap();
            let decoded = decode::<Timeframe>(&encoded).unwrap();
            assert_eq!(timeframe, decoded);
        }
    }

    #[test]
    fn userdata() {
        let statuses = vec![OnlineStatus::Online, OnlineStatus::Offline, OnlineStatus::Idle];
        let time_wants = {
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
        for status in statuses {
            let user_data = UserData {
                status: status,
                time_wants: time_wants.clone(),
            };
            let encoded = encode(&user_data).unwrap();
            let decoded = decode::<UserData>(&encoded).unwrap();
            assert_eq!(user_data, decoded);
        }
    }
}
