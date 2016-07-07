extern crate discord;

use discord::Discord;
use discord::model::Event;

fn main() {
    let token: String;
    match std::env::args().nth(1) {
        None => {
            println!("Pass the bot token as an argument.");
            std::process::exit(1);
        }
        Some(t) => token = t,
    }
    let d = Discord::from_bot_token(&token).expect("error logging in");
    println!("logged in");

    let (mut c, re) = d.connect().expect("failed connect");
    let my_id = re.user.id;
    println!("connected");

    let mut num_msgs = 0;
    while num_msgs < 2 {
        match c.recv_event() {
            Err(_) => println!("error receiving event"),
            Ok(Event::MessageCreate(msg)) => {
                if msg.author.id == my_id {
                    // Don't repeat own messages.
                    continue;
                }
                let reply = msg.author.name + " just said \"" + &msg.content + "\"";
                d.send_message(&msg.channel_id, &reply, "", false).expect("failed to send msg");
                num_msgs += 1;
            }
            _ => {}
        }
        // Wait a bit because of the rate limit.
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    c.shutdown().expect("failed shutdown");
    println!("connection shutdown");
}
