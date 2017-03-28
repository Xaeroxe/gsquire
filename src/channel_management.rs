use discord::{Discord, GetMessages};
use discord::model::{ChannelType, UserId, ServerInfo, ChannelId, PublicChannel, Message};
use chrono::Duration;
use std::time::Duration as StdDuration;
use chrono::Weekday;
use chrono::Datelike;
use chrono::datetime::DateTime;
use chrono::offset::local::Local;
use chrono::offset::fixed::FixedOffset;
use std::thread::sleep;

const ME: UserId = UserId(include!("bot_id.txt"));

pub fn clear_old_channels(discord: &Discord, server: &ServerInfo) {
    println!("Clearing old channels on server: {}", server.name);
    let channels_query = discord.get_server_channels(server.id);
    if let Err(err) = channels_query {
        println!("Error when retrieving channels: {:?}", err);
    } else {
        for channel in channels_query.unwrap() {
            if channel_is_temp(&channel) {
                println!("Found temporary channel: {}", channel.name);
                process_temp_channel(discord, &channel);
            } else {
                println!("Found permanent channel: {}", channel.name);
                println!("\tSkipping.");
            }
        }
    }
}

pub fn it_is_wednesday_my_dudes(discord: &Discord, server: &ServerInfo) {
    println!("Is it wednesday my dudes? : {}", server.name);
    if Local::now().weekday() == Weekday::Wed {
        let channels_query = discord.get_server_channels(server.id);
        if let Err(err) = channels_query {
            println!("Error when retrieving channels: {:?}", err);
        } else {
            let channels_query = channels_query.unwrap();
            let channel = channels_query.iter().filter(|c| c.name == "announcements-").nth(0);
            if let Some(channel) = channel {
               println!("It's wednesday my dudes!");
                let result =
                    discord.send_message(&channel.id, "https://youtu.be/du-TY1GUFGk", "", false);
                if result.is_err() {
                    println!("Failed to send wednesday message to channel: {}",
                             &channel.id);
                }
            } else {
                println!("announcements- not found.");   
            }
        }
    }
}

pub fn april_fools(discord: &Discord, server: &ServerInfo) {
    let now = Local::now();
    if now.month() == 4 && now.day() == 1 {
        let channels_query = discord.get_server_channels(server.id);
        if let Err(err) = channels_query {
            println!("Error when retrieving channels: {:?}", err);
        } else {
            let channels_query = channels_query.unwrap();
            for channel in &channels_query {
                let result =
                    discord.send_message(&channel.id, "Channel deletion in 5...", "", false);
                if result.is_err() {
                    println!("Failed to send message to channel: {}", &channel.id);
                }
            }
            sleep(StdDuration::new(1, 0));
            for channel in &channels_query {
                let result = discord.send_message(&channel.id, "4...", "", false);
                if result.is_err() {
                    println!("Failed to send message to channel: {}", &channel.id);
                }
            }
            sleep(StdDuration::new(1, 0));
            for channel in &channels_query {
                let result = discord.send_message(&channel.id, "3...", "", false);
                if result.is_err() {
                    println!("Failed to send message to channel: {}", &channel.id);
                }
            }
            sleep(StdDuration::new(1, 0));
            for channel in &channels_query {
                let result = discord.send_message(&channel.id, "2...", "", false);
                if result.is_err() {
                    println!("Failed to send message to channel: {}", &channel.id);
                }
            }
            sleep(StdDuration::new(1, 0));
            for channel in &channels_query {
                let result = discord.send_message(&channel.id, "1...", "", false);
                if result.is_err() {
                    println!("Failed to send message to channel: {}", &channel.id);
                }
            }
            sleep(StdDuration::new(3, 0));
            for channel in &channels_query {
                let result = discord.send_message(&channel.id, "April fools!!", "", false);
                if result.is_err() {
                    println!("Failed to send message to channel: {}", &channel.id);
                }
            }
        }
    }
}

fn channel_is_temp(channel: &PublicChannel) -> bool {
    !channel.name.ends_with('-')
}

fn process_temp_channel(discord: &Discord, channel: &PublicChannel) {
    if channel.kind == ChannelType::Text {
        println!("{} is text channel.", channel.name);
        process_temp_text_channel(discord, channel);
    } else {
        println!("{} is voice channel.", channel.name);
    }
}

fn process_temp_text_channel(discord: &Discord, channel: &PublicChannel) {
    let days_old = get_channel_inactive_duration(discord, channel).num_days();
    if days_old == 6 {
        send_delete_warning(discord, &channel.id);
    } else if days_old >= 7 {
        // Never delete a channel on which a warning hasn't been sent.
        match get_warning(discord, channel) {
            Some(warning) => {
                let msg_time;
                match DateTime::<FixedOffset>::parse_from_rfc3339(&warning.timestamp) {
                    Ok(timestamp) => {
                        msg_time = timestamp;
                    }
                    Err(err) => {
                        println!("Could not parse warning timestamp: {}", channel.name);
                        println!("Error: {:?}", err);
                        // This is a SOL scenario.  We found a warning but can't tell
                        // how old it is.  The only thing we can do is send another
                        // and hope this one turns out better than the last one.
                        msg_time = Local::now().with_timezone(&FixedOffset::east(0));
                        send_delete_warning(discord, &channel.id);
                    }
                }
                // 22 is intentional here as exactly 24 hours almost never happens.
                if (Local::now() - msg_time).num_hours() >= 22 {
                    println!("Warning found and it is at least 22 hours old.  Deleting channel.");
                    let result = discord.delete_channel(&channel.id);
                    if result.is_err() {
                        println!("Failed to delete channel: {}", channel.name);
                    }
                }
                // else warning is not old enough yet, don't delete.
            }
            None => {
                println!("Would normally delete this now but no warning has been sent.");
                send_delete_warning(discord, &channel.id);
            }
        }
    }
}

