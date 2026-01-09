use clap::{Parser, Subcommand};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
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
        #[arg(long, default_value = "https://hapi.fhir.org/baseR4")]
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
    // Read raw JSON file
    let raw = fs::read_to_string(fhir_file)?;
    let json: Value = serde_json::from_str(&raw)?;

    // Extract resourceType (required)
    let resource_type = json
        .get("resourceType")
        .and_then(|v| v.as_str())
        .ok_or("FHIR JSON must contain a string field 'resourceType'")?;

    // Build $validate URL: {base}/{resourceType}/$validate
    let url = format!(
        "{}/{}/$validate",
        base_url.trim_end_matches('/'),
        resource_type
    );

    // POST resource as FHIR JSON
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header(ACCEPT, "application/fhir+json")
        .header(CONTENT_TYPE, "application/fhir+json")
        .body(raw)
        .send()
        .await?;

    let status = resp.status();
    let body_text = resp.text().await?;

    // Try to parse response JSON; if it fails, show raw text
    let body_json: Value = match serde_json::from_str(&body_text) {
        Ok(v) => v,
        Err(_) => {
            println!("FHIR Validation");
            println!("--------------");
            println!("File: {}", fhir_file);
            println!("Base: {}", base_url);
            println!("HTTP: {}", status);
            println!("Result: FAIL ❌");
            println!("\nNon-JSON response:\n{}", body_text);
            return Ok(());
        }
    };

    let issues = body_json
        .get("issue")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let has_error_or_fatal = issues.iter().any(|issue| {
        issue
            .get("severity")
            .and_then(|s| s.as_str())
            .map(|sev| sev.eq_ignore_ascii_case("error") || sev.eq_ignore_ascii_case("fatal"))
            .unwrap_or(false)
    });

    println!("FHIR Validation");
    println!("--------------");
    println!("File: {}", fhir_file);
    println!("Base: {}", base_url);
    println!("HTTP: {}", status);

    if status.is_success() && !has_error_or_fatal {
        println!("Result: PASS ✅");
    } else {
        println!("Result: FAIL ❌");
    }

    if !issues.is_empty() {
        println!("\nIssues:");
        for (idx, issue) in issues.iter().enumerate() {
            let sev = issue
                .get("severity")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let code = issue
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Prefer diagnostics, otherwise try details.text
            let diag = issue
                .get("diagnostics")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    issue
                        .get("details")
                        .and_then(|d| d.get("text"))
                        .and_then(|t| t.as_str())
                })
                .unwrap_or("");

            println!("{}. [{}] {} {}", idx + 1, sev, code, diag);
        }
    } else {
        println!("\n(No issues returned)");
    }

    Ok(())
}
