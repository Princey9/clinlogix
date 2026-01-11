use std::fs;

use reqwest::StatusCode;

use crate::validate::types::FhirResource;

pub struct ValidateRequest {
    pub raw: String,
    pub resource_type: String,
}

pub struct ValidateResponse {
    pub status: StatusCode,
    pub body_text: String,
    pub url: String,
}

pub fn load_request(fhir_file: &str) -> Result<ValidateRequest, Box<dyn std::error::Error>> {
    let raw = fs::read_to_string(fhir_file)?;
    let resource: FhirResource = serde_json::from_str(&raw)?;

    Ok(ValidateRequest {
        raw,
        resource_type: resource.resource_type,
    })
}

pub async fn post_validate(
    request: &ValidateRequest,
    base_url: &str,
) -> Result<ValidateResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/{}/$validate",
        base_url.trim_end_matches('/'),
        request.resource_type
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Accept", "application/fhir+json")
        .header("Content-Type", "application/fhir+json")
        .body(request.raw.clone())
        .send()
        .await?;

    let status = response.status();
    let body_text = response.text().await?;

    Ok(ValidateResponse {
        status,
        body_text,
        url,
    })
}
