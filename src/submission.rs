use chrono::Local;
use regex::Regex;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

use crate::error::ArrayifyError;
use crate::types::Job;

impl Job<'_> {
    pub fn submit_jobs(self) -> Result<(), ArrayifyError> {
        fs::create_dir_all(self.log_directory)?;

        if self.jobs.is_empty() {
            return Err(ArrayifyError::CommandError("No jobs found.".into()));
        }

        // Log the jobs
        let timestamp = Local::now().format("%Y-%m-%d-%H-%M").to_string();
        let log_file_path = format!("{}/arrayify-{}.log", self.log_directory, timestamp);
        self.write_job_log(&log_file_path)?;

        let job_id = self.submit_jobs_to_scheduler(&log_file_path)?;

        self.print_run_stats(&log_file_path, &job_id);
        Ok(())
    }

    fn write_job_log(&self, log_file_path: &str) -> Result<(), ArrayifyError> {
        let mut log_file = File::create(log_file_path)?;
        if self.stdout_paths.is_empty() {
            for job_command in self.jobs.iter() {
                writeln!(log_file, "{}", job_command)?;
            }
        } else {
            for (job_command, stdout_path) in self.jobs.iter().zip(self.stdout_paths.iter()) {
                writeln!(log_file, "{}\t{}", job_command, stdout_path)?;
            }
        }
        Ok(())
    }

    fn print_run_stats(&self, log_file_path: &str, job_id: &str) {
        let message = format!(
            r#"🚀 Job submission complete! ✅
    🔖 Job ID is: {}
    📌 {} jobs submitted.
    📝 Job commands logged in: {}
    📂 Logs can be found in: {}
    📡 Track with -
    bjobs -A {}"#,
            job_id, self.jobs.len(), log_file_path, self.log_directory, job_id
        );

        println!("{}", message);
    }

    fn submit_jobs_to_scheduler(&self, job_file_path: &String) -> Result<String, ArrayifyError> {
        let memory_mb: usize = self.memory * 1000;
        // Count the number of lines in the file to determine the job array size
        let num_jobs = self.jobs.len();
        let job_array = format!("{}_job_array[1-{}]%{}", self.job_prefix, num_jobs, self.batch_size);
        let output_log = format!("{}/job_%J_%I.out", self.log_directory);
        let error_log = format!("{}/job_%J_%I.err", self.log_directory);

        // Generate the bsub command
        let bsub_cmd = format!(
            "bsub -J {} -q {} -n {} -M {} -R \"select[mem>{}] rusage[mem={}]\" -o {} -e {}",
            job_array, self.queue, self.threads, memory_mb, memory_mb, memory_mb, output_log, error_log
        );

        // Generate the script that uses `sed` to extract the job command from the file
        let script = if self.stdout_paths.is_empty() {
            format!(
                r#"#!/bin/bash

    INDEX=$((LSB_JOBINDEX - 1))
    COMMAND=$(sed -n "$((INDEX + 1))p" {})
    $COMMAND
    "#,
                job_file_path
            )
        } else {
            format!(
                r#"#!/bin/bash

    INDEX=$((LSB_JOBINDEX - 1))
    LINE=$(sed -n "$((INDEX + 1))p" {})
    COMMAND=$(printf '%s' "$LINE" | cut -f1)
    STDOUT=$(printf '%s' "$LINE" | cut -f2)
    mkdir -p "$(dirname "$STDOUT")"
    bash -c "$COMMAND" > "$STDOUT"
    "#,
                job_file_path
            )
        };

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
