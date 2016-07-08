extern crate discord;

use discord::Discord;
use discord::model::Event;

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

struct ShBot {
    discord: discord::Discord,
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
                false
            }
            Ok(Event::MessageCreate(msg)) => {
                if msg.author.id == self.my_id {
                    // Don't repeat own messages.
                    return false;
                }
                let reply = msg.author.name + " just said \"" + &msg.content + "\"";
                self.discord
                    .send_message(&msg.channel_id, &reply, "", false)
                    .expect("failed to send msg");
                true
            }
            _ => false,
        }
    }
}
