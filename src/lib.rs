use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

// --- ANSI Colors ---
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

// --- Configuration ---
const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "log", "md", "sh", "conf", "yaml", "json", "csv", "cfg", "ini", "c", "cpp", "h", "py",
    "js", "rs",
];
const BUFFER_SIZE: usize = 64 * 1024; // 64KB Buffer for large files

pub struct Config {
    pub filename: PathBuf,
    pub prepend_text: String,
    pub dry_run: bool,
}

pub fn parse_arguments(args: &[String]) -> Result<Config, String> {
    let mut filename = None;
    let mut text_arg = None;
    let mut dry_run = false;
    let mut show_help = false;

    // Skip executable name
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--help" | "-h" => show_help = true,
            _ => {
                if filename.is_none() {
                    filename = Some(PathBuf::from(arg));
                } else if text_arg.is_none() {
                    text_arg = Some(arg.clone());
                }
            }
        }
    }

    if show_help || filename.is_none() {
        print_help(&args[0]);
        process::exit(0);
    }

    let target_file = filename.unwrap();
    let final_text;

    if let Some(txt) = text_arg {
        // Mode 2: Argument
        final_text = format!("{}\n", txt); // Ensure newline
    } else {
        // Mode 1: Interactive
        println!(
            "{}Prepend Tool:{} Ready to process {:?}",
            BLUE, RESET, target_file
        );
        println!(
            "Enter text to prepend (Press {}Ctrl+D{} on a new line to finish):",
            YELLOW, RESET
        );
        println!("----------------------------------------------");
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| e.to_string())?;
        if buffer.trim().is_empty() {
            return Err("Input text is empty.".to_string());
        }
        // Ensure the input ends with a newline so it doesn't merge with the first line of the file
        if !buffer.ends_with('\n') {
            buffer.push('\n');
        }
        final_text = buffer;
    }

    Ok(Config {
        filename: target_file,
        prepend_text: final_text,
        dry_run,
    })
}

pub fn validate_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("File {:?} does not exist.", path));
    }
    if !path.is_file() {
        return Err(format!("{:?} is not a regular file.", path));
    }

    // Permission check (basic write check)
    if OpenOptions::new().write(true).open(path).is_err() {
        return Err(format!("File {:?} is not writable.", path));
    }

    // Extension check
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        let ext_lower = ext.to_lowercase();
        if !ALLOWED_EXTENSIONS.contains(&ext_lower.as_str()) {
            println!(
                "{}WARNING:{} Uncommon extension '.{}'. Proceeding...",
                YELLOW, RESET, ext
            );
        }
    }

    Ok(())
}

pub fn perform_prepend(config: &Config) -> io::Result<()> {
    let source_path = &config.filename;

    // Create a temp file in the SAME DIRECTORY as the source.
    // This is crucial for atomic moves across filesystems.
    let mut temp_path = source_path.clone();
    temp_path.set_extension("tmp_prepend");

    let source_file = File::open(source_path)?;
    let temp_file = File::create(&temp_path)?;

    // Use Buffering for speed
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, source_file);
    let mut writer = BufWriter::with_capacity(BUFFER_SIZE, temp_file);

    // 1. Write the new header
    writer.write_all(config.prepend_text.as_bytes())?;

    // 2. Stream the original file content
    io::copy(&mut reader, &mut writer)?;

    // 3. Flush to ensure all data is on disk
    writer.flush()?;

    // 4. Atomic Replace
    // fs::rename is atomic on POSIX systems if on the same mount point
    match fs::rename(&temp_path, source_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Cleanup temp file if rename fails
            let _ = fs::remove_file(&temp_path);
            Err(e)
        }
    }
}

pub fn print_help(prog_name: &str) {
    println!(
        "{}Usage:{} {} [OPTIONS] <filename> [text]",
        BLUE, RESET, prog_name
    );
    println!("\nSafely prepends text to the beginning of a file using buffering.");
    println!("\n{}Options:{}", BLUE, RESET);
    println!("  --dry-run   Show what would happen without modifying the file.");
    println!("  --help      Show this message.");
}
