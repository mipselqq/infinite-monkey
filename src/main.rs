use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{
    collections::HashSet,
    process::{exit, Command},
    thread,
    time::Duration,
};

const SRC_TEXT: &str = include_str!("../war-and-peace.txt");
const TERM_SHOWCASE: bool = false;
const FILE_LOG_SHOWCASE: bool = true;
const MAX_THREADS: Option<usize> = None;

fn main() {
    let is_done = Arc::new(AtomicBool::new(false));

    let mut handles = Vec::new();
    let threads_count = match MAX_THREADS {
        Some(max_threads) => max_threads,
        None => num_cpus::get(),
    };

    for thread_id in 1..=threads_count {
        let is_done = Arc::clone(&is_done);

        handles.push(thread::spawn(move || {
            start_guessing(&format!("thread{thread_id}_log.txt"), is_done);
        }))
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn start_guessing(log_filename: &str, is_done: Arc<AtomicBool>) -> ! {
    if SRC_TEXT.is_empty() {
        panic!("Empty file")
    }

    if FILE_LOG_SHOWCASE {
        fs::remove_file(log_filename).unwrap_or_default();
    }

    let all_chars = parse_all_chars(SRC_TEXT);
    let mut text_chars = SRC_TEXT.chars().enumerate().peekable();

    if TERM_SHOWCASE {
        write_stats(&all_chars);
    }

    let mut longest_sequence_len = 0;

    loop {
        if is_done.load(Ordering::Relaxed) {
            println!("Thread detected completion, exiting...");
            exit(0);
        }

        if let Some(&(i, text_char)) = text_chars.peek() {
            let random_index = fastrand::usize(0..all_chars.len());
            let random_char = all_chars[random_index];

            if random_char == text_char {
                if FILE_LOG_SHOWCASE && i + 1 > longest_sequence_len {
                    append_log(log_filename, &mut longest_sequence_len, random_char);
                }

                text_chars.next();

                if TERM_SHOWCASE {
                    print_guessed_char_wait_for(random_char, i, 200);
                }
            } else if i != 0 {
                if TERM_SHOWCASE {
                    clear_term();
                };

                text_chars = SRC_TEXT.chars().enumerate().peekable();
            }

            continue;
        }

        println!(
            "\nSuccess! A hypothetical monkey has written the whole text, theorem is confirmed!"
        );
        is_done.store(true, Ordering::Relaxed);
        exit(0);
    }
}

fn print_guessed_char_wait_for(random_char: char, i: usize, delay: u64) {
    print!("{}", random_char);
    io::stdout().flush().unwrap();

    if i >= 1 {
        thread::sleep(Duration::from_millis(delay));
    }
}

fn append_log(filename: &str, longest_sequence_len: &mut usize, random_char: char) {
    *longest_sequence_len += 1;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filename)
        .expect("Failed to open log file");

    write!(file, "{}", random_char).expect("Failed to write to log file");
}

fn write_stats(all_chars: &[char]) {
    let unique_symbols = all_chars.len();
    let overall_symbols = SRC_TEXT.chars().collect::<Vec<char>>().len();
    let single_guess_probability = 1.0 / unique_symbols as f64;
    let each_guess_probability = single_guess_probability.powi(overall_symbols as i32);
    let each_guess_probability_str = &format!("{}%", each_guess_probability);

    println!(
        "Unique symbols: {}\nOverall symbols: {}\nSingle guess probability: {}%\nEach guess probability: {}",
        unique_symbols,
        overall_symbols,
        single_guess_probability * 100.0,
        if each_guess_probability == 0.0 { "approaches 0 (really small)" } else { each_guess_probability_str }
    );

    thread::sleep(Duration::from_millis(5000));
    clear_term();
}

fn clear_term() {
    if cfg!(target_os = "windows") {
        let _ = Command::new("cmd")
            .args(["/C", "echo \x1b[2J\x1b[H"])
            .status();
    } else {
        print!("\x1b[2J\x1b[H");
    }
    std::io::stdout().flush().expect("Failed to flush stdout");
}

fn parse_all_chars(str: &str) -> Vec<char> {
    let mut chars = HashSet::<char>::new();

    for char in str.chars() {
        chars.insert(char);
    }

    chars.into_iter().collect()
}
