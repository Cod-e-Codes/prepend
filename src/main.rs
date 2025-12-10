use prepend::{parse_arguments, perform_prepend, validate_file};
use std::env;
use std::fs::File;
use std::io::Read;
use std::process;

// --- ANSI Colors ---
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let config = parse_arguments(&args).unwrap_or_else(|err| {
        eprintln!("{}ERROR:{} {}", RED, RESET, err);
        process::exit(1);
    });

    // Validate file
    if let Err(e) = validate_file(&config.filename) {
        eprintln!("{}ERROR:{} {}", RED, RESET, e);
        process::exit(1);
    }

    // Execution
    if config.dry_run {
        println!(
            "{}DRY-RUN MODE:{} The following would be written to {:?}:",
            YELLOW, RESET, config.filename
        );
        println!("----------------------------------------------");
        println!(
            "{}{}",
            config.prepend_text,
            if config.prepend_text.ends_with('\n') {
                ""
            } else {
                "\n"
            }
        );
        // In dry run, we just peek at the first few lines of the file to show context
        if let Ok(file) = File::open(&config.filename) {
            let mut handle = file.take(200); // Read only first 200 bytes for preview
            let mut buffer = String::new();
            if handle.read_to_string(&mut buffer).is_ok() {
                println!("{}... (Original Content) ...{}", buffer, RESET);
            }
        }
        println!("----------------------------------------------");
    } else {
        match perform_prepend(&config) {
            Ok(_) => println!(
                "{}SUCCESS:{} Text prepended to {:?}",
                GREEN, RESET, config.filename
            ),
            Err(e) => {
                eprintln!("{}FATAL ERROR:{} {}", RED, RESET, e);
                process::exit(1);
            }
        }
    }
}
