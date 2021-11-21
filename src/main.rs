use core::time;
use std::{
    env, fs,
    fs::{DirEntry, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    ops::Add,
    path::Path,
    sync::{
        mpsc::{self, Sender},
        Arc, Barrier,
    },
    thread,
    time::{Duration, SystemTime},
};

use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use regex::Regex;
use threadpool::ThreadPool;
use walkdir::WalkDir;
fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.reverse();
    args.pop();
    args.reverse();
    let OUTPUTPATH = args[1].clone();
    let mut len: usize = 0;

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
    let mut num_threads = num_cpus::get() * 2;

    if args.len() >= 3 {
        num_threads = args[2]
            .parse::<usize>()
            .expect("Enter an integer for n_thread");
    }

    let output = &args[1];
    len = file_paths.len();

    thread::sleep(Duration::from_secs(1));
    pb.set_message(format!("Running with {} threads", num_threads));
    thread::sleep(Duration::from_secs(1));
    let worker_pool = ThreadPool::new(num_threads);
    // let barrier = Arc::new(Barrier::new(len + 1));

    let script_start = SystemTime::now();
    pb.set_message("Getting to work...");

    for arg in file_paths {
        let sender = tx.clone();
        let barSender = barTx.clone();
        let movedOutput = output.clone();
        // let barrier = barrier.clone();
        worker_pool.execute(move || {
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
    }

    drop(barTx);

    for received in barRx {
        if received == 1 {
            // pb.inc(received);
        }
    }

    let mut durations: Vec<u128> = vec![];
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
    // let mut content = read_from_file(path);

    // for every regex => modify(&variable) <- in place

    let file = File::open(path).expect("Could not open file");
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let mut content = line.unwrap();

        for (i, pattern) in patterns.iter().enumerate() {
            modify(
                &mut content,
                &replacers[i],
                Regex::new(pattern).expect("Regex invalid"),
            );
            // save_to_file(&variable)
        }
        save_to_file(content.clone(), path, outputPath);
    }
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

    let mut new_f = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(new_filepath)
        .unwrap();

    writeln!(new_f, "{}", &content).unwrap();
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
