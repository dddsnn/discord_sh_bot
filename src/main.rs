extern crate discord;

mod discord_connection;
mod common;
mod sh_status;
mod message_parser;

use discord::model::{Event, Channel, ChannelId, CurrentUser, User, Message};
use discord_connection::{DiscordConnection, BotConnection};
use sh_status::ShStatus;
use message_parser::Request;

const BOT_COMMAND: &'static str = ".sh";

fn main() {
    let token: String;
    if let Some(t) = std::env::args().nth(1) {
        token = t;
    } else {
        // TODO log, don't print
        println!("Pass the bot token as an argument.");
        std::process::exit(1);
    }

    ShBot::new(&token).run();
}

struct ShBot<D: DiscordConnection> {
    discord: D,
    me: CurrentUser,
    running: bool,
    sh_status: ShStatus,
}

// TODO do i have to specify which kind of discordconnection?
impl ShBot<BotConnection> {
    fn new(token: &str) -> Self {
        let (d, me) = BotConnection::from_bot_token(token);
        ShBot {
            discord: d,
            me: me,
            running: true,
            sh_status: ShStatus::new(),
        }
    }

    fn run(mut self) {
        while self.running {
            self.handle_event();
        }
        self.discord.shutdown();
    }

    fn handle_event(&mut self) {
        match self.discord.recv_event() {
            Err(msg) => {
                // TODO log, don't print
                println!("Error receiving event: {}", msg);
            }
            Ok(Event::MessageCreate(mut msg)) => {
                match self.message_concerns_me(msg) {
                    Ok((false, _)) => {
                        // Message not directed at the bot.
                        return;
                    }
                    Ok((true, new_msg)) => msg = new_msg,
                    Err(msg) => {
                        // TODO log, don't print
                        println!("Error getting channel information: {}", msg);
                        return;
                    }
                }
                self.handle_message(msg);
            }
            _ => {
                // Event we don't care about.
            }
        }
    }

    fn message_concerns_me(&self, mut msg: Message) -> Result<(bool, Message), String> {
        if msg.author.id == self.me.id {
            // Don't respond to own messages.
            return Ok((false, msg));
        }
        // Get info about the channel the message arrived at.
        // TODO cache
        match self.discord.get_channel(msg.channel_id) {
            Ok(channel) => {
                if let Channel::Public(_) = channel {
                    // Public channel, only handle if it was addressed at the bot (i.e.
                    // prefixed with the bot command).
                    let (first, second) = common::str_head_tail(&msg.content);
                    if first != BOT_COMMAND {
                        // Command doesn't start with bot command, ignore.
                        return Ok((false, msg));
                    }
                    // Handle message, but remove bot command from the beginning.
                    msg.content = second;
                    Ok((true, msg))
                } else {
                    // Private channel, handle.
                    Ok((true, msg))
                }
            }
            Err(msg) => return Err(msg),
        }
    }

    fn handle_message(&mut self, msg: Message) {
        let req = message_parser::parse_message(&msg);
        match req {
            Request::None => {}
            Request::Unknown => self.handle_unknown(msg),
            Request::Shutdown => self.handle_shutdown(msg),
            Request::Echo { echo_msg } => self.handle_echo(msg, &echo_msg),
            Request::Help => self.handle_help(msg),
            Request::Want => self.handle_want(msg),
            Request::Status => self.handle_status(msg),
        }
    }

    fn handle_unknown(&self, msg: Message) {
        let reply = "\"".to_owned() + &msg.content +
                    "\" is not a valid request. Type \"help\" to find out what is.";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_shutdown(&mut self, msg: Message) {
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, "Shutting down. Bye now.", "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
        self.running = false;
    }

    // TODO factor out request handling
    fn handle_echo(&self, msg: Message, echo_msg: &str) {
        let reply = msg.author.name + " wants me to echo \"" + echo_msg + "\".";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_help(&self, msg: Message) {
        // TODO
        let reply = "This is an unhelpful help text. There'll be a better one, I promise.";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_want(&mut self, msg: Message) {
        self.sh_status.user_wants_sh(msg.author.id);
        let reply = "Ok, I'll put you on the list.";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_status(&mut self, msg: Message) {
        let num_wanting = self.sh_status.num_users_wanting_sh();
        // TODO special case one player (is/are)
        let reply = format!("There are currently {} players who want to play Stronghold.",
                            num_wanting);
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}
