extern crate discord;

use discord::model::Event;
use discord::model::ChannelId;
use discord::model::CurrentUser;
use discord::model::Message;

pub trait DiscordConnection {
    fn recv_event(&mut self) -> Result<Event, &str>;
    fn send_message(&self,
                    channel: &ChannelId,
                    text: &str,
                    nonce: &str,
                    tts: bool)
                    -> Result<Message, &str>;
}

pub struct BotConnection {
    discord: discord::Discord,
    conn: discord::Connection,
}

impl BotConnection {
    pub fn from_bot_token(token: &str) -> (Self, CurrentUser) {
        let d = discord::Discord::from_bot_token(&token).expect("error logging in");
        println!("logged in");

        let (c, ready_event) = d.connect().expect("failed connect");
        let me = ready_event.user;
        println!("connected");
        (BotConnection {
            discord: d,
            conn: c,
        },
         me)
    }
}


impl DiscordConnection for BotConnection {
    fn recv_event(&mut self) -> Result<Event, &str> {
        match self.conn.recv_event() {
            // TODO error handling
            Err(_) => Err("error"),
            Ok(e) => Ok(e),
        }
    }

    fn send_message(&self,
                    channel: &ChannelId,
                    text: &str,
                    nonce: &str,
                    tts: bool)
                    -> Result<Message, &str> {
        // TODO error handling
        match self.discord.send_message(channel, text, nonce, tts) {
            Err(_) => Err("error"),
            Ok(msg) => Ok(msg),
        }
    }
}
