use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use prepend::constants::ALLOWED_EXTENSIONS;
use prepend::{Config, perform_prepend, validate_file};

#[test]
fn test_prepend_to_empty_file() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "Header\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Header\n");
}

#[test]
fn test_prepend_to_existing_content() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Original line 1").unwrap();
    writeln!(file, "Original line 2").unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "New Header\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "New Header\nOriginal line 1\nOriginal line 2\n");
}

#[test]
fn test_prepend_multiline_text() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Original content").unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "Line 1\nLine 2\nLine 3\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Line 1\nLine 2\nLine 3\nOriginal content\n");
}

#[test]
fn test_prepend_preserves_original_content() {
    let mut file = NamedTempFile::new().unwrap();
    let original = "Line 1\nLine 2\nLine 3\nLine 4\n";
    write!(file, "{}", original).unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "Header\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("Header\n"));
    assert!(content.ends_with(original));
}

#[test]
fn test_nonexistent_file() {
    let path = PathBuf::from("/tmp/nonexistent_file_12345.txt");
    let result = validate_file(&path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist"));
}

#[test]
fn test_directory_instead_of_file() {
    let dir = tempfile::tempdir().unwrap();
    let result = validate_file(dir.path());
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("not a regular file")
    );
}

#[test]
#[cfg(unix)]
fn test_readonly_file() {
    use std::os::unix::fs::PermissionsExt;

    let file = NamedTempFile::new().unwrap();
    let path = file.path();

    let mut perms = fs::metadata(path).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(path, perms).unwrap();

    let result = validate_file(path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not writable"));
}

#[test]
fn test_large_file() {
    let mut file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();

    let line = "This is a test line with some content\n";
    for _ in 0..131072 {
        write!(file, "{}", line).unwrap();
    }

    let config = Config {
        filename: path.clone(),
        prepend_text: "Header\n".to_string(),
        dry_run: false,
    };

    let start = std::time::Instant::now();
    perform_prepend(&config).unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 5,
        "Large file took too long: {:?}",
        duration
    );

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("Header\n"));
}

#[test]
fn test_binary_file() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();

    let binary_data: Vec<u8> = vec![0, 1, 2, 255, 254, 253, 128, 127];
    fs::write(&path, &binary_data).unwrap();

    let config = Config {
        filename: path.clone(),
        prepend_text: "Text Header\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read(&path).unwrap();

    assert!(content.starts_with(b"Text Header\n"));
    assert_eq!(&content[12..], &binary_data[..]);
}

#[test]
fn test_empty_file() {
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "Only content\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Only content\n");
}

#[test]
fn test_file_with_no_extension() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "content").unwrap();
    let path = file.path().to_path_buf();

    let result = validate_file(&path);
    assert!(result.is_ok());
}

#[test]
fn test_file_with_uncommon_extension() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.xyz");
    fs::write(&path, "content\n").unwrap();

    let result = validate_file(&path);
    assert!(result.is_ok());
}

#[test]
fn test_special_characters_in_text() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "Original").unwrap();
    let path = file.path().to_path_buf();

    let special = "Special chars: 你好 🦀 табуляция \t newlines\n\r\n";
    let config = Config {
        filename: path.clone(),
        prepend_text: special.to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with(special));
}

#[test]
fn test_file_without_trailing_newline() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "No newline at end").unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "Header\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "Header\nNo newline at end");
}

#[test]
fn test_single_character_prepend() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "content").unwrap();
    let path = file.path().to_path_buf();

    let config = Config {
        filename: path.clone(),
        prepend_text: "#\n".to_string(),
        dry_run: false,
    };

    perform_prepend(&config).unwrap();
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "#\ncontent\n");
}

#[test]
fn test_validate_allowed_extensions() {
    let dir = tempfile::tempdir().unwrap();

    for ext in ALLOWED_EXTENSIONS {
        let path = dir.path().join(format!("test.{}", ext));
        fs::write(&path, "test").unwrap();
        assert!(
            validate_file(&path).is_ok(),
            "Failed for extension: {}",
            ext
        );
    }
}
