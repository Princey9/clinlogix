use std::collections::BTreeMap;

use reqwest::StatusCode;

use crate::validate::types::{Issue, OperationOutcome};

pub const PROFILE_RESOLUTION_THEME: &str = "Profile resolution (missing profiles on server)";
const PROFILE_RESOLUTION_HINT: &str = "This usually means the server doesn't have the required implementation guide packages (e.g., US Core) installed.";

#[derive(Debug, Clone)]
pub struct IssueSummary {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub location: Vec<String>,
    pub expression: Vec<String>,
    pub theme: String,
    pub line: Option<u32>,
}

pub struct ValidationReport {
    pub status: StatusCode,
    pub file: String,
    pub base_url: String,
    pub total: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub groups: BTreeMap<String, Vec<IssueSummary>>,
    pub theme_counts: BTreeMap<String, usize>,
}

pub fn parse_operation_outcome(body_text: &str) -> OperationOutcome {
    serde_json::from_str::<OperationOutcome>(body_text)
        .unwrap_or_else(|_| OperationOutcome::empty())
}

pub fn build_report(
    outcome: &OperationOutcome,
    status: StatusCode,
    file: &str,
    base_url: &str,
) -> ValidationReport {
    let mut groups: BTreeMap<String, Vec<IssueSummary>> = BTreeMap::new();
    let mut theme_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;

    for issue in &outcome.issue {
        let summary = summarize_issue(issue);
        match summary.severity.as_str() {
            "error" | "fatal" => error_count += 1,
            "warning" => warning_count += 1,
            "information" => info_count += 1,
            _ => {}
        }

        let key = group_key(&summary);
        groups.entry(key).or_default().push(summary.clone());
        *theme_counts.entry(summary.theme.clone()).or_insert(0) += 1;
    }

    ValidationReport {
        status,
        file: file.to_string(),
        base_url: base_url.to_string(),
        total: outcome.issue.len(),
        error_count,
        warning_count,
        info_count,
        groups,
        theme_counts,
    }
}

fn summarize_issue(issue: &Issue) -> IssueSummary {
    let severity = issue.severity.as_deref().unwrap_or("unknown").to_string();
    let code = issue.code.as_deref().unwrap_or("unknown").to_string();
    let message = issue
        .diagnostics
        .clone()
        .or_else(|| {
            issue
                .details
                .as_ref()
                .and_then(|details| details.text.clone())
        })
        .unwrap_or_default();
    let theme = classify_theme(&message);
    let line = extract_line_number(&message);

    IssueSummary {
        severity,
        code,
        message,
        location: issue.location.clone(),
        expression: issue.expression.clone(),
        theme,
        line,
    }
}

fn classify_theme(message: &str) -> String {
    if is_profile_resolution_issue(message) {
        PROFILE_RESOLUTION_THEME.to_string()
    } else {
        "General".to_string()
    }
}

fn is_profile_resolution_issue(message: &str) -> bool {
    let message_lower = message.to_lowercase();
    message_lower.contains("unable to resolve reference to profile")
        || (message_lower.contains("profile") && message_lower.contains("resolve"))
}

fn extract_line_number(message: &str) -> Option<u32> {
    let lower = message.to_lowercase();
    let needle = "line:";
    let start = lower.find(needle)?;
    let mut chars = lower[start + needle.len()..].chars().peekable();

    while let Some(ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }

    let mut digits = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_ascii_digit() {
            digits.push(*ch);
            chars.next();
        } else {
            break;
        }
    }

    if digits.is_empty() {
        None
    } else {
        digits.parse::<u32>().ok()
    }
}

