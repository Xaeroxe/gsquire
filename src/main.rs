extern crate chrono;
extern crate discord;
extern crate rand;

use std::thread::sleep;
use std::time::Duration;

use discord::{Discord, State};

mod channel_management;

fn main() {
    // Log in to Discord using a bot token from the environment
    let discord =
        Discord::from_bot_token(include_str!("bot_key.txt").trim()).expect("Login failed.");

    let (connection, ready_event) = discord.connect().expect("Websocket login failed.");
    let _state = State::new(ready_event);
    connection.set_game_name(String::from("Your mom."));
    for server in discord.get_servers().expect("Getting servers failed") {
        channel_management::it_is_wednesday_my_dudes(&discord, &server);
        channel_management::clear_old_channels(&discord, &server);
    }
    sleep(Duration::from_secs(10)); // Sleep long enough to show off the game name joke.
    let shutdown_result = connection.shutdown();
    if let Err(err) = shutdown_result {
        println!("Failed to disconnect from server.  Error: {:?}", err);
    }
    println!("Job finished.");
}
