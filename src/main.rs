use core::time;
use std::{
    env, fs,
    fs::{DirEntry, File},
    io::{Read, Write},
    ops::Add,
    path::Path,
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, SystemTime},
};

use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use regex::Regex;
use walkdir::WalkDir;
fn main() {
    let script_start = SystemTime::now();
    let mut args: Vec<String> = env::args().collect();
    args.reverse();
    args.pop();
    args.reverse();
    let OUTPUTPATH = args[1].clone();
    let mut len: usize = 0;
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel();
    // delete output folder
    let (barTx, barRx) = mpsc::channel();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .tick_strings(&[
                "⠀", "⠁", "⠂", "⠃", "⠄", "⠅", "⠆", "⠇", "⡀", "⡁", "⡂", "⡃", "⡄", "⡅", "⡆", "⡇",
                "⠈", "⠉", "⠊", "⠋", "⠌", "⠍", "⠎", "⠏", "⡈", "⡉", "⡊", "⡋", "⡌", "⡍", "⡎", "⡏",
                "⠐", "⠑", "⠒", "⠓", "⠔", "⠕", "⠖", "⠗", "⡐", "⡑", "⡒", "⡓", "⡔", "⡕", "⡖", "⡗",
                "⠘", "⠙", "⠚", "⠛", "⠜", "⠝", "⠞", "⠟", "⡘", "⡙", "⡚", "⡛", "⡜", "⡝", "⡞", "⡟",
                "⠠", "⠡", "⠢", "⠣", "⠤", "⠥", "⠦", "⠧", "⡠", "⡡", "⡢", "⡣", "⡤", "⡥", "⡦", "⡧",
                "⠨", "⠩", "⠪", "⠫", "⠬", "⠭", "⠮", "⠯", "⡨", "⡩", "⡪", "⡫", "⡬", "⡭", "⡮", "⡯",
                "⠰", "⠱", "⠲", "⠳", "⠴", "⠵", "⠶", "⠷", "⡰", "⡱", "⡲", "⡳", "⡴", "⡵", "⡶", "⡷",
                "⠸", "⠹", "⠺", "⠻", "⠼", "⠽", "⠾", "⠿", "⡸", "⡹", "⡺", "⡻", "⡼", "⡽", "⡾", "⡿",
                "⢀", "⢁", "⢂", "⢃", "⢄", "⢅", "⢆", "⢇", "⣀", "⣁", "⣂", "⣃", "⣄", "⣅", "⣆", "⣇",
                "⢈", "⢉", "⢊", "⢋", "⢌", "⢍", "⢎", "⢏", "⣈", "⣉", "⣊", "⣋", "⣌", "⣍", "⣎", "⣏",
                "⢐", "⢑", "⢒", "⢓", "⢔", "⢕", "⢖", "⢗", "⣐", "⣑", "⣒", "⣓", "⣔", "⣕", "⣖", "⣗",
                "⢘", "⢙", "⢚", "⢛", "⢜", "⢝", "⢞", "⢟", "⣘", "⣙", "⣚", "⣛", "⣜", "⣝", "⣞", "⣟",
                "⢠", "⢡", "⢢", "⢣", "⢤", "⢥", "⢦", "⢧", "⣠", "⣡", "⣢", "⣣", "⣤", "⣥", "⣦", "⣧",
                "⢨", "⢩", "⢪", "⢫", "⢬", "⢭", "⢮", "⢯", "⣨", "⣩", "⣪", "⣫", "⣬", "⣭", "⣮", "⣯",
                "⢰", "⢱", "⢲", "⢳", "⢴", "⢵", "⢶", "⢷", "⣰", "⣱", "⣲", "⣳", "⣴", "⣵", "⣶", "⣷",
                "⢸", "⢹", "⢺", "⢻", "⢼", "⢽", "⢾", "⢿", "⣸", "⣹", "⣺", "⣻", "⣼", "⣽", "⣾", "⣿",
            ])
            .template("{spinner:.green} {msg}"),
    );

    let patterns = vec![r"[Ee]nim", r"[Aa]met"];
    let replacers = vec!["ENIM_REPLACE", "AMET_REPLACE"];

    if Path::new(&OUTPUTPATH).exists() {
        pb.set_message(format!("{} already exists", args[1]));
        thread::sleep(Duration::from_secs(1));

        pb.set_message(format!("Deleting {} ...", args[1]));
        fs::remove_dir_all(OUTPUTPATH).expect("Failed to remove output folder");

        pb.set_message(format!("Successfully deleted {}!", args[1]));
        thread::sleep(Duration::from_secs(1));
    }
    fs::create_dir(&args[1]);
    pb.set_message(format!("Created {}!", &args[1]));

    pb.set_message("Getting to work...");
    pb.set_position(0);
    pb.enable_steady_tick(150);

    let mut file_paths = vec![];
    pb.set_message("Finding all files...");

    for entry in WalkDir::new(args[0].clone()) {
        let entry = match entry {
            Ok(i) => i,
            Err(e) => return (),
        };
        let path = entry.path();
        if path.is_file() {
            file_paths.push(path.display().to_string());
            pb.set_message(format!("Found {} paths", file_paths.len()));
        }
    }

    let output = &args[1];
    len = file_paths.len();

    thread::sleep(Duration::from_secs(1));
    pb.set_message("Getting to work...");
    for arg in file_paths {
        let sender = tx.clone();
        let barSender = barTx.clone();
        let movedOutput = output.clone();

        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_secs(5));
            let start = SystemTime::now();
            let patterns = vec![r"[Ee]nim", r"[Aa]met"];
            let replacers = vec!["ENIM_REPLACE", "AMET_REPLACE"];
            // let output = "./output";
            if anon_file(Path::new(&arg), &patterns, &replacers, movedOutput.as_str()) {
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
            // pb.inc(received);
        }
    }

    let mut durations: Vec<u128> = vec![];
    for handle in handles {
        handle.join().unwrap();
    }

    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .template("✓ {msg}"),
    );
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
        end.duration_since(script_start).unwrap().as_millis() / len as u128 // this is technically wrong since doing things in parallel, but I'm lazy
    );
}

fn get_avg(durations: Vec<u128>) -> f32 {
    let mut total = 0;
    for duration in &durations {
        total += duration;
    }

    (total / (durations.len() as u128)) as f32 / 1000.0
}

fn anon_file(path: &Path, patterns: &Vec<&str>, replacers: &Vec<&str>, outputPath: &str) -> bool {
    // read original file to variable

    if path.is_dir() {
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
    save_to_file(content, path, outputPath);
    return true;
}

fn modify(content: &mut String, rep: &str, pattern: Regex) {
    let result = pattern.replace_all(&content, rep).to_string();
    *content = result;

    //println!("Replaced {} in {:?}", &pattern, new_filepath)
}

fn save_to_file(content: String, path: &Path, outputPath: &str) {
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
    let string_path = format!("{}/{}/{}", outputPath, folders, new_filename);

    let new_filepath = Path::new(&string_path);

    fs::create_dir_all(new_filepath.parent().unwrap()).expect("Could not create directory");

    let mut new_f = File::create(new_filepath).unwrap();
    new_f.write_all(content.as_bytes()).unwrap();
    drop(new_f);
}
fn read_from_file(filepath: &Path) -> String {
    //println!("Running for {:?}", filepath);

    let mut f = File::open(filepath).expect("Could not open file.");
    let mut buf = String::new();
    f.read_to_string(&mut buf).expect("Could not read file");
    drop(f);
    buf
}
