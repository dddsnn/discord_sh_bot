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
    Request::Echo { echo_msg: "asdf".to_owned() }
}
