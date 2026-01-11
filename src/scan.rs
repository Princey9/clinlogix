use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn run_scan(logfile: &str, errors_only: bool, json: bool) -> io::Result<()> {
    let file = File::open(logfile)?;
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

            if errors_only {
                println!("{line}");
            }
        } else if lower.contains("warning") {
            warning_count += 1;
        }
    }

    if errors_only {
        return Ok(());
    }

    if json {
        println!(
            "{{\"file\":\"{}\",\"total_lines\":{},\"errors\":{},\"warnings\":{}}}",
            logfile, total_lines, error_count, warning_count
        );
    } else {
        println!("ClinLogix Report");
        println!("----------------");
        println!("File: {}", logfile);
        println!("Total lines: {}", total_lines);
        println!("Errors: {}", error_count);
        println!("Warnings: {}", warning_count);
    }

    Ok(())
}
