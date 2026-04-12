use std::path::PathBuf;
use std::str::FromStr;

use crate::args::Cli;

#[derive(Debug, Clone)]
pub enum InputFormat {
    Csv(PathBuf),
    Directory(PathBuf),
}

impl FromStr for InputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(s);

        if path.is_dir() {
            return Ok(InputFormat::Directory(path));
        }
        
        /*
        I could do some checks to see if it's a valid,
        but for now I'll just assume that if it's not a directory,
        it's a CSV file
        */
        return Ok(InputFormat::Csv(path));
    }
}

#[derive(Debug, Clone)]
pub struct Job<'a> {
    pub input: &'a InputFormat,
    pub command: &'a String,
    pub jobs : Vec<String>,
    pub stdout_paths: Vec<String>,
    pub job_prefix: &'a String,
    pub log_directory: &'a String,
    pub memory: usize,
    pub threads: usize,
    pub queue: &'a String,
    pub batch_size: usize,
    pub stdout: Option<&'a String>,
}

impl<'a> Job<'a> {
    pub fn from_args(args: &'a Cli) -> Self {
        Job {
            input: &args.input,
            command: &args.command,
            jobs: Vec::new(),
            stdout_paths: Vec::new(),
            job_prefix: &args.job_prefix,
            log_directory: &args.log_directory,
            memory: args.memory,
            threads: args.threads,
            queue: &args.queue,
            batch_size: args.batch_size,
            stdout: args.stdout.as_ref(),
        }
    }

    pub fn calculate_batch_size(&mut self) {
        let num_jobs = self.jobs.len();
        let calculated = ((num_jobs as f64) * 0.2).ceil() as usize;
        self.batch_size = calculated.min(num_jobs);
    }
}