mod client;
mod report;
mod types;

pub async fn run_validate(
    fhir_file: &str,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = client::load_request(fhir_file)?;
    let response = client::post_validate(&request, base_url).await?;
    let outcome = report::parse_operation_outcome(&response.body_text);
    let report = report::build_report(&outcome, response.status, fhir_file, base_url);
    report::print_report(&report);
    Ok(())
}
