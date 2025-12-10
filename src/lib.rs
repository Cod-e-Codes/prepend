//! A library for safely prepending text to files.
//!
//! This library provides functionality to prepend text to the beginning of files
//! using buffered I/O and atomic file operations to ensure data safety.

pub mod constants;
pub mod error;

use constants::{ALLOWED_EXTENSIONS, BLUE, BUFFER_SIZE, RESET, YELLOW};
use error::PrependError;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process;

/// Configuration for the prepend operation.
///
/// Contains all the parameters needed to perform a prepend operation,
/// including the target file, text to prepend, and execution mode.
pub struct Config {
    /// Path to the file to be modified
    pub filename: PathBuf,
    /// Text to prepend to the file
    pub prepend_text: String,
    /// If true, show what would happen without modifying the file
    pub dry_run: bool,
}

/// Parses command-line arguments into a configuration.
///
/// # Arguments
///
/// * `args` - Command-line arguments including the program name
///
/// # Returns
///
/// * `Ok(Config)` - Successfully parsed configuration
/// * `Err(PrependError)` - Error parsing arguments or reading input
///
/// # Modes
///
/// - **Interactive mode**: If only filename is provided, prompts for text input
/// - **Argument mode**: If filename and text are provided, uses the text argument
///
/// # Examples
///
/// ```no_run
/// use prepend::parse_arguments;
/// let args = vec!["prepend".to_string(), "file.txt".to_string()];
/// let config = parse_arguments(&args).unwrap();
/// ```
pub fn parse_arguments(args: &[String]) -> Result<Config, PrependError> {
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
        io::stdin().read_to_string(&mut buffer)?;
        if buffer.trim().is_empty() {
            return Err(PrependError::EmptyInput);
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

/// Validates that a file exists, is a regular file, and is writable.
///
/// # Arguments
///
/// * `path` - Path to the file to validate
///
/// # Returns
///
/// * `Ok(())` - File is valid and ready for prepending
/// * `Err(PrependError)` - File validation failed
///
/// # Warnings
///
/// Prints a warning to stdout if the file has an uncommon extension,
/// but does not fail validation.
pub fn validate_file(path: &Path) -> Result<(), PrependError> {
    if !path.exists() {
        return Err(PrependError::FileNotFound(format!("{:?}", path)));
    }
    if !path.is_file() {
        return Err(PrependError::NotAFile(format!("{:?}", path)));
    }

    // Permission check (basic write check)
    if OpenOptions::new().write(true).open(path).is_err() {
        return Err(PrependError::NotWritable(format!("{:?}", path)));
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

/// Performs the prepend operation on a file.
///
/// This function safely prepends text to a file using the following strategy:
/// 1. Creates a temporary file in the same directory
/// 2. Writes the prepend text to the temporary file
/// 3. Streams the original file content to the temporary file
/// 4. Atomically replaces the original file with the temporary file
///
/// # Arguments
///
/// * `config` - Configuration containing the file path and text to prepend
///
/// # Returns
///
/// * `Ok(())` - Prepend operation completed successfully
/// * `Err(PrependError)` - I/O error occurred during the operation
///
/// # Safety
///
/// This function uses atomic file operations to minimize the risk of data loss.
/// If the operation fails, the temporary file is cleaned up automatically.
pub fn perform_prepend(config: &Config) -> Result<(), PrependError> {
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
            Err(PrependError::Io(e))
        }
    }
}

/// Prints help information for the command-line tool.
///
/// # Arguments
///
/// * `prog_name` - Name of the program executable
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
