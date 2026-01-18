//! Output formatting for license data.

use uls_db::models::License;

/// Supported output formats.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable table format.
    #[default]
    Table,
    /// JSON format.
    Json,
    /// JSON with pretty printing.
    JsonPretty,
    /// CSV format.
    Csv,
    /// YAML format.
    Yaml,
    /// Single-line compact format.
    Compact,
}

impl OutputFormat {
    /// Parse from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(OutputFormat::Table),
            "json" => Some(OutputFormat::Json),
            "json-pretty" | "jsonpretty" => Some(OutputFormat::JsonPretty),
            "csv" => Some(OutputFormat::Csv),
            "yaml" | "yml" => Some(OutputFormat::Yaml),
            "compact" | "oneline" => Some(OutputFormat::Compact),
            _ => None,
        }
    }
}

/// Trait for formatting output.
pub trait FormatOutput {
    /// Format as the given output format.
    fn format(&self, format: OutputFormat) -> String;
}

impl FormatOutput for License {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Table => format_license_table(self),
            OutputFormat::Json => serde_json::to_string(self).unwrap_or_default(),
            OutputFormat::JsonPretty => serde_json::to_string_pretty(self).unwrap_or_default(),
            OutputFormat::Csv => format_license_csv(self),
            OutputFormat::Yaml => format_license_yaml(self),
            OutputFormat::Compact => format_license_compact(self),
        }
    }
}

impl FormatOutput for Vec<License> {
    fn format(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Table => format_licenses_table(self),
            OutputFormat::Json => serde_json::to_string(self).unwrap_or_default(),
            OutputFormat::JsonPretty => serde_json::to_string_pretty(self).unwrap_or_default(),
            OutputFormat::Csv => format_licenses_csv(self),
            OutputFormat::Yaml => format_licenses_yaml(self),
            OutputFormat::Compact => self
                .iter()
                .map(|l| format_license_compact(l))
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// Format a single license as a table.
fn format_license_table(license: &License) -> String {
    let mut output = String::new();
    output.push_str(&format!("Call Sign:      {}\n", license.call_sign));
    output.push_str(&format!("Name:           {}\n", license.display_name()));
    output.push_str(&format!(
        "Status:         {} ({})\n",
        license.status,
        license.status_description()
    ));
    output.push_str(&format!("Service:        {}\n", license.radio_service));

    if let Some(class) = license.operator_class_description() {
        output.push_str(&format!("Operator Class: {}\n", class));
    }

    if let Some(ref addr) = license.street_address {
        output.push_str(&format!("Address:        {}\n", addr));
    }

    let location = format_location(license);
    if !location.is_empty() {
        output.push_str(&format!("Location:       {}\n", location));
    }

    if let Some(ref frn) = license.frn {
        output.push_str(&format!("FRN:            {}\n", frn));
    }

    if let Some(date) = license.grant_date {
        output.push_str(&format!("Granted:        {}\n", date));
    }

    if let Some(date) = license.expired_date {
        output.push_str(&format!("Expires:        {}\n", date));
    }

    output
}

/// Format multiple licenses as a table.
fn format_licenses_table(licenses: &[License]) -> String {
    if licenses.is_empty() {
        return "No results found.\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{:<10} {:<30} {:<6} {:<5} {:<20}\n",
        "CALL", "NAME", "STATUS", "CLASS", "LOCATION"
    ));
    output.push_str(&format!(
        "{:-<10} {:-<30} {:-<6} {:-<5} {:-<20}\n",
        "", "", "", "", ""
    ));

    for license in licenses {
        let class = license
            .operator_class
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string());
        let location = format!(
            "{}, {}",
            license.city.as_deref().unwrap_or("-"),
            license.state.as_deref().unwrap_or("-")
        );

        output.push_str(&format!(
            "{:<10} {:<30} {:<6} {:<5} {:<20}\n",
            license.call_sign,
            truncate(&license.display_name(), 30),
            license.status,
            class,
            truncate(&location, 20)
        ));
    }

    output.push_str(&format!("\n{} result(s)\n", licenses.len()));
    output
}

/// Format a license as compact one-liner.
fn format_license_compact(license: &License) -> String {
    let class = license
        .operator_class
        .map(|c| format!(" ({})", c))
        .unwrap_or_default();
    format!(
        "{}{} - {} [{}]",
        license.call_sign,
        class,
        license.display_name(),
        license.status_description()
    )
}

