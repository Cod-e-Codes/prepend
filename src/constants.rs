//! Constants used throughout the prepend tool.

/// ANSI escape code for red text
pub const RED: &str = "\x1b[31m";

/// ANSI escape code for green text
pub const GREEN: &str = "\x1b[32m";

/// ANSI escape code for yellow text
pub const YELLOW: &str = "\x1b[33m";

/// ANSI escape code for blue text
pub const BLUE: &str = "\x1b[34m";

/// ANSI escape code to reset text formatting
pub const RESET: &str = "\x1b[0m";

/// List of file extensions that are considered safe for text prepending
pub const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "log", "md", "sh", "conf", "yaml", "json", "csv", "cfg", "ini", "c", "cpp", "h", "py",
    "js", "rs",
];

/// Buffer size for file I/O operations (64KB)
pub const BUFFER_SIZE: usize = 64 * 1024;
