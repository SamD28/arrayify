use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "arrayify",
    version = "0.2.1",
    author = "Sam Dougan",
    about = "Submits and checks bsub job arrays from a CSV file or directory",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub sub: Option<SubArgs>,
}

#[derive(Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
pub enum Commands {
    Sub(SubArgs),
    Check {
        #[arg(short, long, value_name = "JOB_ID")]
        job_id: String,
    },
}

#[derive(Args, Clone)]
pub struct SubArgs {
    #[arg(
        short = 's',
        long = "csv",
        value_name = "CSV_FILE",
        conflicts_with = "dir",
        required_unless_present = "dir"
    )]
    pub csv: Option<String>,

    #[arg(
        short = 'd',
        long = "dir",
        value_name = "DIRECTORY",
        conflicts_with = "csv",
        required_unless_present = "csv"
    )]
    pub dir: Option<String>,

    #[arg(
        short = 'c',
        long = "command",
        value_name = "COMMAND_TEMPLATE",
        required = true
    )]
    pub command: String,

    #[arg(short = 'p', long = "job_prefix", default_value = "arrayify")]
    pub job_prefix: String,

    #[arg(short = 'l', long = "log", default_value = "logs")]
    pub log: String,

    #[arg(short = 'm', long = "memory", default_value_t = 1, value_name = "INT",)]
    pub memory: u32,

    #[arg(short = 't', long = "threads", default_value_t = 1, value_name = "INT",)]
    pub threads: u32,

    #[arg(
        short = 'b',
        long = "batch",
        value_name = "BATCH_SIZE",
        default_value = "auto"
    )]
    pub batch_size: String,

    #[arg(
        short = 'q',
        long = "queue",
        value_name = "QUEUE",
        default_value = "normal"
    )]
    pub queue: String,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
