# Prepend

A command-line utility for prepending text to files. Uses buffered I/O and atomic file operations.

## Installation

### From Source

```bash
git clone https://github.com/Cod-e-Codes/prepend
cd prepend
cargo build --release
```

### Install Globally

```bash
cargo install --path .
```

## Usage

### Interactive Mode

Run without text argument to enter interactive mode:

```bash
prepend myfile.txt
```

Type or paste your text, then press Ctrl+D (Unix) or Ctrl+Z (Windows) on a new line to finish.

### Command-Line Mode

Provide text directly as an argument:

```bash
prepend myfile.txt "Header text"
```

### Dry-Run Mode

Preview changes without modifying the file:

```bash
prepend --dry-run myfile.txt "Header text"
```

### Examples

Add a comment header to a source file:

```bash
prepend main.rs "// Copyright 2025"
```

Add multiple lines interactively:

```bash
prepend README.md
# Type your text
# Press Ctrl+D when done
```

Prepend to a log file:

```bash
prepend app.log "--- Session started ---"
```

## Project Structure

```
prepend/
├── src/
│   ├── main.rs      # Binary entry point
│   └── lib.rs       # Core library implementation
├── tests/
│   ├── cli_tests.rs         # End-to-end CLI tests (12 tests)
│   └── integration_tests.rs # Library integration tests (15 tests)
└── Cargo.toml
```

The project is structured as both a binary and library crate. Core functionality is exposed through `lib.rs` for testing and potential reuse.

## Supported File Types

The tool validates file extensions and supports common text file types:

- Text files: txt, log, md
- Config files: conf, yaml, json, cfg, ini, csv
- Scripts: sh, py, js
- Source code: c, cpp, h, rs

Files with uncommon extensions will show a warning but can still be processed.

## Technical Details

### Performance

- Uses 64KB buffered I/O for efficient processing
- Handles large files without memory issues
- Atomic file replacement prevents corruption

### Safety

- Validates file existence and permissions before modification
- Creates temporary file in same directory as target
- Atomic rename operation ensures data integrity
- Automatic cleanup on failure

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with output:

```bash
cargo test -- --nocapture
```

Run specific test:

```bash
cargo test test_prepend_to_empty_file
```

Check code quality:

```bash
cargo clippy
cargo fmt
```

### Test Coverage

- 12 CLI tests covering command-line interface behavior
- 15 integration tests covering core library functionality
- Tests include edge cases: empty files, large files, binary files, special characters

## License

MIT License. See LICENSE file for details.
