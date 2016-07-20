use std::collections::HashSet;
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
    let mut wants = HashSet::new();
    if let None = tokens.rest() {
        // No tiers specified, assume all are OK until next logout.
        // TODO clone from a default value
        wants.insert(Want {
            tier: Tier::Tier6,
            time: Timeframe::UntilLogout,
        });
        wants.insert(Want {
            tier: Tier::Tier8,
            time: Timeframe::UntilLogout,
        });
        wants.insert(Want {
            tier: Tier::Tier10,
            time: Timeframe::UntilLogout,
        });
    }
    loop {
        match tokens.next() {
            None => return Request::Want { wants: wants },
            Some("6") => {
                wants.insert(Want {
                    tier: Tier::Tier6,
                    time: Timeframe::UntilLogout,
                });
            }
            Some("8") => {
                wants.insert(Want {
                    tier: Tier::Tier8,
                    time: Timeframe::UntilLogout,
                });
            }
            Some("10") => {
                wants.insert(Want {
                    tier: Tier::Tier10,
                    time: Timeframe::UntilLogout,
                });
            }
            Some(_) => {
                // TODO parse timeframe
                // TODO invalid command instead of unknown: started out right (with want), but
                // didn't specify tier correctly
                return Request::Unknown;
            }
        }
    }
}
