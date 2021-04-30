extern crate serde;
extern crate chrono;
extern crate url;
extern crate serde_json;
extern crate clap;
use std::process::Command;
use std::convert::AsRef;
use std::fmt;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::env;
use serde::{Serialize, Deserialize};
use chrono::{Datelike, Timelike};

/*
 * TO-DO
 * Split into multiple files.
 * Add tests.
*/
fn main() {
    let homedir = env::var("HOME").expect("error in getting home directory");
    let mut config_dir = PathBuf::new();
    config_dir.push(&homedir);
    config_dir.push("autolink-config.json");
    let config_dir = config_dir.to_str().unwrap();
    let app = clap::App::new("autolink")
                            .version("1.0")
                            .author("Oğuz Kurt <kurt.oguz@outlook.com>")
                            .about("when the time comes, opens links automatically")
                            .subcommand(clap::SubCommand::with_name("start")
                                            .about("start the application")
                                            .arg(clap::Arg::with_name("config")
                                                .help("JSON formatted config file")
                                                .long("config")
                                                .short("c")
                                                .default_value(&config_dir)))
                            .subcommand(clap::SubCommand::with_name("list")
                                            .about("list all link entries")
                                            .arg(clap::Arg::with_name("config")
                                                .help("JSON formatted config file")
                                                .long("config")
                                                .short("c")
                                                .default_value(&config_dir)))
                            .subcommand(clap::SubCommand::with_name("add")
                                            .about("add link entries")
                                            .arg(clap::Arg::with_name("name")
                                                    .help("the name of the link entry")
                                                    .index(1)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("link")
                                                    .help("the link")
                                                    .index(2)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("config")
                                                    .help("JSON formatted config file")
                                                    .long("config")
                                                    .short("c")
                                                    .default_value(&config_dir)))
                            .subcommand(clap::SubCommand::with_name("remove")
                                            .about("remove an entry")
                                            .arg(clap::Arg::with_name("name")
                                                    .help("name of the link entry to be removed")
                                                    .index(1)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("config")
                                                    .help("JSON formatted config file")
                                                    .long("config")
                                                    .short("c")
                                                    .default_value(&config_dir)))
                            .subcommand(clap::SubCommand::with_name("launch")
                                            .about("open a link")
                                            .arg(clap::Arg::with_name("name")
                                                .help("name of the link to be opened")
                                                .index(1)
                                                .required(true))
                                            .arg(clap::Arg::with_name("config")
                                                .help("JSON formatted config file")
                                                .long("config")
                                                .short("c")
                                                .default_value(&config_dir)))
                            .subcommand(clap::SubCommand::with_name("create")
                                        .about("create the config file"))
                            .subcommand(clap::SubCommand::with_name("edit")
                                            .about("edit current entries")
                                            .arg(clap::Arg::with_name("name")
                                                .help("name of the link entry to be changed")
                                                .index(1)
                                                .required(true))
                                            .arg(clap::Arg::with_name("new name")
                                                .help("new name of the entry")
                                                .long("new-name")
                                                .short("n")
                                                .takes_value(true))
                                            .arg(clap::Arg::with_name("new link")
                                                .help("new link for the entry")
                                                .long("new-link")
                                                .short("l")
                                                .takes_value(true))
                                            .arg(clap::Arg::with_name("add time")
                                                .help("add time for an entry")
                                                .long("add-time")
                                                .short("a")
                                                .requires_all(&["time", "day"]))
                                            .arg(clap::Arg::with_name("remove time")
                                                .help("remove time")
                                                .long("remove-time")
                                                .short("r")
                                                .requires_all(&["time", "day"]))
                                            .arg(clap::Arg::with_name("time")
                                                .help("specify the time")
                                                .long("time")
                                                .short("t")
                                                .takes_value(true))
                                            .arg(clap::Arg::with_name("day")
                                                .help("specify the day (first letter should be capitalized)")
                                                .long("day")
                                                .short("d")
                                                .takes_value(true))
                                            .arg(clap::Arg::with_name("config")
                                                .help("JSON formatted config file")
                                                .long("config")
                                                .short("c")
                                                .default_value(&config_dir)))
                            .get_matches();

    match app.subcommand() {
        ("start", Some(args)) => start(args),
        ("list", Some(args)) => list(args),
        ("add", Some(args)) => add(args),
        ("remove", Some(args)) => remove(args),
        ("launch", Some(args)) => launch(args),
        ("create", Some(args)) => create(/*args*/),
        ("edit", Some(args)) => edit(args),
        _ => println!("use subcommand help for usage")
    }
}

