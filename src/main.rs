mod args;
mod jobs;
mod submission;

use clap::Subcommand;
use std::process::Command;
use submission::InputFormat;

#[derive(Subcommand)]
enum SubCommands {
    Sub,
    Check {
        #[arg(short, long, value_name = "JOB_ID")]
        job_id: String,
    },
}

fn check_jobs(job_id: &str) {
    let output = Command::new("bjobs")
        .arg("-noheader")
        .arg("-o")
        .arg("job_name stat exit_code")
        .arg(job_id)
        .output()
        .expect("Failed to check job status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut failed_jobs = Vec::new();
    let mut all_done = true;
    let mut running_count = 0;
    let mut pending_count = 0;
    let mut done_count = 0;

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let array_name = parts[0];
            let status = parts[1];
            let exit_code = parts[2];

            match status {
                "EXIT" => {
                    all_done = false;
                    let reason = match exit_code {
                        "2" => "Killed 💀",
                        "130" => "Memory error 🧠💾",
                        "137" => "Killed (OOM) 💀🛑💾",
                        "143" => "Timeout ⏳",
                        _ => "Unknown error ❓🚨",
                    };
                    failed_jobs.push((
                        array_name.to_string(),
                        exit_code.to_string(),
                        reason.to_string(),
                    ));
                }
                "RUN" => {
                    all_done = false;
                    running_count += 1;
                }
                "PEND" => {
                    all_done = false;
                    pending_count += 1;
                }
                "DONE" => {
                    done_count += 1;
                }
                _ => {
                    all_done = false;
                }
            }
        }
    }

    if all_done && failed_jobs.is_empty() {
        println!("✅ All jobs in array {} completed successfully!", job_id);
    } else {
        if running_count > 0 {
            println!("🚀 {} jobs are currently running!", running_count);
        }
        if pending_count > 0 {
            println!("⏳ {} jobs are still pending!", pending_count);
        }
        if done_count > 0 {
            println!("✅ {} jobs have completed successfully!", done_count);
        }
        if !failed_jobs.is_empty() {
            println!("❌ Some jobs in array {} had issues:", job_id);
            for (array_name, code, reason) in failed_jobs {
                println!("  - {} Exit Code {}: {}", array_name, code, reason);
            }
        }
    }
}

fn main() {
    let matches = args::parse_args();

    match matches.subcommand() {
        Some(("sub", sub_matches)) => {
            let csv_file = sub_matches.get_one::<String>("csv");
            let dir_path = sub_matches.get_one::<String>("dir");

            // Ensure only one of csv_file or dir_path is provided
            if csv_file.is_some() && dir_path.is_some() {
                eprintln!("Error: Cannot provide both --csv and --dir at the same time");
                std::process::exit(1);
            }

            let command_template = sub_matches.get_one::<String>("command").unwrap();
            let log_dir = sub_matches.get_one::<String>("log").unwrap();
            let memory_gb: u32 = sub_matches
                .get_one::<String>("memory")
                .unwrap()
                .parse()
                .expect("Memory must be a valid number in GB");
            let threads: u32 = sub_matches
                .get_one::<String>("threads")
                .unwrap()
                .parse()
                .expect("Threads must be a valid number");
            let batch_size = sub_matches
                .get_one::<String>("batch_size")
                .map(|value| {
                    if value == "auto" {
                        None
                    } else {
                        value.parse::<usize>().ok()
                    }
                })
                .unwrap_or(None);

            // Determine the input format and set input_path
            let (format, input_path) = if let Some(csv) = csv_file {
                (InputFormat::Csv, csv)
            } else if let Some(dir) = dir_path {
                (InputFormat::Directory, dir)
            } else {
                eprintln!("Error: Either --csv or --dir must be provided");
                std::process::exit(1);
            };

            submission::submit_jobs(
                input_path,
                command_template,
                log_dir,
                memory_gb,
                threads,
                batch_size,
                format,
            )
            .expect("Job submission failed");
        }
        Some(("check", check_matches)) => {
            let job_id = check_matches.get_one::<String>("job_id").unwrap();
            check_jobs(job_id);
        }
        _ => {}
    }
}