pub fn format_report(report: &ValidationReport) -> String {
    let mut output = String::new();
    output.push_str("FHIR Validation\n");
    output.push_str("--------------\n");
    output.push_str(&format!("File: {}\n", report.file));
    output.push_str(&format!("Base: {}\n", report.base_url));
    output.push_str(&format!("HTTP: {}\n", report.status));
    output.push_str(&format!(
        "Issues: {} (errors: {}, warnings: {}, info: {})\n",
        report.total, report.error_count, report.warning_count, report.info_count
    ));
    output.push_str(&format!(
        "Total: {} issues in {} categories\n",
        report.total,
        report.groups.len()
    ));

    if report.error_count == 0 && report.status.is_success() {
        output.push_str("Result: PASS ✅\n");
    } else {
        output.push_str("Result: FAIL ❌\n");
    }

    output.push_str("Themes:\n");
    if report.theme_counts.is_empty() {
        output.push_str("  none\n");
    } else {
        for (index, (theme, count)) in sorted_theme_counts(&report.theme_counts).iter().enumerate()
        {
            output.push_str(&format!("  {}. {} (x{})\n", index + 1, theme, count));
            if theme == PROFILE_RESOLUTION_THEME {
                output.push_str(&format!("  {}\n", PROFILE_RESOLUTION_HINT));
            }
        }
    }

    if report.groups.is_empty() {
        output.push_str("\nNo issues reported.\n");
        return output;
    }

    output.push_str("\nIssue Categories (severity/code/message):\n");
    for (key, items) in sorted_groups(&report.groups) {
        let counts = summarize_severity(&items);
        let summary = format_severity_summary(&counts);
        output.push_str(&format!("- {} ({}): {}\n", key, items.len(), summary));
        for (index, item) in items.iter().enumerate() {
            let message = if item.message.is_empty() {
                "(no diagnostics provided)"
            } else {
                item.message.as_str()
            };
            output.push_str(&format!(
                "  {}. [{}] {}\n",
                index + 1,
                item.severity,
                message
            ));
            if !item.location.is_empty() {
                output.push_str(&format!("     location: {}\n", item.location.join(", ")));
            }
            if !item.expression.is_empty() {
                output.push_str(&format!(
                    "     expression: {}\n",
                    item.expression.join(", ")
                ));
                if let Some(line) = item.line {
                    output.push_str(&format!("     line: {}\n", line));
                }
            }
        }
    }

    output
}

pub fn print_report(report: &ValidationReport) {
    println!("{}", format_report(report));
}

fn sorted_theme_counts(counts: &BTreeMap<String, usize>) -> Vec<(String, usize)> {
    let mut entries: Vec<(String, usize)> = counts
        .iter()
        .map(|(theme, count)| (theme.clone(), *count))
        .collect();

    entries.sort_by(|(left_theme, left_count), (right_theme, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_theme.cmp(right_theme))
    });

    entries
}

fn group_key(summary: &IssueSummary) -> String {
    format!(
        "{} | {} | {}",
        summary.severity,
        summary.code,
        if summary.message.is_empty() {
            "(no diagnostics provided)"
        } else {
            summary.message.as_str()
        }
    )
}

fn summarize_severity(items: &[IssueSummary]) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in items {
        *counts.entry(item.severity.clone()).or_insert(0) += 1;
    }
    counts
}

fn sorted_groups(groups: &BTreeMap<String, Vec<IssueSummary>>) -> Vec<(String, Vec<IssueSummary>)> {
    let mut entries: Vec<(String, Vec<IssueSummary>)> = groups
        .iter()
        .map(|(key, items)| (key.clone(), items.clone()))
        .collect();

    entries.sort_by(|(left_key, left_items), (right_key, right_items)| {
        right_items
            .len()
            .cmp(&left_items.len())
            .then_with(|| left_key.cmp(right_key))
    });

    entries
}

