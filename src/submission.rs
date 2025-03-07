use crate::jobs;
use chrono::Local;
use regex::Regex;
use std::fs::{self, File};
use std::io::{self, Write};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum InputFormat {
    Csv,
    Directory,
    // Add new formats here in the future
}

pub fn write_job_log(log_file_path: &str, jobs: &[String]) -> io::Result<()> {
    let mut log_file = File::create(log_file_path)?;
    for job_command in jobs.iter() {
        writeln!(log_file, "{}", job_command)?;
    }
    Ok(())
}

pub fn calculate_batch_size(num_jobs: usize, batch_size: Option<usize>) -> usize {
    batch_size.unwrap_or_else(|| {
        let calculated = ((num_jobs as f64) * 0.2).ceil() as usize;
        calculated.min(num_jobs)
    })
}

fn count_lines_in_file(file_path: &str) -> io::Result<usize> {
    let content = std::fs::read_to_string(file_path)?;
    Ok(content.lines().count())
}

fn print_run_stats(num_jobs: usize, log_dir: &str, log_file_path: &str, job_id: &str) {
    let message = format!(
        r#"🚀 Job submission complete! ✅
🔖 Job ID is: {}
📌 {} jobs submitted.
📝 Job commands logged in: {}
📂 Logs can be found in: {}
📡 Track with -
   arrayify check {}"#,
        job_id, num_jobs, log_file_path, log_dir, job_id
    );

    println!("{}", message);
}

fn submit_jobs_to_scheduler(
    job_file_path: &str,
    log_dir: &str,
    job_prefix: &str,
    memory_mb: u32,
    threads: u32,
    queue: &str,
    batch_size: usize,
) -> io::Result<String> {
    // Count the number of lines in the file to determine the job array size
    let num_jobs = count_lines_in_file(job_file_path)?;
    let job_array = format!("{}_job_array[1-{}]%{}", job_prefix, num_jobs, batch_size);
    let output_log = format!("{}/job_%J_%I.out", log_dir);
    let error_log = format!("{}/job_%J_%I.err", log_dir);

    // Generate the bsub command
    let bsub_cmd = format!(
        "bsub -J {} -q {} -n {} -M {} -R \"select[mem>{}] rusage[mem={}]\" -o {} -e {}",
        job_array, queue, threads, memory_mb, memory_mb, memory_mb, output_log, error_log
    );

    // Generate the script that uses `sed` to extract the job command from the file
    let script = format!(
        r#"#!/bin/bash

INDEX=$((LSB_JOBINDEX - 1))
COMMAND=$(sed -n "$((INDEX + 1))p" {})
$COMMAND
"#,
        job_file_path
    );

    // Submit the job using the bsub command
    let child = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{}' | {}", script, bsub_cmd))
        .output()?;

    // Extract the job ID from the bsub output
    let bsub_output = String::from_utf8_lossy(&child.stdout);
    let re = Regex::new(r"Job <(\d+)>").unwrap();
    let job_id = re
        .captures(&bsub_output)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .unwrap_or("unknown");

    Ok(job_id.to_string())
}

pub fn submit_jobs(
    input_path: &str,
    command_template: &str,
    job_prefix: &str,
    log_dir: &str,
    memory_gb: u32,
    threads: u32,
    queue: &str,
    batch_size: Option<usize>,
    format: InputFormat,
) -> io::Result<()> {
    let memory_mb = memory_gb * 1000;
    fs::create_dir_all(log_dir)?;

    // Read jobs based on the input format
    let jobs = match format {
        InputFormat::Csv => jobs::read_jobs_from_csv(input_path, command_template)?,
        InputFormat::Directory => jobs::read_jobs_from_dir(input_path, command_template)?,
        // Add new formats here in the future
    };

    if jobs.is_empty() {
        eprintln!("No jobs found.");
        return Ok(());
    }

    // Log the jobs
    let timestamp = Local::now().format("%Y-%m-%d-%H-%M").to_string();
    let log_file_path = format!("{}/arrayify-{}.log", log_dir, timestamp);
    write_job_log(&log_file_path, &jobs)?;

    // Submit jobs to the scheduler
    let batch_size = calculate_batch_size(jobs.len(), batch_size);
    let job_id = submit_jobs_to_scheduler(&log_file_path, log_dir, job_prefix,  memory_mb, threads, queue, batch_size)?;

    // Print run statistics
    print_run_stats(jobs.len(), log_dir, &log_file_path, &job_id);
    Ok(())
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

        let jobs = jobs::read_jobs_from_csv(
            csv_file.path().to_str().unwrap(),
            "echo {header1} {header2}",
        )
        .unwrap();
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
        assert!(contents.contains("job1"));
        assert!(contents.contains("job2"));
    }

    #[test]
    fn test_submit_jobs_empty_csv() {
        let mut csv_file = NamedTempFile::new().unwrap();
        writeln!(csv_file, "header1,header2").unwrap(); // Empty CSV

        let result = submit_jobs(
            csv_file.path().to_str().unwrap(),
            "echo {header1}",
            "arrayify",
            "logs",
            1,
            1,
            "normal",
            None,
            InputFormat::Csv,
        );

        assert!(result.is_ok());
    }
}
