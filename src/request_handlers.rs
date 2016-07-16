use discord::model::{ChannelId, User};
use discord_connection::DiscordConnection;
use sh_status::ShStatus;

pub struct Request<'a> {
    pub command: &'a str,
    pub options: &'a str,
    pub channel_id: ChannelId,
    pub author: User,
}

pub trait RequestHandler {
    fn handle(&self, req: Request, discord: &DiscordConnection, sh_status: &mut ShStatus);
}

pub struct UnknownHandler;
// TODO only for testing
pub struct EchoHandler;
pub struct HelpHandler;
pub struct WantHandler;
pub struct StatusHandler;

impl RequestHandler for UnknownHandler {
    fn handle(&self, req: Request, discord: &DiscordConnection, sh_status: &mut ShStatus) {
        let reply = "\"".to_owned() + req.command +
                    "\" is not a valid command. Type \"help\" to find out what is.";
        if let Err(msg) = discord.send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}

impl RequestHandler for EchoHandler {
    fn handle(&self, req: Request, discord: &DiscordConnection, sh_status: &mut ShStatus) {
        let reply = req.author.name + " wants me to echo \"" + req.options + "\".";
        if let Err(msg) = discord.send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}

impl RequestHandler for HelpHandler {
    fn handle(&self, req: Request, discord: &DiscordConnection, sh_status: &mut ShStatus) {
        // TODO
        let reply = "This is an unhelpful help text. There'll be a better one, I promise.";
        if let Err(msg) = discord.send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}

impl RequestHandler for WantHandler {
    fn handle(&self, req: Request, discord: &DiscordConnection, sh_status: &mut ShStatus) {
        sh_status.user_wants_sh(req.author.id);
        let reply = "Ok, I'll put you on the list.";
        if let Err(msg) = discord.send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}

impl RequestHandler for StatusHandler {
    fn handle(&self, req: Request, discord: &DiscordConnection, sh_status: &mut ShStatus) {
        let num_wanting = sh_status.num_users_wanting_sh();
        // TODO special case one player (is/are)
        let reply = format!("There are currently {} players who want to play Stronghold.",
                            num_wanting);
        if let Err(msg) = discord.send_message(&req.channel_id, &reply, "", false) {
            // TODO log, don't print
            println!("Failed to send message: {}", msg);
        }
    }
}
