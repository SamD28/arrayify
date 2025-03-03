mod args;

use clap::Subcommand;
use csv::ReaderBuilder;
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;

#[derive(Subcommand)]
enum SubCommands {
    Sub,
    Check {
        #[arg(short, long, value_name = "JOB_ID")]
        job_id: String,
    },
}

fn read_jobs_from_csv(csv_file: &str, command_template: &str) -> io::Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(csv_file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let headers = rdr.headers().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?.clone();
    let mut jobs = Vec::new();

    for result in rdr.records() {
        let record = result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut job_command = command_template.to_string();

        for (i, header) in headers.iter().enumerate() {
            let placeholder = format!("{{{}}}", header);
            if let Some(value) = record.get(i) {
                job_command = job_command.replace(&placeholder, value);
            }
        }
        jobs.push(job_command);
    }

    Ok(jobs)
}

fn calculate_batch_size(num_jobs: usize, batch_size: Option<usize>) -> usize {
    batch_size.unwrap_or_else(|| {
        let calculated = ((num_jobs as f64) * 0.2).ceil() as usize;
        calculated.min(num_jobs)
    })
}

fn submit_jobs_to_scheduler(jobs: &[String], log_dir: &str, memory_mb: u32, threads: u32, batch_size: usize) -> io::Result<String> {
    let num_jobs = jobs.len();
    let job_array = format!("arrayify_job_array[1-{}]%{}", num_jobs, batch_size);
    let output_log = format!("{}/job_%J_%I.out", log_dir);
    let error_log = format!("{}/job_%J_%I.err", log_dir);

    let bsub_cmd = format!(
        "bsub -J {} -n {} -M {} -R \"select[mem>{}] rusage[mem={}]\" -o {} -e {}",
        job_array, threads, memory_mb, memory_mb, memory_mb, output_log, error_log
    );

    let mut script = String::new();
    script.push_str("#!/bin/bash\n\nINDEX=$((LSB_JOBINDEX - 1))\n\n");
    script.push_str("JOBS=(");
    for job in jobs {
        script.push_str(&format!("\"{}\" ", job));
    }
    script.push_str(")\n\n");

    script.push_str("COMMAND=${JOBS[$INDEX]}\n");
    script.push_str("$COMMAND\n");

    let child = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{}' | {}", script, bsub_cmd))
        .output()?;

    let bsub_output = String::from_utf8_lossy(&child.stdout);
    let re = Regex::new(r"Job <(\d+)>").unwrap();
    let job_id = re
        .captures(&bsub_output)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .unwrap_or("unknown");

    Ok(job_id.to_string())
}

fn write_job_log(log_file_path: &str, jobs: &[String]) -> io::Result<()> {
    let mut log_file = File::create(log_file_path)?;
    for (index, job_command) in jobs.iter().enumerate() {
        writeln!(log_file, "[{}] {}", index + 1, job_command)?;
    }
    Ok(())
}

pub fn submit_jobs(
    csv_file: &str,
    command_template: &str,
    log_dir: &str,
    memory_gb: u32,
    threads: u32,
    batch_size: Option<usize>,
) -> io::Result<()> {
    let memory_mb = memory_gb * 1000;
    fs::create_dir_all(log_dir)?;

    let jobs = read_jobs_from_csv(csv_file, command_template)?;
    if jobs.is_empty() {
        eprintln!("No jobs found in CSV.");
        return Ok(());
    }

    let batch_size = calculate_batch_size(jobs.len(), batch_size);
    let job_id = submit_jobs_to_scheduler(&jobs, log_dir, memory_mb, threads, batch_size)?;

    let log_file_path = format!("{}/arrayify-{}.log", log_dir, job_id);
    write_job_log(&log_file_path, &jobs)?;

    print_run_stats(jobs.len(), log_dir, log_file_path, &job_id);
    Ok(())
}

fn print_run_stats(num_jobs: usize, log_dir: &str, log_file_path: String, job_id: &str) {
    println!("🚀 Job submission complete! ✅");
    println!("🔖 Job ID is: {}", job_id);
    println!("📌 {} jobs submitted.", num_jobs);
    println!("📝 Job commands logged in: {}", log_file_path);
    println!("📂 Logs can be found in: {}", log_dir);
    println!("📡 Track with -\narrayify check {}\n", job_id);
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
            let csv_file = sub_matches.get_one::<String>("csv").unwrap();
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

            let _ = submit_jobs(
                csv_file,
                command_template,
                log_dir,
                memory_gb,
                threads,
                batch_size,
            );
        }
        Some(("check", check_matches)) => {
            let job_id = check_matches.get_one::<String>("job_id").unwrap();
            check_jobs(job_id);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_jobs_from_csv() {
        let mut csv_file = NamedTempFile::new().unwrap();
        writeln!(csv_file, "header1,header2\nvalue1,value2").unwrap();

        let jobs = read_jobs_from_csv(csv_file.path().to_str().unwrap(), "echo {header1} {header2}").unwrap();
        assert_eq!(jobs, vec!["echo value1 value2"]);
    }

    #[test]
    fn test_calculate_batch_size() {
        assert_eq!(calculate_batch_size(10, None), 2); // 20% of 10, rounded up
        assert_eq!(calculate_batch_size(10, Some(5)), 5); // Custom batch size
        assert_eq!(calculate_batch_size(1, None), 1); // Minimum batch size
    }

    #[test]
    fn test_write_job_log() {
        let log_file = NamedTempFile::new().unwrap();
        let jobs = vec!["job1".to_string(), "job2".to_string()];

        write_job_log(log_file.path().to_str().unwrap(), &jobs).unwrap();

        let contents = fs::read_to_string(log_file.path()).unwrap();
        assert!(contents.contains("[1] job1"));
        assert!(contents.contains("[2] job2"));
    }

    #[test]
    fn test_submit_jobs_empty_csv() {
        let mut csv_file = NamedTempFile::new().unwrap();
        writeln!(csv_file, "header1,header2").unwrap(); // Empty CSV

        let result = submit_jobs(
            csv_file.path().to_str().unwrap(),
            "echo {header1}",
            "logs",
            1,
            1,
            None,
        );

        assert!(result.is_ok());
    }
}