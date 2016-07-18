use discord::model::Message;
use common::SplitWhitespaceWithRest;

pub enum Request {
    None,
    Unknown,
    Echo {
        echo_msg: String,
    },
    Help,
    Want {
        t6: bool,
        t8: bool,
        t10: bool,
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
    if let None = tokens.rest() {
        // No tiers specified, assume all are OK.
        return Request::Want {
            t6: true,
            t8: true,
            t10: true,
        };
    }
    let (mut t6, mut t8, mut t10) = (false, false, false);
    loop {
        match tokens.next() {
            None => {
                return Request::Want {
                    t6: t6,
                    t8: t8,
                    t10: t10,
                };
            }
            Some("6") => t6 = true,
            Some("8") => t8 = true,
            Some("10") => t10 = true,
            Some(_) => {
                // TODO invalid command instead of unknown: started out right (with want), but
                // didn't specify tier correctly
                return Request::Unknown;
            }
        }
    }
}
