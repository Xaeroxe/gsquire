use discord::{Discord, State, Connection};
use discord::model::UserId;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};
use std::str::FromStr;

const USER_FILE_PATH : &'static str = include_str!("user_file_path.txt");

pub fn collect_birthdays(discord : &mut Discord, connection : &mut Connection, state : &mut State) {
    connection.download_all_members(state);
    for server in state.servers() {
        for member in &server.members {

        }
    }
}

struct UserBirthday {
    user : UserId,
    month : u8,
    day : u8,
}

fn load_user_file() -> Vec<UserBirthday> {
    let result = File::open(Path::new(USER_FILE_PATH));
    let mut to_return = Vec::<UserBirthday>::new();
    match result {
        Ok(mut file) => {
            let mut buf = String::new();
            file.read_to_string(&mut buf);
            let lines = buf.split('\n');
            'lines : for line in lines {
                let values = line.split(' ').collect::<Vec<&str>>();
                let user_num = u64::from_str(values[0]);
                if user_num.is_err() {
                    println!("Unable to parse UserId, skipping. Error: {:?}", user_num.err().unwrap());
                    continue 'lines;
                }
                let day_values = values[1].split('/').collect::<Vec<&str>>();
                let month_num = u8::from_str(day_values[0]);
                let day_num = u8::from_str(day_values[1]);
                if month_num.is_err() {
                    println!("Unable to parse month, skipping. Error: {:?}", month_num.err().unwrap());
                    continue 'lines;
                }
                if day_num.is_err() {
                    println!("Unable to parse day, skipping. Error: {:?}", day_num.err().unwrap());
                    continue 'lines;
                }
                to_return.push(UserBirthday{user : UserId(user_num.unwrap()), month : month_num.unwrap(), day : day_num.unwrap()})
            }
        }
        Err(err) => {
            println!("Failed to open user file: {:?}", err);
        }
    }
    return to_return;
}

fn write_user_file(birthdays : &Vec<UserBirthday>) {
    let result = File::create(Path::new(USER_FILE_PATH));
    match result {
        Ok(mut file) => {
            let mut buf = String::new();
            let mut first_line = true;
            for birthday in birthdays {
                if first_line {
                    first_line = false;
                }
                else {
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
                println!("Failed to write user file: {:?}", write_result.err().unwrap());
            }
        }
        Err(err) => {
            println!("Failed to write to user file: {:?}", err);
        }
    }
}
