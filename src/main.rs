extern crate discord;

mod discord_connection;
mod common;
mod sh_status;
mod request_handlers;

use std::collections::HashMap;
use request_handlers::*;
use sh_status::ShStatus;
use discord_connection::{DiscordConnection, BotConnection};
use discord::model::{Event, Channel, ChannelId, CurrentUser, User, Message};

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

struct ShBot<'a, D: DiscordConnection> {
    discord: D,
    me: CurrentUser,
    running: bool,
    sh_status: ShStatus,
    request_handlers: HashMap<&'a str, &'a RequestHandler>,
}

// TODO do i have to specify which kind of discordconnection?
impl<'a> ShBot<'a, BotConnection> {
    fn new(token: &str) -> Self {
        let (d, me) = BotConnection::from_bot_token(token);
        let mut request_handlers:HashMap<&'a str,&'a RequestHandler> = HashMap::new();
        let eh=EchoHandler;
        request_handlers.insert("echo", &eh);
//        request_handlers.insert("help", HelpHandler);
//        request_handlers.insert("want", WantHandler);
//        request_handlers.insert("status", StatusHandler);
        ShBot {
            discord: d,
            me: me,
            running: true,
            sh_status: ShStatus::new(),
            request_handlers: HashMap::new(),
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
                self.handle_request(msg.channel_id, msg.author, &msg.content);
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

    fn handle_request(&mut self, channel_id: ChannelId, author: User, content: &str) {
        // Split at the first whitespace into command and options.
        let (command, options) = common::str_head_tail(&content);
        if command == "" {
            // No command entered.
            return;
        }
        let req = Request {
            command: &command,
            options: &options,
            channel_id: channel_id,
            author: author,
        };
        // Handle shutdown specially. Handling it via a handler would mean that handler would need a
        // reference to the bot in oder to shut it down.
        if command=="shutdown"{
        	self.handle_shutdown(req);
        }
        // TODO why the hell do i have to do this here? i'm sure i don't. shouldn't.
        let unknown_handler = UnknownHandler;
        let unknown_handler_as_request_handler:&RequestHandler=&unknown_handler;
        self.request_handlers
            .get(req.command)
            .unwrap_or(&unknown_handler_as_request_handler)
            .handle(req, &self.discord, &mut self.sh_status);
        //        match &*command {
        //            // TODO unhardcode command strings
        //            "help" => self.handle_help(req, &options),
        //            "echo" => self.handle_echo(req, &options),
        //            "want" => self.handle_want(req),
        //            "status" => self.handle_status(req),
        //            "shutdown" => self.handle_shutdown(req),
        //            unknown_command => self.handle_unknown(req, unknown_command),
        //        }
    }

    // TODO factor out request handling
    fn handle_echo(&self, req: Request, options: &str) {
        let reply = req.author.name.clone() + " wants me to echo \"" + options + "\".";
        if let Err(msg) = self.discord
            .send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_help(&self, req: Request, options: &str) {
        // TODO
        let reply = "This is an unhelpful help text. There'll be a better one, I promise.";
        if let Err(msg) = self.discord
            .send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_unknown(&mut self, req: Request, unknown_command: &str) {
//        self.unknown_handler.handle(req, &self.discord, &mut self.sh_status);
    }

    fn handle_shutdown(&mut self, req: Request) {
        if let Err(msg) = self.discord
            .send_message(&req.channel_id, "Shutting down. Bye now.", "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
        self.running = false;
    }

    fn handle_want(&mut self, req: Request) {
        self.sh_status.user_wants_sh(req.author.id);
        let reply = "Ok, I'll put you on the list.";
        if let Err(msg) = self.discord
            .send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_status(&mut self, req: Request) {
        let num_wanting = self.sh_status.num_users_wanting_sh();
        // TODO special case one player (is/are)
        let reply = format!("There are currently {} players who want to play Stronghold.",
                            num_wanting);
        if let Err(msg) = self.discord
            .send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}
