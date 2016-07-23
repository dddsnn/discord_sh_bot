use std::collections::HashSet;
use time;

pub enum Request {
    None,
    Unknown,
    Help,
    Want {
        wants: HashSet<Want>,
    },
    DontWant,
    Status,
}

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
