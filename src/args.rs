use clap::Parser;

use crate::types::InputFormat;

#[derive(Parser, Debug)]
#[command(
    name = "arrayify",
    version = "0.2.1",
    author = "Sam Dougan",
    about = "Submits and checks bsub job arrays from a CSV file or directory",
    arg_required_else_help = true
)]
pub struct Cli {
    #[arg(short = 's', long, value_name = "INPUT")]
    pub input: InputFormat,

    #[arg(short, long, value_name = "COMMAND_TEMPLATE", required = true)]
    pub command: String,

    #[arg(short, long, default_value = "arrayify")]
    pub job_prefix: String,

    #[arg(short = 'l', long = "log", default_value = "logs")]
    pub log_directory: String,

    #[arg(short = 'm', long = "memory", default_value_t = 1, value_name = "INT")]
    pub memory: usize,

    #[arg(short = 't', long = "threads", default_value_t = 1, value_name = "INT")]
    pub threads: usize,

    #[arg(
        short = 'b',
        long = "batch",
        value_name = "BATCH_SIZE",
        default_value_t = 0,
        help = "Number of jobs to include in each batch (0 = 20% of total jobs in each batch)"
    )]
    pub batch_size: usize,

    #[arg(
        short = 'q',
        long = "queue",
        value_name = "QUEUE",
        default_value = "normal"
    )]
    pub queue: String,

    #[arg(
        short = 'o',
        long = "stdout",
        value_name = "PATTERN",
        help = "Redirect each job's stdout to a file. Supports the same {column} placeholders as --command (e.g. 'results/{sample_id}.out')"
    )]
    pub stdout: Option<String>,
}
