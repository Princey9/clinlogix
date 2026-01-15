//! Library-style entry points for ClinLogix.
//!
//! This module re-exports existing functionality so it can be called programmatically.
//! The CLI behavior and output remain unchanged in `src/main.rs`.

use std::io;

pub use crate::scan::run_scan;
pub use crate::validate::{
    CodeableConcept, FhirResource, Issue, IssueSummary, OperationOutcome, PROFILE_RESOLUTION_THEME,
    ValidateRequest, ValidateResponse, ValidationReport, build_report, format_report, is_failure,
    load_request, parse_operation_outcome, post_validate, print_report, run_validate,
};

pub fn scan_log(logfile: &str, errors_only: bool, json: bool) -> io::Result<()> {
    crate::scan::run_scan(logfile, errors_only, json)
}

pub async fn validate_file(
    fhir_file: &str,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::validate::run_validate(fhir_file, base_url).await
}
