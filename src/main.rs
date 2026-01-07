use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// ClinLogix: analyze clinical and system logs for errors and warnings.
#[derive(Parser, Debug)]
#[command(name = "clinlogix", version, about)]
struct Args {
    /// Path to the log file
    logfile: String,

    /// Print only error lines
    #[arg(long)]
    errors_only: bool,

    /// Output summary as JSON
    #[arg(long)]
    json: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let file = File::open(&args.logfile)?;
    let reader = BufReader::new(file);

    let mut total_lines: u64 = 0;
    let mut error_count: u64 = 0;
    let mut warning_count: u64 = 0;

    for line in reader.lines() {
        let line = line?;
        total_lines += 1;

        let lower = line.to_lowercase();
        if lower.contains("error") {
            error_count += 1;

            if args.errors_only {
                println!("{line}");
            }
        } else if lower.contains("warning") {
            warning_count += 1;
        }
    }

    if args.errors_only {
        return Ok(());
    }

    if args.json {
        println!(
            "{{\"file\":\"{}\",\"total_lines\":{},\"errors\":{},\"warnings\":{}}}",
            args.logfile, total_lines, error_count, warning_count
        );
    } else {
        println!("ClinLogix Report");
        println!("----------------");
        println!("File: {}", args.logfile);
        println!("Total lines: {}", total_lines);
        println!("Errors: {}", error_count);
        println!("Warnings: {}", warning_count);
    }

    Ok(())
}
