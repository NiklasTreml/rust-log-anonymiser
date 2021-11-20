use core::time;
use std::{
    env, fs,
    fs::File,
    io::{Read, Write},
    ops::Add,
    path::Path,
    sync::mpsc,
    thread,
    time::{Duration, SystemTime},
};

use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use regex::Regex;
const OUTPUTPATH: &str = "./output";
fn main() {
    let script_start = SystemTime::now();
    let mut args: Vec<String> = env::args().collect();
    args.reverse();
    args.pop();
    args.reverse();
    let len = args.len();
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel();
    // delete output folder
    if Path::new(OUTPUTPATH).exists() {
        fs::remove_dir_all(OUTPUTPATH).expect("Failed to remove output folder");
        println!("Deleted {}", OUTPUTPATH);
    }

    let (barTx, barRx) = mpsc::channel();
    let pb = ProgressBar::new(len as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}]  [{pos}/{len}] {msg}",
            )
            .progress_chars("=>-"),
    );
    for arg in args {
        let sender = tx.clone();
        let barSender = barTx.clone();

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(5));
            let start = SystemTime::now();
            let patterns = vec![r"[Ee]nim", r"[Aa]met"];
            let replacers = vec!["ENIM_REPLACE", "AMET_REPLACE"];

            if anon_file(&arg, patterns, replacers) {
                let end = SystemTime::now();
                let duration = end.duration_since(start).unwrap();
                // println!("Finished for {}! Took {}ms", &arg, duration.as_millis());

                sender
                    .send(duration.as_micros())
                    .expect("Failed to send into channel");
                barSender.send(1);
            }
        });

        handles.push(handle)
    }
    drop(barTx);

    for received in barRx {
        if received == 1 {
            pb.inc(received);
        }
    }

    let mut durations: Vec<u128> = vec![];
    for handle in handles {
        handle.join().unwrap();
    }

    pb.finish_with_message("Done");
    drop(tx);

    for received in rx {
        durations.push(received);
    }

    let end = SystemTime::now();

    println!(
        "\n
-----------| Done!! |-----------
\tProcessed files: {}
\tTotal: {}ms
\tAverage speed: {}ms / file
--------------------------------",
        len,
        end.duration_since(script_start).unwrap().as_millis(),
        get_avg(durations)
    );
}

fn get_avg(durations: Vec<u128>) -> f32 {
    let mut total = 0;
    for duration in &durations {
        total += duration;
    }

    (total / (durations.len() as u128)) as f32 / 1000.0
}

fn anon_file(path: &String, patterns: Vec<&str>, replacers: Vec<&str>) -> bool {
    // read original file to variable

    if Path::new(path).is_dir() {
        // println!("Skipping folder {:?}...", path);
        return false;
    }
    let mut content = read_from_file(path);
    // for every regex => modify(&variable) <- in place
    for (i, pattern) in patterns.iter().enumerate() {
        modify(
            &mut content,
            &replacers[i],
            Regex::new(pattern).expect("Regex invalid"),
        );
    }
    // save_to_file(&variable)
    save_to_file(content, path);
    return true;
}

fn modify(content: &mut String, rep: &str, pattern: Regex) {
    let result = pattern.replace_all(&content, rep).to_string();
    *content = result;

    //println!("Replaced {} in {:?}", &pattern, new_filepath)
}

fn save_to_file(content: String, path: &String) {
    let old_path = Path::new(path);

    let new_filename = old_path
        .file_name()
        .expect("Could not get filename")
        .to_str()
        .expect("Could not convert old_path to str");
    let folders = old_path
        .parent()
        .expect("Could not get parents")
        .to_str()
        .expect("Could not convert folders to str");
    let string_path = format!("{}/{}/{}", OUTPUTPATH, folders, new_filename);

    let new_filepath = Path::new(&string_path);

    fs::create_dir_all(new_filepath.parent().unwrap()).expect("Could not create directory");

    let mut new_f = File::create(new_filepath).unwrap();
    new_f.write_all(content.as_bytes()).unwrap();
}
fn read_from_file(filepath: &String) -> String {
    let filepath = Path::new(&filepath);

    //println!("Running for {:?}", filepath);

    let mut f = File::open(filepath).expect("Could not open file.");
    let mut buf = String::new();
    f.read_to_string(&mut buf).expect("Could not read file");
    buf
}