/// Format a license as CSV row.
fn format_license_csv(license: &License) -> String {
    format!(
        "{},{},{},{},{},{},{},{},{}",
        csv_escape(&license.call_sign),
        csv_escape(&license.display_name()),
        license.status,
        &license.radio_service,
        license
            .operator_class
            .map(|c| c.to_string())
            .unwrap_or_default(),
        csv_escape(license.city.as_deref().unwrap_or("")),
        csv_escape(license.state.as_deref().unwrap_or("")),
        license
            .grant_date
            .map(|d| d.to_string())
            .unwrap_or_default(),
        license
            .expired_date
            .map(|d| d.to_string())
            .unwrap_or_default()
    )
}

/// Format multiple licenses as CSV.
fn format_licenses_csv(licenses: &[License]) -> String {
    let mut output =
        String::from("call_sign,name,status,service,class,city,state,grant_date,expiration_date\n");
    for license in licenses {
        output.push_str(&format_license_csv(license));
        output.push('\n');
    }
    output
}

/// Format a license as YAML.
fn format_license_yaml(license: &License) -> String {
    // Simple YAML-like format
    let mut output = String::new();
    output.push_str(&format!("call_sign: {}\n", license.call_sign));
    output.push_str(&format!("name: {}\n", license.display_name()));
    output.push_str(&format!("status: {}\n", license.status));
    output.push_str(&format!("service: {}\n", license.radio_service));
    if let Some(class) = license.operator_class {
        output.push_str(&format!("operator_class: {}\n", class));
    }
    if let Some(ref city) = license.city {
        output.push_str(&format!("city: {}\n", city));
    }
    if let Some(ref state) = license.state {
        output.push_str(&format!("state: {}\n", state));
    }
    output
}

/// Format multiple licenses as YAML.
fn format_licenses_yaml(licenses: &[License]) -> String {
    let mut output = String::from("licenses:\n");
    for license in licenses {
        output.push_str("  - ");
        let yaml = format_license_yaml(license);
        let lines: Vec<&str> = yaml.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                output.push_str(line);
                output.push('\n');
            } else {
                output.push_str("    ");
                output.push_str(line);
                output.push('\n');
            }
        }
    }
    output
}

