extern crate discord;

mod discord_connection;

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

struct Request {
    channel_id: ChannelId,
    content: String,
    author: User,
}

struct ShBot<D: DiscordConnection> {
    discord: D,
    me: CurrentUser,
}

// TODO do i have to specify which kind of discordconnection?
impl ShBot<BotConnection> {
    fn new(token: &str) -> Self {
        let (d, me) = BotConnection::from_bot_token(token);
        ShBot {
            discord: d,
            me: me,
        }
    }

    fn run(mut self) {
        loop {
            self.handle_event();
        }
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
                let req = Request {
                    channel_id: msg.channel_id,
                    content: msg.content,
                    author: msg.author,
                };
                self.handle_request(req);
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
                    let (first, second) = {
                        let mut parts = msg.content.splitn(2, |c: char| c.is_whitespace());
                        let first = parts.next().unwrap_or("").trim().to_owned();
                        let second = parts.next().unwrap_or("").trim().to_owned();
                        (first, second)
                    };
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

    fn handle_request(&mut self, req: Request) {
        // Split at the first whitespace into command and options.
        // TODO i have to use string here so i can do to_owned() to copy the splitn element and
        // later have to deref to get str back. is there a better way to copy?
        let (command, options) = {
            let mut parts = req.content.splitn(2, |c: char| c.is_whitespace());
            let command: String;
            if let Some(s) = parts.next() {
                command = s.trim().to_owned();
            } else {
                // TODO no command entered
                println!("no command");
                return;
            }
            let options = parts.next().unwrap_or("").trim().to_owned();
            (command, options)
        };

        match &*command {
            // TODO unhardcode command strings
            "help" => self.handle_help(req, &options),
            "echo" => self.handle_echo(req, &options),
            "shutdown" => self.handle_shutdown(),
            unknown_command => self.handle_unknown(req, unknown_command),
        }
    }

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

    fn handle_unknown(&self, req: Request, unknown_command: &str) {
        let reply = "\"".to_owned() + unknown_command +
                    "\" is not a valid command. Type \"help\" to find out what is.";
        if let Err(msg) = self.discord
            .send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }

    fn handle_shutdown(&self) {
        std::process::exit(0);
    }
}