fn start(a: &clap::ArgMatches) {
    let plans = import(a.value_of("config").expect("error in opening config file (use create command to create if not)"));
    if plans.len() == 0 {
        panic!("no link entries set")
    }
    check_all(&plans);
}

fn list(a: &clap::ArgMatches) {
    let plans = import(a.value_of("config").expect("error in opening config file (use create command to create if not)"));
    println!("link entries:");
    for plan in plans {
        println!("{}", plan);
    }
}

fn add(a: &clap::ArgMatches) {
    let mut plans = import(a.value_of("config").expect("error in opening config file (use create command to create if not)"));
    let new_plan = Plan::new(a.value_of("name").unwrap().to_string(), a.value_of("link").unwrap().to_string(), Vec::new());
    plans.push(new_plan);
    export(plans, a.value_of("config").unwrap());
}

fn remove(a: &clap::ArgMatches) {
    let mut plans = import(a.value_of("config").expect("error in opening config file (use create command to create if not)"));
    for i in 0..plans.len() {
        if plans.get(i).unwrap().name == a.value_of("name").unwrap() {
            println!("removed entry: {}", a.value_of("name").unwrap());
            plans.remove(i);
        }
    }
    export(plans, a.value_of("config").unwrap());
}

fn launch(a: &clap::ArgMatches) {
    let plans = import(a.value_of("config").expect("error in opening config file (use create command to create if not)"));
    for plan in plans {
        if plan.name == a.value_of("name").unwrap() {
            println!("launched {}", a.value_of("name").unwrap());
            open_link(&plan.link);
        }
    }
}

fn create() {
    let homedir = env::var("HOME").expect("error in getting home directory");
    let _ = File::create(homedir + "/autolink-config.json");
}

fn edit(a: &clap::ArgMatches) {
    let mut plans = import(a.value_of("config").expect("error in opening config file (use create command to create if not)"));
    let length = plans.len();

    if a.is_present("new link") {
        for i in 0..length {
            if plans.get(i).unwrap().name == a.value_of("name").unwrap() {
                let mut p = plans.get(i).unwrap().clone();
                plans.remove(i);
                p.link = a.value_of("new link").unwrap().to_string();
                plans.push(p);
                println!("changed link of entry {} to {}", a.value_of("name").unwrap(), a.value_of("new link").unwrap());
                break
            }
        }
    }

    if a.is_present("add time") {
        for i in 0..length {
            if plans.get(i).unwrap().name == a.value_of("name").unwrap() {
                let mut p = plans.get(i).unwrap().clone();
                plans.remove(i);

                let time = chrono::NaiveTime::parse_from_str(&a.value_of("time").unwrap(), "%H:%M").expect("date cannot be parsed");
                let day = match a.value_of("day").unwrap().to_lowercase().as_str() {
                    "monday" => chrono::Weekday::Mon,
                    "tuesday" => chrono::Weekday::Tue,
                    "wednesday" => chrono::Weekday::Wed,
                    "thursday" => chrono::Weekday::Thu,
                    "friday" => chrono::Weekday::Fri,
                    "saturday" => chrono::Weekday::Sat,
                    "sunday" => chrono::Weekday::Sun,
                    _ => panic!("undefined day")
                };

                let td = TimeDay::new(time, day);
                println!("added {} to entry {}", &td, a.value_of("name").unwrap());
                p.times.push(td);
                plans.push(p);
                break
            }
        }
    }

    if a.is_present("remove time") {
        for i in 0..length {
            if plans.get(i).unwrap().name == a.value_of("name").unwrap() {
                let mut p = plans.get(i).unwrap().clone();
                plans.remove(i);

                let time = chrono::NaiveTime::parse_from_str(&a.value_of("time").unwrap(), "%H:%M").expect("date cannot be parsed");
                let day = match a.value_of("day").unwrap().to_lowercase().as_str() {
                    "monday" => chrono::Weekday::Mon,
                    "tuesday" => chrono::Weekday::Tue,
                    "wednesday" => chrono::Weekday::Wed,
                    "thursday" => chrono::Weekday::Thu,
                    "friday" => chrono::Weekday::Fri,
                    "saturday" => chrono::Weekday::Sat,
                    "sunday" => chrono::Weekday::Sun,
                    _ => panic!("undefined day")
                };

                let td = TimeDay::new(time, day);
                p.remove_matching_time(&td);
                println!("removed {} from entry {}", &td, a.value_of("name").unwrap());
                plans.push(p);
                break
            }
        }
    }

    if a.is_present("new name") {
        for i in 0..length {
            if plans.get(i).unwrap().name == a.value_of("name").unwrap() {
                let mut p = plans.get(i).unwrap().clone();
                plans.remove(i);
                p.name = a.value_of("new name").unwrap().to_string();
                plans.push(p);
                println!("changed name of entry {} to {}", a.value_of("name").unwrap(), a.value_of("new name").unwrap());
                break
            }
        }
    }

    export(plans, a.value_of("config").unwrap());
}

