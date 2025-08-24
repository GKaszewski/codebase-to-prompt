use anyhow::Result;
use clap::Parser;
use codebase_to_prompt::Format;
use std::path::PathBuf;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    directory: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long, use_value_delimiter = true, default_value = "")]
    include: Vec<String>,

    #[arg(short, long, use_value_delimiter = true, default_value = "")]
    exclude: Vec<String>,

    #[arg(long, value_enum, default_value_t = Format::Console)]
    format: Format,

    #[arg(long)]
    append_date: bool,

    #[arg(long)]
    append_git_hash: bool,

    #[arg(long)]
    line_numbers: bool,

    #[arg(long)]
    ignore_hidden: bool,

    #[arg(long, default_value_t = true)]
    respect_gitignore: bool,
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let args = Args::parse();

    let config = codebase_to_prompt::Config {
        directory: args.directory,
        output: args.output,
        include: args.include,
        exclude: args.exclude,
        format: args.format,
        append_date: args.append_date,
        append_git_hash: args.append_git_hash,
        line_numbers: args.line_numbers,
        ignore_hidden: args.ignore_hidden,
        respect_gitignore: args.respect_gitignore,
    };

    codebase_to_prompt::run(config)
}
