extern crate discord;
extern crate time;

mod discord_connection;
mod common;
mod sh_status;
mod message_parser;

use std::collections::HashSet;
use std::sync::mpsc;
use discord::model::{Event, Channel, CurrentUser, Message};
use discord_connection::{DiscordConnection, BotConnection};
use sh_status::{ShStatus, Tier, Want, Timeframe};
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

    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || listen_for_shutdown(sender));
    ShBot::new(&token, receiver).run();
}

fn listen_for_shutdown(shutdown_sender: mpsc::Sender<()>) {
    println!("Enter \"s\" or \"shutdown\" to shut down gracefully.");
    let mut buf = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buf)
            .unwrap_or_else(|err| {
                println!("Unable to read from stdin: {}", err);
                0
            });
        let input = buf.trim();
        if input == "s" || input == "shutdown" {
            break;
        }
    }
    shutdown_sender.send(()).expect("Unable to send shutdown to main loop.");
    println!("Sent the shutdown signal to the main thread. It will exit as soon as it receives \
              the next event. (Don't ask...)");
}

struct ShBot<D: DiscordConnection> {
    discord: D,
    me: CurrentUser,
    shutdown_receiver: mpsc::Receiver<()>,
    sh_status: ShStatus,
}

// TODO do i have to specify which kind of discordconnection?
impl ShBot<BotConnection> {
    fn new(token: &str, shutdown_receiver: mpsc::Receiver<()>) -> Self {
        let (d, me) = BotConnection::from_bot_token(token);
        //        let shutdown_received = shutdown_received.clone();
        ShBot {
            discord: d,
            me: me,
            shutdown_receiver: shutdown_receiver,
            sh_status: ShStatus::new(),
        }
    }

    fn run(mut self) {
        while let Err(mpsc::TryRecvError::Empty) = self.shutdown_receiver.try_recv() {
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
            Request::Echo { echo_msg } => self.handle_echo(msg, &echo_msg),
            Request::Help => self.handle_help(msg),
            Request::Want { wants } => self.handle_want(msg, wants),
            Request::DontWant => self.handle_dont_want(msg),
            Request::Status => self.handle_status(msg),
        }
    }

    fn handle_unknown(&self, msg: Message) {
        let reply = "\"".to_owned() + &msg.content +
                    "\" is not a valid request. Type \"help\" to find out what is.";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_echo(&self, msg: Message, echo_msg: &str) {
        let reply = msg.author.name + " wants me to echo \"" + echo_msg + "\".";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_help(&self, msg: Message) {
        // TODO
        let reply = "This is an unhelpful help text. There'll be a better one, I promise.";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_want(&mut self, msg: Message, wants: HashSet<Want>) {
        let ud = self.sh_status.set_user_wants_sh(msg.author.id, wants);
        // TODO maybe factor out forming the reply? this gets pretty long
        // TODO sort based on tier (tier 6 should always be first etc.) and group to compactify the
        // information
        let mut kind = String::new();
        for (i, want) in ud.wants.iter().enumerate() {
            match want.tier {
                Tier::Tier6 => kind.push_str("tier 6 "),
                Tier::Tier8 => kind.push_str("tier 8 "),
                Tier::Tier10 => kind.push_str("tier 10 "),
            }
            if i == 0 {
                // First in the list, add a Stronghold.
                kind.push_str(" Stronghold ");
            }
            match want.time {
                Timeframe::Always => kind.push_str("whenever you're online"),
                Timeframe::UntilLogout => kind.push_str("until you log out"),
                Timeframe::Timespan { until } => {
                    // TODO handle error on unwrap
                    let tm_fmt = until.strftime("{tm_hour}:{tm_minute}").unwrap();
                    kind.push_str(&format!("until {}", tm_fmt));
                }
            }
            if i + 2 < ud.wants.len() {
                // Before second-to-last one, add comma for enumeration.
                kind.push_str(", ");
            } else if i + 2 == ud.wants.len() {
                // Second-to-last one, add "and".
                kind.push_str(" and ");
            }
        }
        let reply = format!("Ok, I'll note you're up for {}.", kind);
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_dont_want(&mut self, msg: Message) {
        self.sh_status.set_user_doesnt_want_sh(msg.author.id);
        let reply = "Ok, I'll take you off the list.";
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_status(&mut self, msg: Message) {
        let status_report = self.sh_status.get_current_status();
        // TODO special case one player (is/are)
        // TODO better solution for multiline strings?
        let reply = format!("There is currently a total of {} players who want to play Stronghold.
{} want tier 6, {} tier 8 and {} tier 10.",
                            status_report.num_wanting_total,
                            status_report.num_wanting_t6,
                            status_report.num_wanting_t8,
                            status_report.num_wanting_t10);
        if let Err(msg) = self.discord
            .send_message(&msg.channel_id, &reply, false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}
