extern crate serde;
extern crate chrono;
extern crate url;
extern crate serde_json;
extern crate clap;
use std::process::Command;
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
 * Add Time and Day into another struct called Time and replace the time field of the Plan struct with a vector of Time.
*/
fn main() {
    let homedir = env::var("HOME").expect("error in getting home directory");
    let mut schedule_dir = PathBuf::new();
    schedule_dir.push(&homedir);
    schedule_dir.push("autozoom-schedule.json");
    let schedule_dir = schedule_dir.to_str().unwrap();
    let app = clap::App::new("autozoom")
                            .version("1.0")
                            .author("Oğuz Kurt <kurt.oguz@outlook.com>")
                            .about("when the time comes, opens Zoom meetings automatically")
                            .subcommand(clap::SubCommand::with_name("start")
                                            .about("start the application")
                                            .arg(clap::Arg::with_name("schedule")
                                                .help("JSON formatted schedule file")
                                                .long("schedule")
                                                .short("s")
                                                .default_value(&schedule_dir)))
                            .subcommand(clap::SubCommand::with_name("list")
                                            .arg(clap::Arg::with_name("schedule")
                                                .help("JSON formatted schedule file")
                                                .long("schedule")
                                                .short("s")
                                                .default_value(&schedule_dir)))
                            .subcommand(clap::SubCommand::with_name("add")
                                            .arg(clap::Arg::with_name("name")
                                                    .help("the name of the Zoom")
                                                    .index(1)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("link")
                                                    .help("the link of the Zoom")
                                                    .index(2)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("time")
                                                    .help("the time of the Zoom in the format hour:minutes (example: 22:10)")
                                                    .index(3)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("day")
                                                    .help("the day of the week the Zoom takes place (Monday, Tuesday, Wednesday, etc.)")
                                                    .index(4)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("schedule")
                                                    .help("JSON formatted schedule file")
                                                    .long("schedule")
                                                    .short("s")
                                                    .default_value(&schedule_dir)))
                            .subcommand(clap::SubCommand::with_name("remove")
                                            .arg(clap::Arg::with_name("name")
                                                    .help("name of the Zoom meeting to be removed")
                                                    .index(1)
                                                    .required(true))
                                            .arg(clap::Arg::with_name("schedule")
                                                    .help("JSON formatted schedule file")
                                                    .long("schedule")
                                                    .short("s")
                                                    .default_value(&schedule_dir)))
                            .subcommand(clap::SubCommand::with_name("launch")
                                            .arg(clap::Arg::with_name("name")
                                                .help("name of the Zoom meeting to be launched")
                                                .index(1)
                                                .required(true))
                                            .arg(clap::Arg::with_name("schedule")
                                                .help("JSON formatted schedule file")
                                                .long("schedule")
                                                .short("s")
                                                .default_value(&schedule_dir)))
                            .subcommand(clap::SubCommand::with_name("create"))
                            .get_matches();

    match app.subcommand() {
        ("start", Some(args)) => start(args),
        ("list", Some(args)) => list(args),
        ("add", Some(args)) => add(args),
        ("remove", Some(args)) => remove(args),
        ("launch", Some(args)) => launch(args),
        ("create", Some(args)) => create(args),
        _ => println!("use subcommand help for usage")
    }
}

fn start(a: &clap::ArgMatches) {
    let mut plans = import(a.value_of("schedule").expect("error in opening schedule file (use create command to create if not)"));
    if plans.len() == 0 {
        panic!("no Zoom meetings set")
    }
    check_all(&mut plans);
}

fn list(a: &clap::ArgMatches) {
    let plans = import(a.value_of("schedule").expect("error in opening schedule file (use create command to create if not)"));
    println!("meetings:");
    for plan in plans {
        println!("{}", plan);
    }
}

fn add(a: &clap::ArgMatches) {
    let mut plans = import(a.value_of("schedule").expect("error in opening schedule file (use create command to create if not)"));
    let new_plan = Plan::new_user_friendly(a.value_of("name").unwrap(), a.value_of("link").unwrap(), a.value_of("time").unwrap(), a.value_of("day").unwrap());
    plans.push(new_plan);
    export(plans, a.value_of("schedule").unwrap());
}

fn remove(a: &clap::ArgMatches) {
    let mut plans = import(a.value_of("schedule").expect("error in opening schedule file (use create command to create if not)"));
    for i in 0..plans.len() {
        if plans.get(i).unwrap().name == a.value_of("name").unwrap() {
            plans.remove(i);
            break
        }
    }
}

fn launch(a: &clap::ArgMatches) {
    let plans = import(a.value_of("schedule").expect("error in opening schedule file (use create command to create if not)"));
    for plan in plans {
        if plan.name == a.value_of("name").unwrap() {
            /*
            open_zoom_link(
                https_to_zoommtg(
                    url::Url::parse(&plan.zoom_link)
                    .expect("link parsing error")
                )
            )
            */
            open_link(&plan.zoom_link);
        }
    }
}

