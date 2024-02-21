#[macro_use]
mod data;
mod datefn;
use std::{fs, io::ErrorKind, path::PathBuf, thread, time::Duration};

use chrono::{
    format::ParseErrorKind, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, ParseError,
};
use data::{Config, ConfigError};
use datefn::WithinRange;

use crate::data::ErrorDisplay;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.contains(&String::from("--gen-test")) {
        create_test_folders("test/", true);
        // create_test_folders("test/deep/", false);
        // create_test_folders("test/deep/verydeep/", false);
        return;
    }
    if args.len() == 1 {
        println!("Please provide a path!");
        return;
    }

    let once = args.contains(&String::from("--once"));

    let p = args.last().unwrap().to_string();
    _ = thread::spawn(move || loop {
        if !execute(&p, !once) {
            break;
        }
    })
    .join();
}

fn execute(path: &String, schedule: bool) -> bool {
    let root_path = PathBuf::from(&path).canonicalize().unwrap();

    let config = match get_config(&root_path) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e.display());
            return false;
        }
    };

    let mut queue = Vec::<PathBuf>::new();
    queue.push(root_path);
    while let Some(path) = queue.pop() {
        let mut children_dirs = clean_dir(&path, &config);
        queue.append(&mut children_dirs);
    }

    if schedule {
        println_info!(config, "Cleaned at: {}", Local::now());
        thread::sleep(Duration::from_secs(config.refresh));
    } else {
        return false;
    }
    return true;
}

fn get_config(path: &PathBuf) -> Result<Config, ConfigError> {
    let root_path = PathBuf::from(path)
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let mut config_path = PathBuf::from(&root_path);
    config_path.push("dateframe.conf");

    Config::from_file(config_path.to_str().unwrap())
}

fn clean_dir(path: &PathBuf, config: &Config) -> Vec<PathBuf> {
    let mut children = Vec::<PathBuf>::new();

    let mut options = open_dir(path).expect("Cannot open dir");
    options.sort();

    for o in options.into_iter() {
        let mut target_path = PathBuf::from(path);
        target_path.push(&o);
        println!("\t{}", target_path.display());

        let file_name = config.format_name(&o);
        println_debug!(config, "\t\tUsing name {} for {}", file_name, o);

        if let Ok(md) = fs::metadata(&target_path) {
            match attempt_path_parse(config, &file_name.trim()) {
                Some(date) => {
                    println_debug!(config, "\t\tParsed {} as {}", o, date);
                    if !date.is_within(config) {
                        match fs::remove_dir_all(&target_path) {
                            Ok(_) => {
                                println_info!(config, "\t\t! {} removed", &o)
                            }
                            Err(x) => match x.kind() {
                                ErrorKind::PermissionDenied => println_error!(
                                    config,
                                    "\t\tCouldn't remove folder, permission denied"
                                ),
                                _ => println_error!(config, "\t\tError with folder, {}", x),
                            },
                        }
                    }
                }
                None => {
                    if config.deep && !md.is_file() {
                        children.push(target_path);
                    }
                    println_info!(config, "\t\t{} did not match date format, skipping.", o);
                }
            }
        }
    }
    children
}

fn attempt_path_parse(config: &Config, path: &str) -> Option<NaiveDateTime> {
    for f in &config.format {
        match NaiveDateTime::parse_from_str(path, &f) {
            Ok(date) => return Some(date),
            Err(x) => match x.kind() {
                ParseErrorKind::NotEnough => match NaiveDate::parse_from_str(path, &f) {
                    Ok(date) => {
                        return Some(config.date_only_behavior.add_to_date(date))
                    }
                    Err(_) => {}
                },
                _ => {}
            },
        }
    }
    return None;
}

fn open_dir(path: &PathBuf) -> Option<Vec<String>> {
    fs::read_dir(path).ok().map(|files| {
        files
            .filter_map(|f| Some(f.ok()?.file_name().into_string().unwrap()))
            .collect::<Vec<_>>()
    })
}

const TEST_CONF: &str = "
format=%Y-%m-%dT%H-%M-%S
format=%Y-%m-%d
retention=10d
refresh=20
log=debug
remove=Test
split_string= 
split_index=0
date_only_behavior=h11
";

fn create_test_folders(root_dir: &str, write_config: bool) {
    if write_config {
        let mut config_file = PathBuf::from(root_dir);
        config_file.push("dateframe.conf");
        let _ = fs::write(config_file, TEST_CONF);
    }
    let today = Local::now();
    let date_past_to_generate = 60;
    let format_one = "%Y-%m-%dT%H-%M-%S";
    let format_two = "%Y-%m-%d";
    for n in (0..date_past_to_generate).rev() {
        let prev = today.checked_sub_days(Days::new(n)).unwrap();
        let formatted_value = prev.format(format_one).to_string();
        let mut write_path = PathBuf::from(root_dir);
        write_path.push(&formatted_value);
        match fs::create_dir_all(write_path) {
            Ok(_) => println!("{} created!", formatted_value),
            Err(_) => println!("Could not create {}!", formatted_value),
        };
    }
    for n in (0..date_past_to_generate).rev() {
        let prev = today.checked_sub_days(Days::new(n)).unwrap();
        let formatted_value = prev.format(format_two).to_string();
        let mut write_path = PathBuf::from(root_dir);
        write_path.push(&formatted_value);
        match fs::create_dir_all(write_path) {
            Ok(_) => println!("{} created!", formatted_value),
            Err(_) => println!("Could not create {}!", formatted_value),
        };
    }
}
