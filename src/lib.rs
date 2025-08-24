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

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum Format {
    Markdown,
    Text,
    #[default]
    Console,
}

#[derive(Debug)]
pub struct Config {
    pub directory: PathBuf,
    pub output: Option<PathBuf>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub format: Format,
    pub append_date: bool,
    pub append_git_hash: bool,
    pub line_numbers: bool,
    pub ignore_hidden: bool,
    pub respect_gitignore: bool,
}

pub fn run(config: Config) -> Result<()> {
    let mut output_path = config.output.clone();

    if config.append_date || config.append_git_hash {
        append_date_and_git_hash(&mut output_path, &config)?;
    }

    let writer = determine_output_writer(&output_path)?;

    process_directory(&config, writer)
}

/// Appends date and git hash to the output file name if required.
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

/// Determines the output writer (file or stdout).
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

// Refactored `process_directory` function
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

/// Determines if a directory entry should be included.
fn should_include_entry(entry: &DirEntry, gitignore: &Gitignore, config: &Config) -> bool {
    !is_hidden(entry, config) && !is_ignored(entry, gitignore, config)
}

/// Processes a single file entry.
fn process_file_entry(entry: &DirEntry, writer: &mut dyn Write, config: &Config) -> Result<()> {
    let path = entry.path();
    if !path.is_file() {
        return Ok(());
    }

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let apply_include_filter =
        !config.include.is_empty() && !(config.include.len() == 1 && config.include[0].is_empty());

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

/// Helper to write content line by line, optionally with line numbers.
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

/// Helper function to check if a directory entry is hidden.
fn is_hidden(entry: &DirEntry, config: &Config) -> bool {
    config.ignore_hidden
        && entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
}

/// Helper function to check if a directory entry is ignored by .gitignore.
fn is_ignored(entry: &DirEntry, gitignore: &Gitignore, config: &Config) -> bool {
    if !config.respect_gitignore {
        return false;
    }
    gitignore
        .matched(entry.path(), entry.file_type().is_dir())
        .is_ignore()
}