fn format_severity_summary(counts: &BTreeMap<String, usize>) -> String {
    if counts.is_empty() {
        return "no severities".to_string();
    }
    counts
        .iter()
        .map(|(severity, count)| format!("{}: {}", severity, count))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue_summary(message: &str, severity: &str, theme: &str) -> IssueSummary {
        IssueSummary {
            severity: severity.to_string(),
            code: "invalid".to_string(),
            message: message.to_string(),
            location: vec![],
            expression: vec![],
            theme: theme.to_string(),
            line: None,
        }
    }

    #[test]
    fn groups_issues_by_code() {
        let outcome = OperationOutcome {
            resource_type: Some("OperationOutcome".to_string()),
            issue: vec![
                Issue {
                    severity: Some("error".to_string()),
                    code: Some("invalid".to_string()),
                    diagnostics: Some("Missing id".to_string()),
                    details: None,
                    location: vec!["Patient.id".to_string()],
                    expression: vec![],
                },
                Issue {
                    severity: Some("warning".to_string()),
                    code: Some("invalid".to_string()),
                    diagnostics: Some("Unknown system".to_string()),
                    details: None,
                    location: vec!["Patient.identifier".to_string()],
                    expression: vec![],
                },
            ],
        };

        let report = build_report(&outcome, StatusCode::BAD_REQUEST, "test.json", "base");
        assert_eq!(report.groups.len(), 2);
        let first_key = report
            .groups
            .keys()
            .find(|key| key.contains("Missing id"))
            .expect("missing id group");
        let second_key = report
            .groups
            .keys()
            .find(|key| key.contains("Unknown system"))
            .expect("unknown system group");
        assert_eq!(report.groups[first_key].len(), 1);
        assert_eq!(report.groups[second_key].len(), 1);
        assert_eq!(report.error_count, 1);
        assert_eq!(report.warning_count, 1);
    }

    #[test]
    fn parse_operation_outcome_fallbacks_on_invalid_json() {
        let outcome = parse_operation_outcome("not-json");
        assert!(outcome.issue.is_empty());
    }

    #[test]
    fn summarizes_severity_counts_per_group() {
        let items = vec![
            issue_summary("Missing id", "error", "General"),
            issue_summary("Unknown system", "warning", "General"),
            issue_summary("Deprecated", "warning", "General"),
        ];

        let counts = summarize_severity(&items);
        assert_eq!(counts.get("error"), Some(&1));
        assert_eq!(counts.get("warning"), Some(&2));
    }

    #[test]
    fn groups_same_key_together() {
        let items = vec![
            issue_summary("Missing id", "error", "General"),
            issue_summary("Missing id", "error", "General"),
        ];

        let mut groups: BTreeMap<String, Vec<IssueSummary>> = BTreeMap::new();
        for item in items {
            let key = group_key(&item);
            groups.entry(key).or_default().push(item);
        }

        assert_eq!(groups.len(), 1);
        let counts = groups.values().map(Vec::len).sum::<usize>();
        assert_eq!(counts, 2);
    }

    #[test]
    fn separates_groups_when_message_differs() {
        let items = vec![
            issue_summary("Missing id", "error", "General"),
            issue_summary("Unknown system", "error", "General"),
        ];

        let mut groups: BTreeMap<String, Vec<IssueSummary>> = BTreeMap::new();
        for item in items {
            let key = group_key(&item);
            groups.entry(key).or_default().push(item);
        }

        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn sorts_groups_by_count_descending() {
        let mut groups: BTreeMap<String, Vec<IssueSummary>> = BTreeMap::new();
        let high = issue_summary("Missing id", "error", "General");
        let low = issue_summary("Missing field", "warning", "General");
        let high_key = group_key(&high);
        let low_key = group_key(&low);
        groups.insert(high_key.clone(), vec![high.clone(), high]);
        groups.insert(low_key.clone(), vec![low]);

        let sorted = sorted_groups(&groups);
        assert_eq!(sorted[0].0, high_key);
        assert_eq!(sorted[0].1.len(), 2);
        assert_eq!(sorted[1].0, low_key);
        assert_eq!(sorted[1].1.len(), 1);
    }

    #[test]
    fn totals_match_sum_of_group_counts() {
        let outcome = OperationOutcome {
            resource_type: Some("OperationOutcome".to_string()),
            issue: vec![
                Issue {
                    severity: Some("error".to_string()),
                    code: Some("invalid".to_string()),
                    diagnostics: Some("Missing id".to_string()),
                    details: None,
                    location: vec![],
                    expression: vec![],
                },
                Issue {
                    severity: Some("error".to_string()),
                    code: Some("invalid".to_string()),
                    diagnostics: Some("Missing id".to_string()),
                    details: None,
                    location: vec![],
                    expression: vec![],
                },
                Issue {
                    severity: Some("warning".to_string()),
                    code: Some("incomplete".to_string()),
                    diagnostics: Some("Missing field".to_string()),
                    details: None,
                    location: vec![],
                    expression: vec![],
                },
            ],
        };

        let report = build_report(&outcome, StatusCode::BAD_REQUEST, "test.json", "base");
        let grouped_total: usize = report.groups.values().map(Vec::len).sum();
        assert_eq!(report.total, grouped_total);
    }

    #[test]
    fn group_key_is_exactly_severity_code_message() {
        let summary = IssueSummary {
            severity: "error".to_string(),
            code: "invalid".to_string(),
            message: "Missing id".to_string(),
            location: vec!["Patient.id".to_string()],
            expression: vec!["Patient.id".to_string()],
            theme: "General".to_string(),
            line: None,
        };

        let key = group_key(&summary);
        assert_eq!(key, "error | invalid | Missing id");
    }

    #[test]
    fn sorted_groups_matches_print_ordering() {
        let mut groups: BTreeMap<String, Vec<IssueSummary>> = BTreeMap::new();
        let high = issue_summary("Missing id", "error", "General");
        let mid = issue_summary("Missing field", "warning", "General");
        let low = issue_summary("FYI", "information", "General");
        let high_key = group_key(&high);
        let mid_key = group_key(&mid);
        let low_key = group_key(&low);
        groups.insert(high_key.clone(), vec![high.clone(), high]);
        groups.insert(mid_key.clone(), vec![mid.clone()]);
        groups.insert(low_key.clone(), vec![low.clone()]);

        let sorted = sorted_groups(&groups);
        let ordered_keys: Vec<String> = sorted.iter().map(|(key, _)| key.clone()).collect();
        assert_eq!(ordered_keys, vec![high_key, low_key, mid_key]);
    }

    #[test]
    fn parses_operation_outcome_and_groups_expected_counts() {
        let json = r#"{
            "resourceType":"OperationOutcome",
            "issue":[
                {"severity":"error","code":"invalid","diagnostics":"Missing id","location":["Patient.id"]},
                {"severity":"error","code":"invalid","diagnostics":"Missing id","location":["Patient.id"]},
                {"severity":"warning","code":"incomplete","diagnostics":"Missing field","location":["Patient.name"]},
                {"severity":"information","code":"informational","diagnostics":"FYI","location":["Patient.gender"]}
            ]
        }"#;

        let outcome: OperationOutcome = serde_json::from_str(json).expect("outcome parse");
        let report = build_report(&outcome, StatusCode::BAD_REQUEST, "test.json", "base");

        assert_eq!(report.total, 4);
        let grouped_total: usize = report.groups.values().map(Vec::len).sum();
        assert_eq!(report.total, grouped_total);

        let error_key = "error | invalid | Missing id";
        let warning_key = "warning | incomplete | Missing field";
        let info_key = "information | informational | FYI";
        assert_eq!(report.groups[error_key].len(), 2);
        assert_eq!(report.groups[warning_key].len(), 1);
        assert_eq!(report.groups[info_key].len(), 1);
    }

    #[test]
    fn classifies_profile_resolution_theme_with_line() {
        let json = r#"{
            "resourceType":"OperationOutcome",
            "issue":[
                {
                    "severity":"error",
                    "code":"invalid",
                    "diagnostics":"Unable to resolve reference to profile 'http://hl7.org/fhir/us/core/StructureDefinition/us-core-bmi'. (line: 6)"
                }
            ]
        }"#;

        let outcome: OperationOutcome = serde_json::from_str(json).expect("outcome parse");
        let report = build_report(&outcome, StatusCode::BAD_REQUEST, "test.json", "base");
        assert_eq!(
            report.theme_counts[PROFILE_RESOLUTION_THEME], 1,
            "expected profile resolution theme count"
        );
        assert_eq!(report.groups.values().next().unwrap()[0].line, Some(6));
    }

    #[test]
    fn classifies_profile_resolution_when_phrase_missing() {
        let outcome = OperationOutcome {
            resource_type: Some("OperationOutcome".to_string()),
            issue: vec![Issue {
                severity: Some("error".to_string()),
                code: Some("invalid".to_string()),
                diagnostics: Some("Failed to resolve profile reference".to_string()),
                details: None,
                location: vec![],
                expression: vec![],
            }],
        };

        let report = build_report(&outcome, StatusCode::BAD_REQUEST, "test.json", "base");
        assert_eq!(report.theme_counts[PROFILE_RESOLUTION_THEME], 1);
    }
}
