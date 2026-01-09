use clap::{Parser, Subcommand};
use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

/// ClinLogix: Health IT utility CLI (log scan + FHIR validation)
#[derive(Parser, Debug)]
#[command(name = "clinlogix", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan a log file and summarize errors/warnings (your original feature)
    Scan {
        /// Path to the log file
        logfile: String,

        /// Print only error lines
        #[arg(long)]
        errors_only: bool,

        /// Output summary as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate a FHIR JSON resource using a FHIR server ($validate)
    Validate {
        /// Path to a FHIR JSON file (e.g., examples/patient.json)
        fhir_file: String,

        /// FHIR base URL (defaults to public HAPI R4 server)
        #[arg(long, default_value = "https://server.fire.ly")]
base_url: String,

    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan {
            logfile,
            errors_only,
            json,
        } => run_scan(&logfile, errors_only, json)?,

        Commands::Validate { fhir_file, base_url } => run_validate(&fhir_file, &base_url).await?,
    }

    Ok(())
}

// ----------------------
// Scan (your existing logic, preserved)
// ----------------------
fn run_scan(logfile: &str, errors_only: bool, json: bool) -> io::Result<()> {
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

// ----------------------
// Validate (new feature)
// ----------------------
async fn run_validate(fhir_file: &str, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(fhir_file)?;
    let json: Value = serde_json::from_str(&raw)?;

    let resource_type = json
        .get("resourceType")
        .and_then(|v| v.as_str())
        .ok_or("FHIR JSON must contain a string field 'resourceType'")?;

    let url = format!(
        "{}/{}/$validate",
        base_url.trim_end_matches('/'),
        resource_type
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Accept", "application/fhir+json")
        .header("Content-Type", "application/fhir+json")
        .body(raw)
        .send()
        .await?;

    let status = resp.status();
    let body_text = resp.text().await?;

    let body_json: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);

    let issues = body_json
        .get("issue")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for issue in issues {
        let severity = issue
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let code = issue
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let message = issue
            .get("diagnostics")
            .and_then(|v| v.as_str())
            .or_else(|| {
                issue
                    .get("details")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
            })
            .unwrap_or("");

        let entry = format!("[{}] {} {}", severity, code, message);

        match severity {
            "error" | "fatal" => errors.push(entry),
            "warning" | "information" => warnings.push(entry),
            _ => {}
        }
    }

    println!("FHIR Validation");
    println!("--------------");
    println!("File: {}", fhir_file);
    println!("Base: {}", base_url);
    println!("HTTP: {}", status);

    if errors.is_empty() && status.is_success() {
        println!("Result: PASS ✅");
    } else {
        println!("Result: FAIL ❌");
    }

    if !errors.is_empty() {
        println!("\nErrors:");
        for (i, e) in errors.iter().enumerate() {
            println!("{}. {}", i + 1, e);
        }
    }

    if !warnings.is_empty() {
        println!("\nWarnings:");
        for (i, w) in warnings.iter().enumerate() {
            println!("{}. {}", i + 1, w);
        }
    }

    if errors.is_empty() && warnings.is_empty() {
        println!("\nNo issues reported.");
    }

    Ok(())
}
