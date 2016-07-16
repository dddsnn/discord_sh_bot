extern crate discord;

use std;
use discord::model::{Event, ChannelId, CurrentUser, Message, Channel};

const MAX_RETRIES: u32 = 5;

pub trait DiscordConnection {
    fn recv_event(&mut self) -> Result<Event, String>;
    fn send_message(&self, channel: &ChannelId, text: &str, tts: bool) -> Result<Message, String>;
    fn get_channel(&self, channel: ChannelId) -> Result<Channel, String>;
    fn shutdown(self);
}

pub struct BotConnection {
    discord: discord::Discord,
    conn: discord::Connection,
}

impl BotConnection {
    pub fn from_bot_token(token: &str) -> (Self, CurrentUser) {
        let d = match discord::Discord::from_bot_token(&token) {
            Ok(d) => d,
            Err(err) => {
                // TODO log, don't print
                println!("Error logging in: {}", err);
                std::process::exit(1);
            }
        };

        let (c, ready_event) = match d.connect() {
            Ok((c, re)) => (c, re),
            Err(err) => {
                // TODO log, don't print
                println!("Error connecting: {}", err);
                std::process::exit(1);
            }
        };
        let me = ready_event.user;
        (BotConnection {
            discord: d,
            conn: c,
        },
         me)
    }

    fn retry<R>(f: &mut FnMut() -> Result<R, discord::Error>) -> Result<R, String> {
        Self::retry_n(f, MAX_RETRIES, "Maximum number of retries exceeded.")
    }

    fn retry_n<R>(f: &mut FnMut() -> Result<R, discord::Error>,
                  tries: u32,
                  last_err_msg: &str)
                  -> Result<R, String> {
        if tries == 0 {
            return Err(last_err_msg.to_owned());
        }
        match f() {
            Ok(r) => Ok(r),
            Err(err) => {
                match err {
                    discord::Error::RateLimited(millis) => {
                        // Rate limited, sleep the prescribed amount of ms. Don't decrese the number
                        // of tries because this can always be fixed.
                        std::thread::sleep(std::time::Duration::from_millis(millis));
                        Self::retry_n(f, tries, last_err_msg)
                    }
                    _ => {
                        // Some other error, wait a second and hope the cause goes away.
                        // TODO maybe handle other specific cases?
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        Self::retry_n(f, tries - 1, &format!("{}", err))
                    }
                }
            }
        }
    }
}

impl DiscordConnection for BotConnection {
    /// Returns an error message on error.
    fn recv_event(&mut self) -> Result<Event, String> {
        // TODO do i have to use a move closure? no way to simply pass a function?
        Self::retry(&mut move || self.conn.recv_event())
    }

    /// Returns an error message on error.
    fn send_message(&self, channel: &ChannelId, text: &str, tts: bool) -> Result<Message, String> {
        Self::retry(&mut move || self.discord.send_message(channel, text, "", tts))
    }

    fn get_channel(&self, channel: ChannelId) -> Result<Channel, String> {
        Self::retry(&mut move || self.discord.get_channel(channel))
    }

    fn shutdown(self) {
        if let Err(err) = self.conn.shutdown() {
            // TODO log, don't print
            println!("Error shutting down the connection: {}", err);
        }
    }
}
