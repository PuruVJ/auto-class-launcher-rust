use chrono::{Datelike, Timelike, Weekday};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::{thread, time};
use urlencoding::encode;
use webbrowser::open;

#[derive(Debug, Serialize, Deserialize)]
struct ClassTime {
    day: String,
    time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Class {
    link: Option<String>,
    times: Vec<ClassTime>,
}

type ClassConfig = HashMap<String, Class>;

#[derive(Debug)]
struct ClassToday {
    name: String,
    link: Option<String>,
    time: chrono::DateTime<chrono::Local>,
}

fn set_interval<F>(mut callback: F, seconds: u64) -> thread::JoinHandle<()>
where
    F: FnMut() -> (),
    F: Send + 'static,
{
    thread::spawn(move || loop {
        thread::sleep(time::Duration::from_secs(seconds));
        callback();
    })
}

fn get_time_from_hh_mm(hh_mm: &str) -> chrono::DateTime<chrono::Local> {
    let mut hh_mm = hh_mm.split(":");
    let hh = hh_mm.next().unwrap().parse::<u32>().unwrap();
    let mm = hh_mm.next().unwrap().parse::<u32>().unwrap();

    chrono::offset::Local::now()
        .with_hour(hh)
        .unwrap()
        .with_minute(mm)
        .unwrap()
        .with_second(0)
        .unwrap()
}

fn get_classes_today(
    config: &ClassConfig,
    weekday: String,
    todays_class_launched: &mut HashMap<String, bool>,
) -> Vec<ClassToday> {
    let mut classes_today = Vec::new();

    for (class_name, class) in config {
        for time in &class.times {
            if time.day == *weekday {
                classes_today.push(ClassToday {
                    name: class_name.to_string(),
                    link: class.link.clone(),
                    time: get_time_from_hh_mm(&time.time),
                });
            }
        }
    }

    // Also set the todays_class_launched here
    for class in &classes_today {
        todays_class_launched.insert(class.name.clone(), false);
    }

    classes_today.sort_by(|a, b| a.time.cmp(&b.time));
    classes_today
}

fn open_class_link(config: &ClassConfig, todays_class_launched: &mut HashMap<String, bool>) {
    let date = chrono::offset::Local::now();
    let weekday = date.weekday();
    let day_str = match weekday {
        Weekday::Mon => "mon",
        Weekday::Tue => "tue",
        Weekday::Wed => "wed",
        Weekday::Thu => "thu",
        Weekday::Fri => "fri",
        Weekday::Sat => "sat",
        Weekday::Sun => "sun",
    };

    let classes_today = get_classes_today(config, day_str.to_lowercase(), todays_class_launched);

    // Get the next upcoming class. The class just after current time is the target class
    let next_class = classes_today.iter().find(|c| c.time > date);

    // Check if all classes have been launched
    let mut all_classes_launched = true;
    for (_class_name, launched) in todays_class_launched.clone() {
        if !launched {
            all_classes_launched = false;
        }
    }

    if next_class.is_none() || all_classes_launched {
        println!(
            "{}",
            "No more classes for today 🥳🥳🥳. Feel free to close this window.".yellow()
        );
        return;
    }

    let launch_time = next_class.unwrap().time - chrono::Duration::minutes(5);

    let mut minutes_str = next_class.unwrap().time.minute().to_string();
    if minutes_str.len() == 1 {
        minutes_str = "0".to_string() + &minutes_str;
    }

    if !todays_class_launched
        .get(&next_class.unwrap().name)
        .unwrap()
    {
        println!(
            "{}",
            format!(
                "[RUNNING] Launching {} at {}:{}",
                next_class.unwrap().name.cyan(),
                next_class.unwrap().time.hour(),
                minutes_str
            )
            .blue()
        );
    }

    dbg!(todays_class_launched.clone());
    if date > launch_time
        && !todays_class_launched
            .get(&next_class.unwrap().name)
            .unwrap()
    {
        let mut link = next_class.unwrap().link.clone();
        if !link.is_some() {
            link = Some(String::from(format!(
                "https://auto-class-launcher-alarm.vercel.app/?className={}&timing={}:{}",
                encode(&next_class.unwrap().name),
                &next_class.unwrap().time.hour().to_string(),
                &minutes_str
            )));
        }

        open(link.unwrap().as_str()).unwrap();

        todays_class_launched.insert(next_class.unwrap().name.clone(), true);
        // dbg!(next_class.unwrap().name.clone());
    }
}

fn main() {
    let mut todays_class_launched: HashMap<String, bool> = HashMap::new();

    let default_config_str = include_str!("sample.json");
    let default_config: ClassConfig = serde_json::from_str(default_config_str).unwrap();

    let config_path = "./auto-class-launcher-timetable.json";

    let config: ClassConfig;

    let file_operation = fs::read_to_string(config_path);
    if file_operation.is_ok() {
        let file_contents = file_operation.unwrap();
        config = serde_json::from_str(&file_contents).unwrap();
    } else {
        config = default_config;

        // Create the file
        fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    }

    set_interval(
        move || open_class_link(&config, &mut todays_class_launched),
        1,
    )
    .join()
    .unwrap();
}
