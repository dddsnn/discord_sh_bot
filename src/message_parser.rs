use discord::model::Message;

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
    let mut tokens = msg.content.split_whitespace().map(str::to_lowercase);
    //    let previous = Vec::new();
    loop {
        // TODO use matching here once slice matching becomes stable (don't want to use nighyly}
        //        if previous == vec![] {
        match tokens.next() {
            None => return Request::None,
            Some(token) => {
                match &*token {
                    "" => return Request::None,
                    "shutdown" => return Request::Shutdown,
                    // TODO add actual message
                    "echo" => {
                        return Request::Echo { echo_msg: tokens.next().unwrap_or("".to_owned()) }
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
