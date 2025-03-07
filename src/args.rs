use clap::{Arg, ArgMatches, Command as ClapCommand};

pub fn parse_args() -> ArgMatches {
    ClapCommand::new("arrayify")
        .version("0.2.1")
        .author("Sam Dougan")
        .about("Submits and checks bsub job arrays from a CSV file or directory")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            ClapCommand::new("sub")
                .about("Submit a job array from a CSV file or a directory")
                .long_about(
                    "This command allows you to submit a job array using a CSV file or a directory. \
                    You must provide a command template with placeholders. \
                    Example: 'echo {ID} {R1} {R2}'"
                )
                .arg(
                    Arg::new("csv")
                        .short('s')
                        .long("csv")
                        .value_name("CSV_FILE")
                        .help("Path to the CSV file containing job information")
                        .long_help(
                            "Specify a CSV file containing job details. \
                            Each row represents a separate job, and headers can be used as placeholders \
                            in the command template."
                        )
                        .conflicts_with("dir")
                        .required_unless_present("dir")
                )
                .arg(
                    Arg::new("dir")
                        .short('d')
                        .long("dir")
                        .value_name("DIRECTORY")
                        .help("Path to the directory containing input files")
                        .long_help(
                            "Specify a directory that contains input files for job processing. \
                            This option is mutually exclusive with --csv. \
                            Headers are always ID, R1, R2 extracted from _1* _2* and ID being the prefix"
                        )
                        .conflicts_with("csv")
                        .required_unless_present("csv")
                )
                .arg(
                    Arg::new("command")
                        .short('c')
                        .long("command")
                        .value_name("COMMAND_TEMPLATE")
                        .help("Command template using placeholders for CSV headers")
                        .long_help(
                            "Define the command template that will be executed for each job. \
                            Placeholders enclosed in {} (e.g., {ID}, {R1}, {R2}) will be replaced with \
                            values from the CSV or directory listing. \
                            Example: 'echo {ID} {R1} {R2}'"
                        )
                        .required(true)
                )
                .arg(
                    Arg::new("job_prefix")
                    .short('p')
                    .long("job_prefix")
                    .value_name("PREFIX")
                    .help("prefix for job submission name i.e. prefix_job_array")
                    .default_value("arrayify")
                )
                .arg(
                    Arg::new("log")
                        .short('l')
                        .long("log")
                        .value_name("LOG_DIR")
                        .help("Directory to store log files")
                        .default_value("logs")
                )
                .arg(
                    Arg::new("memory")
                        .short('m')
                        .long("memory")
                        .value_name("MEMORY_GB")
                        .help("Amount of memory per job in GB")
                        .default_value("1")
                )
                .arg(
                    Arg::new("threads")
                        .short('t')
                        .long("threads")
                        .value_name("THREADS")
                        .help("Number of threads per job")
                        .default_value("1")
                )
                .arg(
                    Arg::new("batch_size")
                        .short('b')
                        .long("batch")
                        .value_name("BATCH_SIZE")
                        .help("Number of jobs running concurrently (default: 20% of array)")
                        .default_value("auto")
                )
                .arg(
                    Arg::new("queue")
                    .short('q')
                    .long("queue")
                    .value_name("QUEUE")
                    .help("Bsub queue to submit to")
                    .default_value("normal")
                )
        )
        .subcommand(
            ClapCommand::new("check")
                .about("Check the status of a submitted job")
                .long_about(
                    "Use this command to check the status of a job by providing its LSF Job ID."
                )
                .arg(
                    Arg::new("job_id")
                        .value_name("JOB_ID")
                        .help("The LSF Job ID to check")
                        .required(true)
                )
        )
        .get_matches()
}
