use discord::{Discord, State};
use discord::model::UserId;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::str::FromStr;
use chrono::NaiveDate;
use chrono::offset::local::Local;
use chrono::Datelike;

const USER_FILE_PATH: &'static str = include_str!("user_file_path.txt");

pub fn collect_birthdays(discord: &mut Discord, state: &mut State) {
    let known_birthdays = load_user_file();
    for server in state.servers() {
        for member in &server.members {
            let mut user_birthday;
            let user = &member.user;
            for birthday in &known_birthdays {
                if birthday.user == user.id {
                    user_birthday = get_next_birthday_date(birthday.month, birthday.day);
                }
            }
            let pm_result = discord.create_private_channel(user.id);
            if let Err(err) = pm_result {
                println!("Unable to pm user: {}.  Skipping.", user.name);
                continue;
            } else {
                let pm = pm_result.unwrap();

            }
            // In our user file a birthday of 0/0 indicates we have introduced ourself to the user
            // but do not know their birthday yet.
        }
    }
}

// Get the next date for which it is month and day.
// i.e. if today is 11/24/16 and my birthday is on June 20th this should return 06/20/17
fn get_next_birthday_date(month: u8, day: u8) -> Option<NaiveDate> {
    if month == 0 || day == 0 {
        return None;
    }
    let now = Local::now().date().naive_utc();
    let current_year = now.year();
    let result;
    // Leap day in a non-leap year.
    if month == 2 && day == 29 && current_year % 4 != 0 {
        // Bump the day back 1 so it's valid.
        result = NaiveDate::from_ymd_opt(current_year, 2, 28);
    } else {
        result = NaiveDate::from_ymd_opt(current_year, month as u32, day as u32);
    }
    if let None = result {
        println!("{}/{} month and day out of range.  Invalid.", month, day);
        return None;
    }
    if let Some(mut date) = result {
        // We're about to advance the year.
        if date < now {
            // Leap day handling.
            // Leap day is valid this year but won't be next year, so take the day back 1
            // before we increment the year.
            if month == 2 && day == 29 && current_year % 4 == 0 {
                date = date.with_day(28).unwrap();
            }
            let mut new_date = date.with_year(current_year + 1);
            if new_date.is_some() {
                // If the input was a leap day that wasn't valid this year but will be next year
                // we need to add the leap day back in.
                if month == 2 && day == 29 && (current_year + 1) % 4 == 0 {
                    return new_date.unwrap().with_day(29);
                }
                return new_date;
            }
            if new_date.is_none() {
                println!("Year out of range: {}", current_year + 1);
                println!("261,954 years of service.  Yeah baby!");
                println!("P.S. sorry about the crash future boy/girl/alien/robot.");
                println!("Probably shouldn't have used code that hadn't been maintained for over \
                          200,000 years though.");
                return None;
            }
        } else {
            return Some(date);
        }
    }
    // Unreachable.
    return None;
}

struct UserBirthday {
    user: UserId,
    month: u8,
    day: u8,
}

fn load_user_file() -> Vec<UserBirthday> {
    let result = File::open(Path::new(USER_FILE_PATH.trim()));
    let mut to_return = Vec::<UserBirthday>::new();
    match result {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf);
            let lines = buf.split('\n');
            'lines: for line in lines {
                let values = line.split(' ').collect::<Vec<&str>>();
                if values.len() >= 2 {
                    let user_num = u64::from_str(values[0]);
                    if let Err(err) = user_num {
                        println!("Unable to parse UserId, skipping. Error: {:?}", err);
                        continue 'lines;
                    }
                    let day_values = values[1].split('/').collect::<Vec<&str>>();
                    if day_values.len() >= 2 {
                        let month_num = u8::from_str(day_values[0]);
                        let day_num = u8::from_str(day_values[1]);
                        if let Err(err) = month_num {
                            println!("Unable to parse month, skipping. Error: {:?}", err);
                            continue 'lines;
                        }
                        if let Err(err) = day_num {
                            println!("Unable to parse day, skipping. Error: {:?}", err);
                            continue 'lines;
                        }
                        to_return.push(UserBirthday {
                            user: UserId(user_num.unwrap()),
                            month: month_num.unwrap(),
                            day: day_num.unwrap(),
                        });
                    } else {
                        println!("Unable to parse line, no / found. Line: {:?}", line);
                    }
                } else {
                    println!("Unable to parse line, no space found. Line: {:?}", line);
                }
            }
        }
        Err(err) => {
            println!("Failed to open user file: {:?}", err);
        }
    }
    to_return
}

fn write_user_file(birthdays: &Vec<UserBirthday>) {
    let result = File::create(Path::new(USER_FILE_PATH.trim()));
    match result {
        Ok(mut file) => {
            let mut buf = String::new();
            let mut first_line = true;
            for birthday in birthdays {
                if first_line {
                    first_line = false;
                } else {
                    buf.push('\n');
                }
                buf.push_str(format!("{}", birthday.user).as_str());
                buf.push(' ');
                buf.push_str(format!("{}", birthday.month).as_str());
                buf.push('/');
                buf.push_str(format!("{}", birthday.day).as_str());
            }
            let write_result = file.write_all(buf.as_bytes());
            if write_result.is_err() {
                println!("Failed to write user file: {:?}",
                         write_result.err().unwrap());
            }
        }
        Err(err) => {
            println!("Failed to write to user file: {:?}", err);
        }
    }
}
