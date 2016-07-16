use discord::model::Message;
use common::SplitWhitespaceWithRest;

pub enum Request {
    None,
    Unknown,
    Shutdown,
    Echo {
        echo_msg: String,
    },
    Help,
    Want,
    Status,
}

pub fn parse_message(msg: &Message) -> Request {
    let mut tokens = SplitWhitespaceWithRest::new(&msg.content);
    //    let previous = Vec::new();
    loop {
        // TODO use matching here once slice matching becomes stable (don't want to use nightly}
        //        if previous == vec![] {
        match tokens.next() {
            // TODO any way to get rid of the returns?
            None => return Request::None,
            Some(token) => {
                match &*token.to_lowercase() {
                    "" => return Request::None,
                    "shutdown" => return Request::Shutdown,
                    "echo" => {
                        return Request::Echo { echo_msg: tokens.rest().unwrap_or("").to_owned() }
                    }
                    "help" => return Request::Help,
                    "want" => return Request::Want,
                    "status" => return Request::Status,
                    _ => return Request::Unknown,
                }
            }
        }
        //        } else {
        //            break;
        //        }
    }
}
