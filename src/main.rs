extern crate discord;
extern crate chrono;

use discord::{Discord, GetMessages};
use discord::model::{ChannelType, UserId};
use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use chrono::offset::fixed::FixedOffset;

fn main() {
    // Log in to Discord using a bot token from the environment
	let discord = Discord::from_bot_token(include_str!("bot_key.txt").trim()).expect("login failed");
	discord.connect().expect("Login failed.");
	let me = UserId(include_str!("bot_id.txt").trim().parse::<u64>().expect("Unable to interpret bot_id.txt as u64"));
	for server in discord.get_servers().expect("Getting servers failed") {
		println!("Handling server: {}", server.name);
		let channels_query = discord.get_server_channels(server.id);
		if let Err(err) = channels_query {
			println!("Error when retrieving channels: {:?}", err);
		}
		else {
			'channel_loop: for channel in channels_query.unwrap() {
				//Channel is not marked as permanent
				if !channel.name.ends_with('-') {
					println!("Found temporary channel: {}", channel.name);
					if channel.kind == ChannelType::Text {
						println!("{} is text channel.", channel.name);
						let last_msg_query = discord.get_messages(
							channel.id,
							GetMessages::MostRecent,
							Some(1)
						);
						if let Err(err) = last_msg_query {
							println!("Error retrieving most recent message: {:?}", err);
						}
						else
						{
							let last_msg_vec = last_msg_query.unwrap();
							if last_msg_vec.len() == 0 {
								println!("No messages found in channel.  Skipping.");
							}
							else {
								println!("Got most recent message..");
								let mut last_msg = last_msg_vec[0].clone();
								//Find the last message that wasn't sent by us
								while last_msg.author.id == me {
									println!("Message id {} is from me, getting the one before it instead.", last_msg.id);
									let msg_query = discord.get_messages(
										channel.id,
										GetMessages::Before(last_msg.id),
										Some(1)
									);
									//Could not find a message sent by someone other than us, so skip this channel.
									if let Err(err) = msg_query  {
										println!("Skipping this channel, error on getting message before current message.");
										println!("Error text: {:?}", err);
										continue 'channel_loop;
									}
									else {
										let msg_query_vec = msg_query.unwrap();
										if msg_query_vec.len() == 0 {
											println!("Skipping this channel, since no message was sent by anyone other than us.");
											continue 'channel_loop;
										}
										else {
											last_msg = msg_query_vec[0].clone();
										}
									}
								}
								println!("Found good user message, proceeding.");
								let msg_time;
								match DateTime::<FixedOffset>::parse_from_rfc3339(&last_msg.timestamp) {
									Ok(timestamp) => {
										msg_time = timestamp;
									}
									Err(err) => {
										println!("Could not parse timestamp for most recent non-bot message from channel: {}", channel.name);
										println!("Error: {:?}", err);
										continue 'channel_loop;
									}
								}
								println!("Timestamp of message being evaluated is: {}", msg_time);
								let days_old = (Local::now() - msg_time).num_days();
								println!("Message being evaluated is determined to be {} days old.", days_old);
								//If the message is 6 days old then send a warning.
								if days_old == 6 {
									println!("Message is 6 days old, sending warning message.");
									let result = discord.send_message(
										&channel.id,
										include_str!("delete_warning.txt"),
										"",
										false
									);
									if result.is_err() {
										println!("Failed to send warning message to channel: {}", channel.name);
									}
								}
								else if days_old >= 7 {
									println!("Message is 7 days old, deleting channel.");
									let result = discord.delete_channel(&channel.id);
									if result.is_err() {
										println!("Failed to delete channel: {}", channel.name);
									}
								}
							}
						}
					}
					else {
						println!("{} is voice channel.", channel.name);
					}
				}
				//Channel is permanent
				else {
					println!("Found permanent channel: {}", channel.name);
				}
			}
		}
	}
	println!("Parsing complete.");
}
