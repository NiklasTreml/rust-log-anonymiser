use std::{
    env, fs,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::mpsc::{self},
    thread,
    time::{Duration, SystemTime},
};

use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use threadpool::ThreadPool;
use walkdir::WalkDir;
fn main() {
    let mut args: Vec<String> = env::args().collect(); // collect cli args
    args.reverse(); // reverse the vector, pop the last item, reverse again
    args.pop(); // this is basically a more efficient way to remove the first item of an array
    args.reverse();
    /*
    the cli params are expected to be in the following order
    lAnon inputFolder outputFolder maxNumberOfThreads
    note that threads are equal to rust threads, so 1:1 threading model is used
    if the parameter is not passed, the program will either use two threads per logical cpu core
    or one per number of files that are found. Whichever one of these is less
    One can also use more than two threads per core.
    If the number is higher than the number of files only the threads necessary will be spawned though
     */
    let output_path = args[1].clone();
    let amnt_files: usize;

    // delete output folder
    let (bar_tx, bar_rx) = mpsc::channel();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            // this is just a spinner for the progress bar
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

    // the patterns and replacers have to be equal in length because
    // suppose i is our iterator over the patterns
    // every match of patterns[i] will be replaced by replacers[i]

    // to make this a bit more readable it could also be replaced by a hashmap
    /*
       {
           "[Ee]nim" : "ENIM_REPLACE",
           "[Aa]met": "AMET_REPLACE"
       }
    */

    let patterns = vec![r"[Ee]nim", r"[Aa]met"];
    let replacers = vec!["ENIM_REPLACE", "AMET_REPLACE"];

    if Path::new(&output_path).exists() {
        pb.set_message(format!("{} already exists", args[1]));
        thread::sleep(Duration::from_secs(1));

        pb.set_message(format!("Deleting {} ...", args[1]));
        fs::remove_dir_all(output_path).expect("Failed to remove output folder");

        pb.set_message(format!("Successfully deleted {}!", args[1]));
        thread::sleep(Duration::from_secs(1));
    }
    fs::create_dir(&args[1]).unwrap();
    pb.set_message(format!("Created {}!", &args[1]));

    pb.set_message("Getting to work...");
    pb.set_position(0);
    pb.enable_steady_tick(150);

    let mut file_paths = vec![];
    pb.set_message("Finding all files...");

    for entry in WalkDir::new(args[0].clone()) {
        // find all the files in the target folder and add them to a vector
        // we could also already start working on them here, but multithreading inside of recursive function is a mess
        // so we're doing that later
        let entry = match entry {
            Ok(i) => i,
            Err(_e) => return,
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
    amnt_files = file_paths.len();

    if num_threads > amnt_files {
        /*
            Prevent scheduling too many threads

            if we only find 10 files but we want to schedule 50 threads we will reduce them down to 10 since having more does not help us
            since we're using one thread per file
        */
        num_threads = amnt_files;
        pb.set_message(format!(
            "Reducing threads to {} because we will only work on {} files",
            num_threads, amnt_files
        ))
    }

    thread::sleep(Duration::from_secs(1)); // the sleeps are there so people are to read the output
    pb.set_message(format!("Running with {} threads", num_threads));
    thread::sleep(Duration::from_secs(1));
    let worker_pool = ThreadPool::new(num_threads); // were using a threadpool to limit the number of threads
                                                    // Refer to this for an explanation on thread pools https://doc.rust-lang.org/book/ch20-02-multithreaded.html#improving-throughput-with-a-thread-pool

    let script_start = SystemTime::now(); // record the starting time for calculating the total duration later
    pb.set_message(format!("Anonymizing {} files ...", amnt_files));

    for arg in file_paths {
        // Clone all the variables so that every thread has their own copy and no issues arise
        let bar_sender = bar_tx.clone();
        let moved_output = output.clone();

        let cpatterns = patterns.clone();
        let creplacers = replacers.clone();
        worker_pool.execute(move || {
            if anon_file(
                // this will return true if it has anonymized a file successfully or false, if it hits a directory
                Path::new(&arg),
                &cpatterns,
                &creplacers,
                moved_output.as_str(),
            ) {
                bar_sender.send(1).unwrap();
            }
        });
    }

    drop(bar_tx); // close the channel

    for received in bar_rx {
        if received == 1 {
            // pb.inc(received);
        }
    }

    pb.set_style(
        ProgressStyle::default_spinner()
            // For more spinners check out the cli-spinners project:
            // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
            .template("✓ {msg}"),
    );
    pb.finish_with_message("Done");

    let end = SystemTime::now(); // grab end time

    println!(
        "\n
-----------| Done!! |-----------
\tProcessed files: {}
\tTotal: {}ms
\tAverage speed: {:.2}ms / file
--------------------------------",
        amnt_files,
        end.duration_since(script_start).unwrap().as_millis(), // calculate time since start
        (end.duration_since(script_start).unwrap().as_millis() as f64 * num_threads as f64) // calculate the time taken per job using this formula which might be wrong 
            / amnt_files as f64  // (TotalTime * NumberOfWorkers) / NumberOfJobs
    );
}

fn anon_file(path: &Path, patterns: &[&str], replacers: &[&str], output_path: &str) -> bool {
    if path.is_dir() {
        // return if we hit a directory
        return false;
    }

    let file = File::open(path).expect("Could not open file"); // open the file
    let reader = BufReader::new(file); // create a reader

    for line in reader.lines() {
        /*  we're operating on the file on a line by line basis in order to reduce the used memory
        one could also read the whole file, but I have experienced bad performance and OoM crashes on some very big files
        turns out loading 10 2.5 GiB log file into RAM is a bad idea when youre only running on 8 GBs of RAM in a tiny laptop */
        let mut content = line.unwrap();

        for (i, pattern) in patterns.iter().enumerate() {
            // loop over the regexes and apply each one to the line
            // were using a reference to content here to mutate it in-place
            modify(
                &mut content,
                &replacers[i],
                Regex::new(pattern).expect("Regex invalid"),
            );
            // save_to_file(&variable)
        }
        // once we're done modifying the line we'll save it to its new location
        save_to_file(content.clone(), path, output_path);
    }
    true
}

fn modify(content: &mut String, rep: &str, pattern: Regex) {
    // replace every occurence of the given pattern in the givn string and mutate it in-place
    let result = pattern.replace_all(&content, rep).to_string();
    *content = result;
}

fn save_to_file(content: String, path: &Path, output_path: &str) {
    /*
        Generate the new path by just prepending the output path to the old path
        it would also be possible remove the old parent folder and replace it with the output folder
    */
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

    let string_path = format!("{}/{}/{}", output_path, folders, new_filename);

    let new_filepath = Path::new(&string_path);

    // create the folders if they dont yet exist
    fs::create_dir_all(new_filepath.parent().unwrap()).expect("Could not create directory");

    // open the new file and write to it by appending to it
    // we have to append because we're modifying the file original file line by line
    let mut new_f = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(new_filepath)
        .unwrap();

    writeln!(new_f, "{}", &content).unwrap(); // write to the new file
    drop(new_f);
}
