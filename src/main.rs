mod args;
mod error;
mod jobs;
mod submission;
mod types;
use clap::Parser;

use crate::error::ArrayifyError;

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn run() -> Result<(), ArrayifyError> {
    let args = args::Cli::parse();

    let mut job = types::Job::from_args(&args);
    
    job.generate_jobs();
    job.calculate_batch_size();

    job.submit_jobs().map_err(|e| ArrayifyError::FarmError(e.to_string()))?;

    return Ok(());
}
