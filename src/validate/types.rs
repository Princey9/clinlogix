use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct FhirResource {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
}

#[derive(Debug, Deserialize)]
pub struct OperationOutcome {
    #[serde(rename = "resourceType")]
    #[allow(dead_code)]
    pub resource_type: Option<String>,
    #[serde(default)]
    pub issue: Vec<Issue>,
}

impl OperationOutcome {
    pub fn empty() -> Self {
        Self {
            resource_type: None,
            issue: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    pub severity: Option<String>,
    pub code: Option<String>,
    pub diagnostics: Option<String>,
    pub details: Option<CodeableConcept>,
    #[serde(default)]
    pub location: Vec<String>,
    #[serde(default)]
    pub expression: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CodeableConcept {
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_fhir_resource_type() {
        let json = r#"{"resourceType":"Patient"}"#;
        let resource: FhirResource = serde_json::from_str(json).expect("resource parse");
        assert_eq!(resource.resource_type, "Patient");
    }

    #[test]
    fn parse_operation_outcome_issues() {
        let json = r#"{
            "resourceType":"OperationOutcome",
            "issue":[
                {"severity":"error","code":"invalid","diagnostics":"Missing id"}
            ]
        }"#;
        let outcome: OperationOutcome = serde_json::from_str(json).expect("outcome parse");
        assert_eq!(outcome.issue.len(), 1);
        assert_eq!(outcome.issue[0].code.as_deref(), Some("invalid"));
    }
}
