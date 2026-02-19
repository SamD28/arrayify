mod args;
mod jobs;
mod submission;
use crate::args::Commands;
use std::process::Command;
use submission::InputFormat;

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
    let cli = args::parse_args();

    let command = cli
        .command
        .or(cli.sub.map(Commands::Sub))
        .expect("error validating clap subcommand");

    match command {
        Commands::Sub(args) => {
            let csv_file = args.csv;
            let dir_path = args.dir;

            if csv_file.is_some() && dir_path.is_some() {
                eprintln!("Error: Cannot provide both --csv and --dir at the same time");
                std::process::exit(1);
            }

            let (format, input_path): (InputFormat, String) = if let Some(csv) = csv_file {
                (InputFormat::Csv, csv)
            } else if let Some(dir) = dir_path {
                (InputFormat::Directory, dir)
            } else {
                eprintln!("Error: Either --csv or --dir must be provided");
                std::process::exit(1);
            };

            let batch_size = if args.batch_size == "auto" {
                None
            } else {
                args.batch_size.parse::<usize>().ok()
            };

            submission::submit_jobs(
                &input_path,
                &args.command,
                &args.job_prefix,
                &args.log,
                &args.memory,
                &args.threads,
                &args.queue,
                batch_size,
                format,
            )
            .expect("Job submission failed");
        }
        Commands::Check { job_id } => {
            check_jobs(&job_id);
        }
    }
}
