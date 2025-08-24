use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Local;
use clap::ValueEnum;
use git2::Repository;
use ignore::gitignore::Gitignore;
use tracing::{error, info, warn};
use walkdir::{DirEntry, WalkDir};

/// Represents the output format for the bundled files.
///
/// - `Markdown`: Outputs files in Markdown format with code blocks.
/// - `Text`: Outputs files as plain text.
/// - `Console`: Outputs files formatted for console display (default).
#[derive(Debug, Clone, ValueEnum, Default)]
pub enum Format {
    Markdown,
    Text,
    #[default]
    Console,
}

/// Configuration options for the file bundling process.
#[derive(Debug)]
pub struct Config {
    /// The directory to process.
    pub directory: PathBuf,
    /// The optional output file path. If not provided, output is written to stdout.
    pub output: Option<PathBuf>,
    /// File extensions to include in the output.
    pub include: Vec<String>,
    /// File extensions to exclude from the output.
    pub exclude: Vec<String>,
    /// The format of the output (Markdown, Text, or Console).
    pub format: Format,
    /// Whether to append the current date to the output file name.
    pub append_date: bool,
    /// Whether to append the current Git hash to the output file name.
    pub append_git_hash: bool,
    /// Whether to include line numbers in the output.
    pub line_numbers: bool,
    /// Whether to ignore hidden files and directories.
    pub ignore_hidden: bool,
    /// Whether to respect `.gitignore` rules.
    pub respect_gitignore: bool,
}

/// Runs the file bundling process based on the provided configuration.
///
/// # Arguments
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` if successful, or an error if the process fails.
pub fn run(config: Config) -> Result<()> {
    let mut output_path = config.output.clone();

    if config.append_date || config.append_git_hash {
        append_date_and_git_hash(&mut output_path, &config)?;
    }

    let writer = determine_output_writer(&output_path)?;

    process_directory(&config, writer)
}

/// Appends the current date and/or Git hash to the output file name if required.
///
/// # Arguments
/// * `output_path` - The optional output file path to modify.
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` if successful, or an error if the operation fails.
fn append_date_and_git_hash(output_path: &mut Option<PathBuf>, config: &Config) -> Result<()> {
    if let Some(path) = output_path {
        let mut new_filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        if config.append_date {
            new_filename.push('_');
            new_filename.push_str(&Local::now().format("%Y%m%d").to_string());
            info!("Appending date to filename.");
        }

        if config.append_git_hash {
            match Repository::open(&config.directory) {
                Ok(repo) => {
                    let head = repo.head().context("Failed to get repository HEAD")?;
                    if let Some(oid) = head.target() {
                        new_filename.push('_');
                        new_filename.push_str(&oid.to_string()[..7]);
                        info!("Appending git hash to filename.");
                    }
                }
                Err(_) => warn!("Not a git repository, cannot append git hash."),
            }
        }

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            new_filename.push('.');
            new_filename.push_str(ext);
        }
        path.set_file_name(new_filename);
    }
    Ok(())
}

/// Determines the output writer (file or stdout) based on the configuration.
///
/// # Arguments
/// * `output_path` - The optional output file path.
///
/// # Returns
/// * `Result<Box<dyn Write>>` - Returns a writer for the output.
fn determine_output_writer(output_path: &Option<PathBuf>) -> Result<Box<dyn Write>> {
    if let Some(path) = output_path {
        info!("Output will be written to: {}", path.display());
        let file = File::create(path)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?;
        Ok(Box::new(BufWriter::new(file)))
    } else {
        info!("Output will be written to stdout.");
        Ok(Box::new(BufWriter::new(io::stdout())))
    }
}

/// Processes the specified directory and writes the bundled content to the writer.
///
/// # Arguments
/// * `config` - The configuration options for the bundling process.
/// * `writer` - The writer to output the bundled content.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` if successful, or an error if the process fails.
fn process_directory(config: &Config, mut writer: Box<dyn Write>) -> Result<()> {
    let (gitignore, _) = Gitignore::new(config.directory.join(".gitignore"));

    let walker = WalkDir::new(&config.directory)
        .into_iter()
        .filter_entry(|e| should_include_entry(e, &gitignore, config));

    for result in walker {
        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                error!("Failed to access entry: {}", err);
                continue;
            }
        };

        if let Err(err) = process_file_entry(&entry, &mut writer, config) {
            error!("{}", err);
        }
    }

    info!("File bundling complete.");
    Ok(())
}

