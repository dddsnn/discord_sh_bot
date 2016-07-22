use std::collections::HashSet;
use time;
use time::Duration;
use discord::model::Message;
use common::SplitWhitespaceWithRest;
use sh_status::{Want, Timeframe, Tier};

pub enum Request {
    None,
    Unknown,
    Echo {
        echo_msg: String,
    },
    Help,
    Want {
        wants: HashSet<Want>,
    },
    DontWant,
    Status,
}

// TODO unhardcode command strings
pub fn parse_message(msg: &Message) -> Request {
    let mut tokens = SplitWhitespaceWithRest::new(&msg.content);
    let mut previous: Vec<String> = Vec::new();
    loop {
        // TODO use matching here once slice matching becomes stable (don't want to use nightly}
        if previous == vec![] as Vec<String> {
            match tokens.next() {
                None => return Request::None,
                Some(token) => {
                    match &*token.to_lowercase() {
                        "" => return Request::None,
                        "echo" => return parse_echo(tokens),
                        "help" => return Request::Help,
                        "want" => return parse_want(tokens),
                        "status" => return Request::Status,
                        "dont" | "don't" => previous.push("dont".to_owned()),
                        _ => return Request::Unknown,
                    }
                }
            }
        } else if previous == vec!["dont"] {
            match tokens.next() {
                None => return Request::Unknown,
                Some(token) => {
                    match &*token.to_lowercase() {
                        "want" => return Request::DontWant,
                        _ => return Request::Unknown,
                    }
                }
            }
        } else {
            return Request::Unknown;
        }
    }
}

fn parse_echo(tokens: SplitWhitespaceWithRest) -> Request {
    Request::Echo { echo_msg: tokens.rest().unwrap_or("").to_owned() }
}

fn parse_want(mut tokens: SplitWhitespaceWithRest) -> Request {
    let mut tiers = HashSet::new();
    loop {
        match tokens.next() {
            Some("6") => {
                tiers.insert(Tier::Tier6);
            }
            Some("8") => {
                tiers.insert(Tier::Tier8);
            }
            Some("10") => {
                tiers.insert(Tier::Tier10);
            }
            t @ None | t @ Some(_) => {
                if tiers.is_empty() {
                    // No tiers are specified (either because nothing is specified at all, or only a
                    // timeframe is specified), assume all are OK.
                    tiers.insert(Tier::Tier6);
                    tiers.insert(Tier::Tier8);
                    tiers.insert(Tier::Tier10);
                }
                if let Some(_) = t {
                    // There are more tokens, but not any specifying tiers. Rewind so they can be
                    // parsed as timeframe.
                    tokens.rewind();
                }
                return parse_want_timeframe(tokens, tiers);
            }
        }
    }
}

fn parse_want_timeframe(mut tokens: SplitWhitespaceWithRest, mut tiers: HashSet<Tier>) -> Request {
    let timeframe = match tokens.next() {
        None => Timeframe::UntilLogout,
        Some("always") => Timeframe::Always,
        Some("until") => {
            if let Some("logout") = tokens.next() {
                Timeframe::UntilLogout
            } else {
                // TODO invalid command instead of unknown: started out right (with want and tiers),
                // but didn't specify timeframe correctly
                return Request::Unknown;
            }
        }
        Some("for") => {
            if let Some(time_str) = tokens.next() {
                if let Ok(duration) = parse_duration(time_str) {
                    Timeframe::Timespan { until: time::now_utc() + duration }
                } else {
                    // TODO invalid command instead of unknown: started out right (with want and
                    // tiers), but didn't specify timeframe correctly
                    return Request::Unknown;
                }
            } else {
                // TODO invalid command instead of unknown: started out right (with want and tiers),
                // but didn't specify timeframe correctly
                return Request::Unknown;
            }
        }
        Some(_) => {
            // TODO invalid command instead of unknown: started out right (with want and tiers),
            // but didn't specify timeframe correctly
            return Request::Unknown;
        }

    };
    let wants = tiers.drain()
        .map(|tier| {
            Want {
                tier: tier,
                time: timeframe,
            }
        })
        .collect();
    Request::Want { wants: wants }
}

/// Parses format ("{}:{}h", hours, minutes)
fn parse_duration(hours_mins_str: &str) -> Result<Duration, String> {
    let mut split = hours_mins_str.split(":");
    let hours = match split.next() {
        None => return Err("No hours given.".to_owned()),
        Some(hours_str) => {
            if let Ok(hours) = hours_str.parse::<i64>() {
                hours
            } else {
                return Err("Hours are not a positive integer.".to_owned());
            }
        }
    };
    let minutes = match split.next() {
        None => return Err("No minutes given.".to_owned()),
        Some(mins_str) => {
            let last_char_idx = match mins_str.char_indices().rev().next() {
                Some((last_char_idx, last_char)) => {
                    if last_char != 'h' {
                        return Err("Duration doesn't end with \"h\".".to_owned());
                    }
                    last_char_idx
                }
                None => {
                    // Shouldn't happen, we've already checked the string isn't empty
                    return Err("No minutes given.".to_owned());
                }
            };
            if let Ok(mins) = mins_str[..last_char_idx].parse::<i64>() {
                mins
            } else {
                return Err("Minutes are not a positive integer.".to_owned());
            }
        }
    };
    if hours < 0 || minutes < 0 {
        return Err("Negative duration given.".to_owned());
    }
    if minutes > 59 {
        return Err("Too many minutes given.".to_owned());
    }
    // Total number of milliseconds in a Tm must not exceed i64::max_value().
    if i64::max_value() / (3600 * 1000) < hours + 1 {
        return Err("Given duration is too large.".to_owned());
    }
    let total_minutes = hours * 60 + minutes;
    Ok(Duration::minutes(total_minutes))
}
