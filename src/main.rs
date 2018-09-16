extern crate chrono;
extern crate discord;

use discord::Discord;

mod channel_management;

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord =
        Discord::from_bot_token(include_str!("bot_key.txt").trim()).expect("Login failed.");

    let (connection, _ready_event) = discord.connect().expect("Websocket login failed.");
    connection.set_game_name(String::from("Your mom."));
    for server in discord.get_servers().expect("Getting servers failed") {
        channel_management::clear_old_channels(&discord, &server);
    }
    let shutdown_result = connection.shutdown();
    if let Err(err) = shutdown_result {
        println!("Failed to disconnect from server.  Error: {:?}", err);
    }
    println!("Job finished.");
}
