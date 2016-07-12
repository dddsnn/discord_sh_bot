extern crate discord;

use discord::Discord;
use discord::model::Event;
use discord::model::ChannelId;
use discord::model::User;

fn main() {
    let token: String;
    if let Some(t) = std::env::args().nth(1) {
        token = t;
    } else {
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

struct ShBot {
    discord: Discord,
    conn: discord::Connection,
    my_id: discord::model::UserId,
}

impl ShBot {
    fn new(token: &str) -> Self {
        let d = Discord::from_bot_token(&token).expect("error logging in");
        println!("logged in");

        let (c, re) = d.connect().expect("failed connect");
        let my_id = re.user.id;
        println!("connected");
        ShBot {
            discord: d,
            conn: c,
            my_id: my_id,
        }
    }

    /// Shuts down when done.
    fn run(mut self) {
        let mut num_msgs = 0;
        while num_msgs < 2 {
            if self.handle_event() == true {
                num_msgs += 1;
            }
            // Wait a bit because of the rate limit.
            // TODO do smarter retrying
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        self.conn.shutdown().expect("failed shutdown");
        println!("connection shutdown");
    }

    fn handle_event(&mut self) -> bool {
        // TODO remove return type (only used to limit number of loops)
        match self.conn.recv_event() {
            Err(_) => {
                println!("error receiving event");
                // TODO
                false
            }
            Ok(Event::MessageCreate(msg)) => {
                if msg.author.id == self.my_id {
                    // Don't repeat own messages.
                    return false;
                }
                let req = Request {
                    channel_id: msg.channel_id,
                    content: msg.content,
                    author: msg.author,
                };
                self.handle_request(req);
                true
            }
            // TODO
            _ => false,
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
            "help" => self.handle_help(req, &options),
            "echo" => self.handle_echo(req, &options),
            unknown_command => self.handle_unknown(req, unknown_command),
        }
    }

    fn handle_echo(&self, req: Request, options: &str) {
        let reply = req.author.name.clone() + " wants me to echo \"" + options + "\".";
        self.discord
            .send_message(&req.channel_id, &reply, "", false)
            .expect("failed to send msg");
    }

    fn handle_help(&self, req: Request, options: &str) {
        let reply = "This is an unhelpful help text. There'll be a better one, I promise.";
        self.discord
            .send_message(&req.channel_id, &reply, "", false)
            .expect("failed to send msg");
    }

    fn handle_unknown(&self, req: Request, unknown_command: &str) {
        let reply = "\"".to_owned() + unknown_command +
                    "\" is not a valid command. Type \"help\" to find out what is.";
        self.discord
            .send_message(&req.channel_id, &reply, "", false)
            .expect("failed to send msg");
    }
}
