mod client;
mod report;
mod types;

#[allow(unused_imports)]
pub use client::{ValidateRequest, ValidateResponse, load_request, post_validate};
#[allow(unused_imports)]
pub use report::{
    IssueSummary, PROFILE_RESOLUTION_THEME, ValidationReport, build_report, format_report,
    is_failure, parse_operation_outcome, print_report,
};
#[allow(unused_imports)]
pub use types::{CodeableConcept, FhirResource, Issue, OperationOutcome};

pub async fn run_validate(
    fhir_file: &str,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = client::load_request(fhir_file)?;
    let response = client::post_validate(&request, base_url).await?;
    let outcome = report::parse_operation_outcome(&response.body_text);
    let report = report::build_report(
        &outcome,
        response.status,
        fhir_file,
        base_url,
        &response.url,
    );
    report::print_report(&report);
    if report::is_failure(&report) {
        return Err("FHIR validation failed".into());
    }
    Ok(())
}
