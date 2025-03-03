# arrayify

![Rust Version](https://img.shields.io/badge/Rust-1.85.0-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)
![Issues](https://img.shields.io/github/issues/SamD28/arrayify)
![Tests](https://github.com/SamD28/arrayify/actions/workflows/rust.yml/badge.svg)

arrayify is a command-line tool for submitting and managing LSF job arrays using a CSV file.

This is a Rust project with automated CI/CD using GitHub Actions.

## Installation

Ensure you have Rust installed, then build and install the tool:

```
cargo build --release
```
```
cp target/release/arrayify /usr/local/bin/
```

Alternatively, run directly with:

```
cargo run -- sub --csv jobs.csv --command "echo {ID} {R1} {R2}"
```

## Usage

Submit Job Array

```
arrayify sub --csv <CSV_FILE> --command "<COMMAND_TEMPLATE>" [OPTIONS]
```

Required Arguments

```
-s, --csv <CSV_FILE>
```

Path to the CSV file containing job parameters.

```
-c, --command <COMMAND_TEMPLATE>
```

Command template with placeholders matching CSV headers.

Example: ```"echo {ID} {R1} {R2}"```

Optional Arguments

```
-l, --log <LOG_DIR>
```

Path to store log files (default: logs).

```
-t, --threads <THREADS>
```

Number of threads per job (default: 1).

```
-b, --batch <BATCH_SIZE>Number of concurrently running jobs:
```

Default: 20% of total jobs (rounded up).

Set an explicit number to override auto-batching.

Example Submission

```
arrayify sub --csv jobs.csv \
  --command "echo {ID} {SAMPLE} {FASTQ}" \
  --log my_logs --memory 4 --threads 2 --batch 10
```

Check Job Status

```
arrayify check <JOB_ID>
```

Example

```
arrayify check 12345
```

## How It Works

1. Parses the CSV file to extract job parameters.

2. Replaces placeholders in the command template with CSV values.

3. Generates and submits a job array using bsub.

4. Handles batch size dynamically:

    - If --batch auto, it runs 20% of jobs simultaneously.

    - If a number is provided, it runs that many concurrently.

5. Logs output and errors to the specified directory.

6. Allows job status checking using bjobs.

## Troubleshooting

Jobs not running? Check LSF queue status with:

```
bjobs -u $USER
```

Need more memory? Increase --memory based on job requirements.

## License

MIT License