fn create(a: &clap::ArgMatches) {
    let homedir = env::var("HOME").expect("error in getting home directory");
    let _ = File::create(homedir + "/autozoom-schedule.json");
}

#[derive(Serialize, Deserialize, Clone)]
struct Plan {
    name: String,
    zoom_link: String,
    time: chrono::NaiveTime,
    day: chrono::Weekday,
}

impl Plan {
    fn new(n: String, l: String, t: chrono::NaiveTime, d: chrono::Weekday) -> Self {
        Plan {
            name: n,
            zoom_link: l,
            time: t,
            day: d
        }
    }

    fn new_user_friendly(n: &str, l: &str, t: &str, d: &str) -> Self {
        let time = chrono::NaiveTime::parse_from_str(&t, "%H:%M").expect("date cannot be parsed");
        let day = match d {
            "Monday" => chrono::Weekday::Mon,
            "Tuesday" => chrono::Weekday::Tue,
            "Wednesday" => chrono::Weekday::Wed,
            "Thursday" => chrono::Weekday::Thu,
            "Friday" => chrono::Weekday::Fri,
            "Saturday" => chrono::Weekday::Sat,
            "Sunday" => chrono::Weekday::Sun,
            _ => panic!("undefined day")
        };
        
        Self::new(n.to_string(), l.to_string(), time, day)
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name: {}, day: {}, time: {}, zoom link: {}", self.name, self.day, self.time, self.zoom_link)
    }
}

fn export<T: AsRef<Path>>(v: Vec<Plan>, p: T) {
    let mut file = OpenOptions::new().write(true).open(p).expect("error in opening schedule file"); //File::open(p).expect("error in opening schedule file");
    let s = file.write(serde_json::to_string(&v).expect("error in serializing").as_bytes());
}

fn import<T: AsRef<Path>>(p: T) -> Vec<Plan> {
    let mut file = File::open(p).expect("error in opening schedule file");
    let mut buf = Vec::new();
    let _ = file.read_to_end(&mut buf);
    if buf.len() == 0 {
        Vec::new()
    } else {
        serde_json::from_slice(&buf).expect("error while parsing schedule file")
    }
}

fn check(p: &Plan) -> bool {
    let day = chrono::Local::now().naive_local().date().weekday();
    let time = chrono::Local::now().naive_local().time();
    let time = chrono::NaiveTime::from_hms(time.hour(), time.minute(), 0);

    if day == p.day && time == p.time {
        /*
        open_zoom_link(
            https_to_zoommtg(
                url::Url::parse(&p.zoom_link)
                .expect("link parsing error")
            )
        );
        */
        open_link(&p.zoom_link);
        true
    } else {
        false
    }
}

fn check_all(v: &mut Vec<Plan>) {
    let mut cv = v.clone();
    loop { 
        for i in 0..cv.len() {
            if check(v.get(i).unwrap()) {
                cv.remove(i);
                break
            }
        }   
        if cv.is_empty() {
            cv = v.clone();
        }
        std::thread::sleep(std::time::Duration::new(1, 0))
    }
    
}

#[cfg(target_os = "macos")]
fn open_link(z: &String) {
    Command::new("open").arg(z).spawn().expect("error in opening Zoom using open");
}

#[cfg(all(not(target_os = "macos"), target_family = "unix"))]
fn open_link(z: &String) {
    Command::new("xdg-open").arg(z).spawn().expect("error in opening Zoom using xdg-open");
}

#[cfg(target_os = "windows")]
fn open_link(z: &String) {
    Command::new("start").arg(z).spawn().expect("error in opening Zoom using xdg-open");
}

/*
#[cfg(target_os = "macos")]
fn open_zoom_link(z: String) {
    Command::new("open").arg(z).spawn().expect("error in opening Zoom using open");
}

#[cfg(all(not(target_os = "macos"), target_family = "unix"))]
fn open_zoom_link(z: String) {
    Command::new("xdg-open").arg(z).spawn().expect("error in opening Zoom using xdg-open");
}

#[cfg(target_os = "windows")]
fn open_zoom_link(z: String) {
    Command::new("start").arg(z).spawn().expect("error in opening Zoom using xdg-open");
}
*/
/*
fn https_to_zoommtg(u: url::Url) -> String {
    const SCHEME: &str = "zoommtg://";
    const CONF_NO: &str = "?confno=";
    // const PWD: &str = "?pwd=";
    let mut path_segments = u.path_segments().expect("error in link");
    path_segments.next();
    let conference_number = path_segments.next().expect("error in link");
    SCHEME.to_string() + u.host_str().unwrap() + "/join" + CONF_NO + conference_number + "?" + u.query().expect("error in resolving password from link")
}
*/