#[derive(Serialize, Deserialize, Clone)]
struct Plan {
    name: String,
    link: String,
    times: Vec<TimeDay>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
struct TimeDay {
    time: chrono::NaiveTime,
    day: chrono::Weekday,
}

impl TimeDay {
    fn new(t: chrono::NaiveTime, d: chrono::Weekday) -> Self {
        TimeDay {
            time: t,
            day: d,
        }
    }
}

impl fmt::Display for TimeDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "time: {}, day: {},\n", self.time, self.day)
    }
}

impl Plan {
    fn new(n: String, l: String, t: Vec<TimeDay>) -> Self {
        Plan {
            name: n,
            link: l,
            times: t,
        }
    }

    fn new_user_friendly(n: &str, l: &str, t: &str, d: &str) -> Self {
        let time = chrono::NaiveTime::parse_from_str(&t, "%H:%M").expect("date cannot be parsed");
        let day = match d.to_lowercase().as_str() {
            "monday" => chrono::Weekday::Mon,
            "tuesday" => chrono::Weekday::Tue,
            "wednesday" => chrono::Weekday::Wed,
            "thursday" => chrono::Weekday::Thu,
            "friday" => chrono::Weekday::Fri,
            "saturday" => chrono::Weekday::Sat,
            "sunday" => chrono::Weekday::Sun,
            _ => panic!("undefined day")
        };

        let mut tdv = Vec::new();
        tdv.push(TimeDay::new(time, day));
        
        Self::new(n.to_string(), l.to_string(), tdv)
    }

    fn remove_matching_time(&mut self, td: &TimeDay) {
        let length = self.times.len();
        for i in 0..length {
            if self.times.get(i).unwrap() == td {
                self.times.remove(i);
                break
            }
        }
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut times = String::new();
        for t in self.times.clone() {
            times += format!("{}", &t).as_str();
        }
        write!(f, "name: {},\n\ntimes:\n{}\nlink: {}\n", self.name, times, self.link)
    }
}

fn export<T: AsRef<Path>>(v: Vec<Plan>, p: T) {
    let mut file = OpenOptions::new().write(true).open(p).expect("error in opening config file"); //File::open(p).expect("error in opening config file");
    let _ = file.set_len(0);
    let _ = file.write(serde_json::to_string(&v).expect("error in serializing").as_bytes());
}

fn import<T: AsRef<Path>>(p: T) -> Vec<Plan> {
    let file = File::open(&p);
    let mut file = match file.is_err() {
        true => {
            create();
            File::open(&p).unwrap()
        },
        false => file.unwrap(),
    };

    let mut buf = Vec::new();
    let _ = file.read_to_end(&mut buf);
    if buf.len() == 0 {
        Vec::new()
    } else {
        serde_json::from_slice(&buf).expect("error while parsing config file")
    }
}

fn check(p: &Plan, td: &TimeDay) -> bool {
    let mut result = false;

    for t in &p.times {
        if td == t {
            open_link(&p.link);
            result |= true
        } else {
            result |= false
        }
    }

    result

}

fn check_all(v: &Vec<Plan>) {
    let mut cv = v.clone();
    loop { 
        let day = chrono::Local::now().naive_local().date().weekday();
        let time = chrono::Local::now().naive_local().time();
        let time = chrono::NaiveTime::from_hms(time.hour(), time.minute(), 0);
        let timeday = TimeDay::new(time, day);
        let length = cv.len();
        for i in 0..length {
            if check(cv.get(i).unwrap(), &timeday) {
                let mut p = cv.get(i).unwrap().clone();
                p.remove_matching_time(&timeday);
                cv.remove(i);
                cv.push(p);
                break
            }
        }

        let mut times = 0;
        for p in &cv {
            times += p.times.len();
        }
        if times == 0 {
            cv = v.clone()
        }
        std::thread::sleep(std::time::Duration::new(5, 0))
    }
    
}

#[cfg(target_os = "macos")]
fn open_link(z: &String) {
    Command::new("open").arg(z).spawn().expect("error in opening link using open");
}

#[cfg(all(not(target_os = "macos"), target_family = "unix"))]
fn open_link(z: &String) {
    Command::new("xdg-open").arg(z).spawn().expect("error in opening link using xdg-open");
}

#[cfg(target_os = "windows")]
fn open_link(z: &String) {
    Command::new("start").arg(z).spawn().expect("error in opening link using start");
}