/// Determines if a directory entry should be included based on the configuration.
///
/// # Arguments
/// * `entry` - The directory entry to check.
/// * `gitignore` - The `.gitignore` rules to respect.
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `bool` - Returns `true` if the entry should be included, `false` otherwise.
fn should_include_entry(entry: &DirEntry, gitignore: &Gitignore, config: &Config) -> bool {
    !is_hidden(entry, config) && !is_ignored(entry, gitignore, config)
}

/// Processes a single file entry and writes its content to the writer.
///
/// # Arguments
/// * `entry` - The file entry to process.
/// * `writer` - The writer to output the file content.
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` if successful, or an error if the process fails.
fn process_file_entry(entry: &DirEntry, writer: &mut dyn Write, config: &Config) -> Result<()> {
    let path = entry.path();
    if !path.is_file() {
        return Ok(());
    }

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let apply_include_filter =
        !(config.include.is_empty() || config.include.len() == 1 && config.include[0].is_empty());

    if apply_include_filter && !config.include.contains(&extension.to_string()) {
        return Ok(());
    }

    if config.exclude.contains(&extension.to_string()) {
        return Ok(());
    }

    let relative_path = path.strip_prefix(&config.directory).unwrap_or(path);
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            warn!("Skipping non-UTF-8 file: {}", path.display());
            return Ok(()); // Skip non-text files
        }
    };

    write_file_content(writer, relative_path, &content, extension, config)
        .with_context(|| format!("Failed to write file content for {}", path.display()))
}

/// Writes the content of a single file to the writer based on the specified format.
///
/// # Arguments
/// * `writer` - The writer to output the file content.
/// * `path` - The relative path of the file.
/// * `content` - The content of the file.
/// * `extension` - The file extension.
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` if successful, or an error if the operation fails.
fn write_file_content(
    writer: &mut dyn Write,
    path: &Path,
    content: &str,
    extension: &str,
    config: &Config,
) -> Result<()> {
    match config.format {
        Format::Markdown => {
            writeln!(writer, "### `{}`\n", path.display())?;
            writeln!(writer, "```{}", extension)?;
            write_content_lines(writer, content, config.line_numbers)?;
            writeln!(writer, "```\n")?;
        }
        Format::Text | Format::Console => {
            // In Console mode, we could add colors or other specific formatting later
            writeln!(writer, "./{}\n---", path.display())?;
            write_content_lines(writer, content, config.line_numbers)?;
            writeln!(writer, "---")?;
        }
    }
    Ok(())
}

/// Writes content line by line to the writer, optionally including line numbers.
///
/// # Arguments
/// * `writer` - The writer to output the content.
/// * `content` - The content to write.
/// * `line_numbers` - Whether to include line numbers.
///
/// # Returns
/// * `Result<()>` - Returns `Ok(())` if successful, or an error if the operation fails.
fn write_content_lines(writer: &mut dyn Write, content: &str, line_numbers: bool) -> Result<()> {
    if line_numbers {
        for (i, line) in content.lines().enumerate() {
            writeln!(writer, "{:4} | {}", i + 1, line)?;
        }
    } else {
        writeln!(writer, "{}", content)?;
    }
    Ok(())
}

/// Checks if a directory entry is hidden based on the configuration.
///
/// # Arguments
/// * `entry` - The directory entry to check.
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `bool` - Returns `true` if the entry is hidden, `false` otherwise.
fn is_hidden(entry: &DirEntry, config: &Config) -> bool {
    config.ignore_hidden
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
}

/// Checks if a directory entry is ignored by `.gitignore` rules.
///
/// # Arguments
/// * `entry` - The directory entry to check.
/// * `gitignore` - The `.gitignore` rules to respect.
/// * `config` - The configuration options for the bundling process.
///
/// # Returns
/// * `bool` - Returns `true` if the entry is ignored, `false` otherwise.
fn is_ignored(entry: &DirEntry, gitignore: &Gitignore, config: &Config) -> bool {
    if !config.respect_gitignore {
        return false;
    }
    gitignore
        .matched(entry.path(), entry.file_type().is_dir())
        .is_ignore()
}
