use std::fs;
use std::process::Command;
use csv::ReaderBuilder;
use clap::{Arg, Command as ClapCommand, ArgMatches};

fn parse_args() -> ArgMatches {
    ClapCommand::new("arrayify")
        .version("1.3")
        .author("Sam Dougan")
        .about("Submits a bsub job array from a CSV file")
        .arg(Arg::new("csv")
            .short('s')
            .long("csv")
            .value_name("CSV_FILE")
            .help("Path to the CSV file")
            .required(true))
        .arg(Arg::new("command")
            .short('c')
            .long("command")
            .value_name("COMMAND_TEMPLATE")
            .help("Command template with placeholders for CSV headers (e.g., 'echo {ID} {R1} {R2}')")
            .required(true))
        .arg(Arg::new("log")
            .short('l')
            .long("log")
            .value_name("LOG_DIR")
            .help("Path to the log directory")
            .default_value("logs"))
        .arg(Arg::new("memory")
            .short('m')
            .long("memory")
            .value_name("MEMORY_GB")
            .help("Memory per job in GB")
            .default_value("1"))
        .arg(Arg::new("threads")
            .short('t')
            .long("threads")
            .value_name("THREADS")
            .help("Threads per job")
            .default_value("1"))
        .get_matches()
}

fn print_run_stats(num_jobs: usize, log_dir: &str) {
    println!("✅ Job submission complete!");
    println!("📌 {} jobs submitted.", num_jobs);
    println!("📂 Logs can be found in: {}", log_dir);
    println!("📡 Track progress using 'bjobs'!");
}

fn main() {
    let matches = parse_args();
    
    let csv_file = matches.get_one::<String>("csv").unwrap();
    let command_template = matches.get_one::<String>("command").unwrap();
    let log_dir = matches.get_one::<String>("log").unwrap();
    let memory_gb: u32 = matches.get_one::<String>("memory").unwrap().parse()
        .expect("Memory must be a valid number in GB");
    let threads: u32 = matches.get_one::<String>("threads").unwrap().parse()
        .expect("Threads must be a valid number");

    // Convert memory to MB (bsub uses MB)
    let memory_mb = memory_gb * 1000; 

    // Create log directory if it doesn't exist
    fs::create_dir_all(log_dir).expect("Failed to create log directory");

    // Read the CSV file
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(csv_file).expect("Failed to open CSV file");

    let headers = rdr.headers().unwrap().clone();
    let mut jobs = Vec::new();

    for result in rdr.records() {
        let record = result.expect("Failed to read CSV record");
        let mut job_command = command_template.clone();

        for (i, header) in headers.iter().enumerate() {
            let placeholder = format!("{{{}}}", header);
            if let Some(value) = record.get(i) {
                job_command = job_command.replace(&placeholder, value);
            }
        }

        jobs.push(job_command);
    }

    let num_jobs = jobs.len();
    if num_jobs == 0 {
        eprintln!("No jobs found in CSV.");
        return;
    }

    // Submit the job array
    let job_array = format!("arrayify_job_array[1-{}]", num_jobs);
    let output_log = format!("{}/job_%J_%I.out", log_dir);
    let error_log = format!("{}/job_%J_%I.err", log_dir);

    let bsub_cmd = format!(
        "bsub -J {} -n {} -M {} -R \"select[mem>{}] rusage[mem={}]\" -o {} -e {}",
        job_array, threads, memory_mb, memory_mb, memory_mb, output_log, error_log
    );

    let mut script = String::new();
    script.push_str("#!/bin/bash\n\nINDEX=$((LSB_JOBINDEX - 1))\n\n");
    script.push_str("JOBS=(");
    for job in &jobs {
        script.push_str(&format!("\"{}\" ", job));
    }
    script.push_str(")\n\n");

    script.push_str("COMMAND=${JOBS[$INDEX]}\n");
    script.push_str("$COMMAND\n");

    // Run the bsub command
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{}' | {}", script, bsub_cmd))
        .spawn()
        .expect("Failed to execute bsub command");

    let _ = child.wait();
    
    // Print run stats
    print_run_stats(num_jobs, log_dir);
}
