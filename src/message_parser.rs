use discord::model::Message;
use common::SplitWhitespaceWithRest;

pub enum Request {
    None,
    Unknown,
    Echo {
        echo_msg: String,
    },
    Help,
    Want,
    DontWant,
    Status,
}

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
                        "echo" => {
                            return Request::Echo {
                                echo_msg: tokens.rest().unwrap_or("").to_owned(),
                            }
                        }
                        "help" => return Request::Help,
                        "want" => return Request::Want,
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