/// Format location string.
fn format_location(license: &License) -> String {
    let parts: Vec<&str> = [
        license.city.as_deref(),
        license.state.as_deref(),
        license.zip_code.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect();

    if parts.is_empty() {
        String::new()
    } else if parts.len() >= 2 {
        format!(
            "{}, {} {}",
            parts[0],
            parts.get(1).unwrap_or(&""),
            parts.get(2).unwrap_or(&"")
        )
        .trim()
        .to_string()
    } else {
        parts[0].to_string()
    }
}

/// Truncate a string to max length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Escape a value for CSV.
fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_license() -> License {
        License {
            unique_system_identifier: 123,
            call_sign: "W1TEST".to_string(),
            licensee_name: "Test User".to_string(),
            first_name: Some("Test".to_string()),
            middle_initial: None,
            last_name: Some("User".to_string()),
            status: 'A',
            radio_service: "HA".to_string(),
            grant_date: None,
            expired_date: None,
            cancellation_date: None,
            frn: Some("0001234567".to_string()),
            street_address: Some("123 Main St".to_string()),
            city: Some("NEWINGTON".to_string()),
            state: Some("CT".to_string()),
            zip_code: Some("06111".to_string()),
            operator_class: Some('E'),
            previous_call_sign: None,
        }
    }

    #[test]
    fn test_table_format() {
        let license = test_license();
        let output = license.format(OutputFormat::Table);
        assert!(output.contains("W1TEST"));
        assert!(output.contains("Test User"));
        assert!(output.contains("NEWINGTON"));
    }

    #[test]
    fn test_compact_format() {
        let license = test_license();
        let output = license.format(OutputFormat::Compact);
        assert!(output.contains("W1TEST"));
        assert!(output.contains("(E)"));
    }

    #[test]
    fn test_csv_format() {
        let license = test_license();
        let output = license.format(OutputFormat::Csv);
        assert!(output.contains("W1TEST"));
        assert!(output.contains("NEWINGTON"));
    }

    #[test]
    fn test_csv_escape() {
        assert_eq!(csv_escape("simple"), "simple");
        assert_eq!(csv_escape("with,comma"), "\"with,comma\"");
        assert_eq!(csv_escape("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_json_format() {
        let license = test_license();
        let output = license.format(OutputFormat::Json);
        assert!(output.contains("W1TEST"));
        assert!(output.contains("unique_system_identifier"));
    }

    #[test]
    fn test_json_pretty_format() {
        let license = test_license();
        let output = license.format(OutputFormat::JsonPretty);
        assert!(output.contains("W1TEST"));
        assert!(output.contains("\n")); // Pretty format has newlines
    }

    #[test]
    fn test_yaml_format() {
        let license = test_license();
        let output = license.format(OutputFormat::Yaml);
        assert!(output.contains("call_sign: W1TEST"));
        assert!(output.contains("status: A"));
    }

    #[test]
    fn test_vec_table_format() {
        let licenses = vec![test_license()];
        let output = licenses.format(OutputFormat::Table);
        assert!(output.contains("W1TEST"));
        assert!(output.contains("CALL")); // Header
        assert!(output.contains("1 result"));
    }

    #[test]
    fn test_vec_empty_table() {
        let licenses: Vec<License> = vec![];
        let output = licenses.format(OutputFormat::Table);
        assert!(output.contains("No results found"));
    }

    #[test]
    fn test_vec_csv_format() {
        let licenses = vec![test_license()];
        let output = licenses.format(OutputFormat::Csv);
        assert!(output.contains("call_sign,name")); // Header
        assert!(output.contains("W1TEST"));
    }

    #[test]
    fn test_vec_yaml_format() {
        let licenses = vec![test_license()];
        let output = licenses.format(OutputFormat::Yaml);
        assert!(output.contains("licenses:"));
        assert!(output.contains("call_sign: W1TEST"));
    }

    #[test]
    fn test_vec_compact_format() {
        let licenses = vec![test_license()];
        let output = licenses.format(OutputFormat::Compact);
        assert!(output.contains("W1TEST"));
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("table"), Some(OutputFormat::Table));
        assert_eq!(OutputFormat::from_str("json"), Some(OutputFormat::Json));
        assert_eq!(
            OutputFormat::from_str("json-pretty"),
            Some(OutputFormat::JsonPretty)
        );
        assert_eq!(OutputFormat::from_str("csv"), Some(OutputFormat::Csv));
        assert_eq!(OutputFormat::from_str("yaml"), Some(OutputFormat::Yaml));
        assert_eq!(
            OutputFormat::from_str("compact"),
            Some(OutputFormat::Compact)
        );
        assert_eq!(OutputFormat::from_str("unknown"), None);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_csv_escape_newline() {
        assert_eq!(csv_escape("with\nnewline"), "\"with\nnewline\"");
    }

    // ==========================================================================
    // JSON output validation tests
    // ==========================================================================

    #[test]
    fn test_json_format_is_valid_json() {
        let license = test_license();
        let output = license.format(OutputFormat::Json);
        let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&output);
        assert!(
            parsed.is_ok(),
            "JSON output should be valid JSON: {}",
            output
        );
    }

    #[test]
    fn test_json_pretty_format_is_valid_json() {
        let license = test_license();
        let output = license.format(OutputFormat::JsonPretty);
        let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(&output);
        assert!(
            parsed.is_ok(),
            "JSON-pretty output should be valid JSON: {}",
            output
        );
    }

    #[test]
    fn test_vec_json_format_is_valid_json_array() {
        let licenses = vec![test_license(), test_license()];
        let output = licenses.format(OutputFormat::Json);
        let parsed: serde_json::Result<Vec<serde_json::Value>> = serde_json::from_str(&output);
        assert!(
            parsed.is_ok(),
            "Vec JSON output should be valid JSON array: {}",
            output
        );
        assert_eq!(parsed.unwrap().len(), 2);
    }

    #[test]
    fn test_vec_json_pretty_format_is_valid_json_array() {
        let licenses = vec![test_license()];
        let output = licenses.format(OutputFormat::JsonPretty);
        let parsed: serde_json::Result<Vec<serde_json::Value>> = serde_json::from_str(&output);
        assert!(
            parsed.is_ok(),
            "Vec JSON-pretty output should be valid JSON array: {}",
            output
        );
    }

    #[test]
    fn test_empty_vec_json_format_is_valid_json() {
        let licenses: Vec<License> = vec![];
        let output = licenses.format(OutputFormat::Json);
        let parsed: serde_json::Result<Vec<serde_json::Value>> = serde_json::from_str(&output);
        assert!(parsed.is_ok(), "Empty vec JSON should be valid: {}", output);
        assert_eq!(parsed.unwrap().len(), 0);
    }
}
