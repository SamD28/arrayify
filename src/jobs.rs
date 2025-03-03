use csv::ReaderBuilder;
use std::collections::HashMap;
use std::fs::{self};
use std::io::{self};
use std::path::{Path, PathBuf};

pub fn read_jobs_from_csv(csv_file: &str, command_template: &str) -> io::Result<Vec<String>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(csv_file)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let headers = rdr
        .headers()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .clone();
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

pub fn read_jobs_from_dir(
    dir_path: &str,
    command_template: &str,
) -> io::Result<Vec<std::string::String>> {
    let dir = Path::new(dir_path);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    // Collect all files in the directory
    let mut file_map: HashMap<String, (Option<PathBuf>, Option<PathBuf>)> = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                if file_name.contains("_1") {
                    let id = file_name
                        .split("_1")
                        .next()
                        .unwrap_or(file_name)
                        .to_string();
                    file_map.entry(id).or_insert((None, None)).0 = Some(path.clone());
                } else if file_name.contains("_2") {
                    let id = file_name
                        .split("_2")
                        .next()
                        .unwrap_or(file_name)
                        .to_string();
                    file_map.entry(id).or_insert((None, None)).1 = Some(path.clone());
                }
            }
        }
    }

    // Validate and collect paired files
    let mut jobs = Vec::new();
    for (id, (r1, r2)) in file_map {
        if let (Some(r1_path), Some(r2_path)) = (r1, r2) {
            // Replace placeholders in the command template
            let job_command = command_template
                .replace("{ID}", &id)
                .replace("{R1}", r1_path.to_str().unwrap_or_default())
                .replace("{R2}", r2_path.to_str().unwrap_or_default());
            jobs.push(job_command);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Missing R1 or R2 for ID: {}", id),
            ));
        }
    }

    if jobs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No valid file pairs found in the directory",
        ));
    }

    Ok(jobs)
}