fn get_warning(discord: &Discord, channel: &PublicChannel) -> Option<Message> {
    let last_msg_query = discord.get_messages(channel.id, GetMessages::MostRecent, Some(1));
    if let Err(err) = last_msg_query {
        println!("Error retrieving most recent message: {:?}", err);
        send_filler_message(discord, &channel.id);
        return None;
    } else {
        let last_msg_vec = last_msg_query.unwrap();
        if last_msg_vec.len() == 0 {
            println!("No messages found in channel.  Posting one.");
            send_filler_message(discord, &channel.id);
            return None;
        } else {
            println!("Got most recent message.  Checking if warning.");
            let last_msg = last_msg_vec[0].clone();
            if message_is_warning(&last_msg) {
                return Some(last_msg);
            } else {
                return None;
            }
        }
    }
}

fn message_is_warning(message: &Message) -> bool {
    message.author.id == ME && message.content.starts_with("WARNING CHANNEL DELETION IMMINENT!")
}

fn get_channel_inactive_duration(discord: &Discord, channel: &PublicChannel) -> Duration {
    // Get the most recent message from someone other than gsquire.
    // If no such message exists then use one from gsquire.
    // If there are no messages on this channel at all, post one.
    let last_msg_query = discord.get_messages(channel.id, GetMessages::MostRecent, Some(1));
    if let Err(err) = last_msg_query {
        println!("Error retrieving most recent message posting one.: {:?}",
                 err);
        send_filler_message(discord, &channel.id);
        return Duration::days(0);
    } else {
        let last_msg_vec = last_msg_query.unwrap();
        if last_msg_vec.len() == 0 {
            println!("No messages found in channel.  Posting one.");
            send_filler_message(discord, &channel.id);
            return Duration::days(0);
        } else {
            println!("Got most recent message..");
            let mut best_msg = last_msg_vec[0].clone();
            let mut last_is_warning = message_is_warning(&best_msg);
            let mut last_msg = best_msg.clone();
            // If this was sent by gsquire try and find one that isn't.
            'search: while best_msg.author.id == ME {
                println!("Message id {} is from me, getting the one before it.",
                         best_msg.id);
                let msg_query =
                    discord.get_messages(channel.id, GetMessages::Before(best_msg.id), Some(1));
                if let Err(err) = msg_query {
                    println!("Error on getting message before current message.");
                    println!("Error text: {:?}", err);
                    break 'search;
                } else {
                    let msg_query_vec = msg_query.unwrap();
                    if msg_query_vec.len() == 0 {
                        println!("No message was sent by anyone other than me.");
                        break 'search;
                    } else {
                        best_msg = msg_query_vec[0].clone();
                        // In the event that gsquire is the only sender on this channel gsquire
                        // should not use its own warning message to determine the age of the channel.
                        // Otherwise gsquire will continue warning indefinitely but never actually
                        // delete the channel.  This channel will likely contain a filler message
                        // posted by gsquire for the purpose of determining channel age.
                        // This code will likely grab that filler message.
                        if last_is_warning && !message_is_warning(&best_msg) {
                            last_msg = best_msg.clone();
                            last_is_warning = false;
                        }
                    }
                }
            }

            // If all messages in channel were sent by gsquire, use the most recent one to
            // determine length of inactivity.
            if best_msg.author.id == ME {
                best_msg = last_msg;
            }
            println!("Found good message, proceeding.");
            let msg_time;
            match DateTime::<FixedOffset>::parse_from_rfc3339(&best_msg.timestamp) {
                Ok(timestamp) => {
                    msg_time = timestamp;
                }
                Err(err) => {
                    println!("Could not parse timestamp from channel: {}", channel.name);
                    println!("Error: {:?}", err);
                    return Duration::days(0);
                }
            }
            println!("Timestamp of message being evaluated is: {}", msg_time);
            return Local::now() - msg_time;
        }
    }
}

fn send_filler_message(discord: &Discord, channel_id: &ChannelId) {
    println!("Sending filler message.");
    let result = discord.send_message(channel_id, include_str!("filler_message.txt"), "", false);
    if result.is_err() {
        println!("Failed to send filler message to channel: {}", channel_id);
    }
}

fn send_delete_warning(discord: &Discord, channel_id: &ChannelId) {
    println!("Sending warning message.");
    let result = discord.send_message(channel_id, include_str!("delete_warning.txt"), "", false);
    if result.is_err() {
        println!("Failed to send warning message to channel: {}", channel_id);
    }
}
