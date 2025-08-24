use codebase_to_prompt::{Config, Format, run};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_run_with_markdown_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.md");

    let config = Config {
        directory: PathBuf::from("tests/fixtures"),
        output: Some(output_file.clone()),
        include: vec!["rs".to_string()],
        exclude: vec![],
        format: Format::Markdown,
        append_date: false,
        append_git_hash: false,
        line_numbers: false,
        ignore_hidden: true,
        respect_gitignore: true,
    };

    let result = run(config);
    assert!(result.is_ok());

    let output_content = fs::read_to_string(output_file).unwrap();
    assert!(output_content.contains("### `example.rs`"));
}

#[test]
fn test_run_with_text_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let config = Config {
        directory: PathBuf::from("tests/fixtures"),
        output: Some(output_file.clone()),
        include: vec!["txt".to_string()],
        exclude: vec![],
        format: Format::Text,
        append_date: false,
        append_git_hash: false,
        line_numbers: true,
        ignore_hidden: true,
        respect_gitignore: true,
    };

    let result = run(config);
    assert!(result.is_ok());

    let output_content = fs::read_to_string(output_file).unwrap();
    assert!(output_content.contains("1 | Example text file content"));
}

#[test]
fn test_run_with_git_hash_append() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let config = Config {
        directory: PathBuf::from("tests/fixtures"),
        output: Some(output_file.clone()),
        include: vec!["txt".to_string()],
        exclude: vec![],
        format: Format::Text,
        append_date: false,
        append_git_hash: true,
        line_numbers: false,
        ignore_hidden: true,
        respect_gitignore: true,
    };

    let result = run(config);
    assert!(result.is_ok());

    let output_file_name = output_file.file_name().unwrap().to_str().unwrap();
    assert!(output_file_name.contains("output"));
    assert!(output_file_name.len() > "output".len());
}
