//! Search filters for license queries.

use serde::{Deserialize, Serialize};
use uls_core::codes::{LicenseStatus, OperatorClass, RadioService};

/// Convert enum field values from user-friendly strings to database integer codes.
///
/// For fields like `status`, `class`, and `service`, the database stores integer
/// codes, not the string representations. This function handles the conversion.
fn convert_enum_value(field_name: &str, value: &str) -> Option<String> {
    match field_name {
        "status" | "license_status" => {
            // Try parsing as LicenseStatus char (A, E, C, T, X)
            value
                .parse::<LicenseStatus>()
                .ok()
                .map(|s| s.to_u8().to_string())
        }
        "class" | "operator_class" => {
            // Try parsing as OperatorClass char (A, E, G, N, P, T)
            value
                .parse::<OperatorClass>()
                .ok()
                .map(|c| c.to_u8().to_string())
        }
        "service" | "radio_service" | "radio_service_code" => {
            // Try parsing as RadioService code (HA, HV, etc)
            value
                .parse::<RadioService>()
                .ok()
                .map(|s| s.to_u8().to_string())
        }
        _ => None, // Not an enum field, no conversion needed
    }
}

/// Result of analyzing a search pattern for wildcards.
#[derive(Debug, Clone, PartialEq)]
enum MatchPattern {
    /// Exact match (no wildcards)
    Exact(String),
    /// Pattern match using SQL LIKE
    Like(String),
}

impl MatchPattern {
    /// Analyze a search term and determine the matching strategy.
    ///
    /// Wildcards:
    /// - `*` matches any sequence of characters
    /// - `?` matches exactly one character
    ///
    /// Examples:
    /// - `SMITH` → Exact match
    /// - `SMITH*` → Prefix match (`SMITH%`)
    /// - `*SMITH` → Suffix match (`%SMITH`)
    /// - `*SMITH*` → Contains match (`%SMITH%`)
    /// - `SM?TH` → Single-char wildcard (`SM_TH`)
    fn from_search_term(term: &str) -> Self {
        if term.contains('*') || term.contains('?') {
            let pattern = term.replace('*', "%").replace('?', "_");
            MatchPattern::Like(pattern)
        } else {
            MatchPattern::Exact(term.to_string())
        }
    }
}

/// Generate a SQL condition and parameter for a text field match.
///
/// Returns (condition_string, parameters).
fn text_match_condition(column: &str, value: &str) -> (String, Vec<String>) {
    match MatchPattern::from_search_term(value) {
        MatchPattern::Exact(v) => (format!("{} = ?", column), vec![v]),
        MatchPattern::Like(pattern) => (format!("{} LIKE ?", column), vec![pattern]),
    }
}

/// Generate SQL condition for matching across multiple columns (OR).
///
/// Useful for name searches that span entity_name, first_name, last_name.
fn multi_column_match_condition(columns: &[&str], value: &str) -> (String, Vec<String>) {
    let pattern = MatchPattern::from_search_term(value);

    let (conditions, params): (Vec<_>, Vec<_>) = columns
        .iter()
        .map(|col| match &pattern {
            MatchPattern::Exact(v) => (format!("{} = ?", col), v.clone()),
            MatchPattern::Like(p) => (format!("{} LIKE ?", col), p.clone()),
        })
        .unzip();

    (format!("({})", conditions.join(" OR ")), params)
}

/// Filter criteria for license searches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    // ===== Generic filter system =====
    /// Generic filter expressions (e.g., "grant_date>2025-01-01", "state=TX").
    #[serde(default)]
    pub filters: Vec<crate::fields::FilterExpr>,
    /// Sort by field name (prefix with - for descending).
    pub sort_field: Option<String>,
    /// Sort direction (true = descending).
    #[serde(default)]
    pub sort_desc: bool,

    // ===== Legacy convenience fields (still supported) =====
    /// Filter by callsign pattern (supports wildcards).
    pub callsign: Option<String>,
    /// Filter by name (partial match).
    pub name: Option<String>,
    /// Filter by city.
    pub city: Option<String>,
    /// Filter by state (2-letter code).
    pub state: Option<String>,
    /// Filter by ZIP code.
    pub zip_code: Option<String>,
    /// Filter by radio service code(s).
    pub radio_service: Option<Vec<String>>,
    /// Filter by license status (A=Active, E=Expired, etc.).
    pub status: Option<char>,
    /// Filter by operator class (for amateur).
    pub operator_class: Option<char>,
    /// Only include active licenses.
    pub active_only: bool,
    /// FRN filter.
    pub frn: Option<String>,
    /// Maximum results to return.
    pub limit: Option<usize>,
    /// Number of results to skip (for pagination).
    pub offset: Option<usize>,
    /// Sort order (legacy enum, use sort_field for generic).
    pub sort: SortOrder,
    /// Filter by grant date (licenses granted on or after this date).
    pub granted_after: Option<String>,
    /// Filter by grant date (licenses granted on or before this date).
    pub granted_before: Option<String>,
    /// Filter by expiration date (licenses expiring on or before this date).
    pub expires_before: Option<String>,
}

impl SearchFilter {
    /// Create an empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter for callsign lookup.
    pub fn callsign(callsign: impl Into<String>) -> Self {
        Self {
            callsign: Some(callsign.into()),
            ..Default::default()
        }
    }

    /// Create a filter for name search.
    pub fn name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }

    /// Create a filter for location search.
    pub fn location(city: Option<String>, state: Option<String>) -> Self {
        Self {
            city,
            state,
            ..Default::default()
        }
    }

    /// Set the maximum results.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set pagination offset.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Only include active licenses.
    pub fn active_only(mut self) -> Self {
        self.active_only = true;
        self.status = Some('A');
        self
    }

    /// Filter by state.
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Filter by operator class.
    pub fn with_operator_class(mut self, class: char) -> Self {
        self.operator_class = Some(class);
        self
    }

    /// Set sort order.
    pub fn with_sort(mut self, sort: SortOrder) -> Self {
        self.sort = sort;
        self
    }

    /// Add a generic filter expression (e.g., "grant_date>2025-01-01").
    pub fn with_filter(mut self, expr: impl AsRef<str>) -> Self {
        if let Some(filter) = crate::fields::FilterExpr::parse(expr.as_ref()) {
            self.filters.push(filter);
        }
        self
    }

    /// Set sort field by name (prefix with - for descending).
    pub fn with_sort_field(mut self, field: impl Into<String>) -> Self {
        let field_str = field.into();
        if let Some(rest) = field_str.strip_prefix('-') {
            self.sort_field = Some(rest.to_string());
            self.sort_desc = true;
        } else {
            self.sort_field = Some(field_str);
            self.sort_desc = false;
        }
        self
    }

    /// Build the SQL WHERE clause for this filter.
    pub fn to_where_clause(&self) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        // Callsign - exact or wildcard match
        if let Some(ref callsign) = self.callsign {
            let (cond, p) = text_match_condition("l.call_sign", callsign);
            conditions.push(cond);
            params.extend(p);
        }

        // Name - search across entity_name, first_name, last_name
        if let Some(ref name) = self.name {
            let (cond, p) = multi_column_match_condition(
                &["e.entity_name", "e.first_name", "e.last_name"],
                name,
            );
            conditions.push(cond);
            params.extend(p);
        }

        // City - exact or wildcard match
        if let Some(ref city) = self.city {
            let (cond, p) = text_match_condition("e.city", city);
            conditions.push(cond);
            params.extend(p);
        }

        // State - always exact match (2-letter code)
        if let Some(ref state) = self.state {
            conditions.push("e.state = ?".to_string());
            params.push(state.clone());
        }

        // ZIP - prefix match by default (allows 5-digit or 9-digit)
        if let Some(ref zip) = self.zip_code {
            // If no wildcards, treat as prefix search
            let value = if zip.contains('*') || zip.contains('?') {
                zip.clone()
            } else {
                format!("{}*", zip)
            };
            let (cond, p) = text_match_condition("e.zip_code", &value);
            conditions.push(cond);
            params.extend(p);
        }

        // Convert status char to integer code for comparison
        if let Some(status) = self.status {
            if let Ok(status_enum) = status.to_string().parse::<LicenseStatus>() {
                conditions.push("l.license_status = ?".to_string());
                params.push(status_enum.to_u8().to_string());
            }
        } else if self.active_only {
            let active_code = LicenseStatus::Active.to_u8();
            conditions.push(format!("l.license_status = {}", active_code));
        }

        // Convert operator_class char to integer code for comparison
        if let Some(class) = self.operator_class {
            if let Ok(class_enum) = class.to_string().parse::<OperatorClass>() {
                conditions.push("a.operator_class = ?".to_string());
                params.push(class_enum.to_u8().to_string());
            }
        }

        if let Some(ref frn) = self.frn {
            conditions.push("e.frn = ?".to_string());
            params.push(frn.clone());
        }

        // Convert radio_service strings to integer codes for comparison
        if let Some(ref services) = self.radio_service {
            let codes: Vec<String> = services
                .iter()
                .filter_map(|s| {
                    s.parse::<RadioService>()
                        .ok()
                        .map(|r| r.to_u8().to_string())
                })
                .collect();
            if !codes.is_empty() {
                let placeholders: Vec<String> = codes.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!(
                    "l.radio_service_code IN ({})",
                    placeholders.join(", ")
                ));
                params.extend(codes);
            }
        }

        // Date range filters
        if let Some(ref date) = self.granted_after {
            conditions.push("l.grant_date >= ?".to_string());
            params.push(date.clone());
        }

        if let Some(ref date) = self.granted_before {
            conditions.push("l.grant_date <= ?".to_string());
            params.push(date.clone());
        }

        if let Some(ref date) = self.expires_before {
            conditions.push("l.expired_date <= ?".to_string());
            params.push(date.clone());
        }

        // Process generic filter expressions
        let registry = crate::fields::FieldRegistry::new();
        for expr in &self.filters {
            if let Some(field_def) = registry.get(&expr.field) {
                // Check wildcards for LIKE
                let op = if expr.value.contains('*') || expr.value.contains('?') {
                    crate::fields::FilterOp::Like
                } else {
                    expr.op
                };

                // Validate operator for field type
                if op.valid_for(field_def.field_type) {
                    if op == crate::fields::FilterOp::Like {
                        let pattern = expr.value.replace('*', "%").replace('?', "_");
                        conditions.push(format!("{} LIKE ?", field_def.column));
                        params.push(pattern);
                    } else {
                        // Convert enum values (status, class, service) to integer codes
                        let param_value = convert_enum_value(&expr.field, &expr.value)
                            .unwrap_or_else(|| expr.value.clone());
                        conditions.push(format!("{} {} ?", field_def.column, op.sql()));
                        params.push(param_value);
                    }
                }
            }
        }

        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        (where_clause, params)
    }

    /// Get the ORDER BY clause.
    pub fn order_clause(&self) -> String {
        // If sort_field is set, use generic field-based sorting
        if let Some(ref field_name) = self.sort_field {
            let registry = crate::fields::FieldRegistry::new();
            if let Some(field_def) = registry.get(field_name) {
                let dir = if self.sort_desc { "DESC" } else { "ASC" };
                return format!("ORDER BY {} {}", field_def.column, dir);
            }
        }

        // Fall back to legacy SortOrder enum
        match self.sort {
            SortOrder::CallSign => "ORDER BY l.call_sign ASC".to_string(),
            SortOrder::CallSignDesc => "ORDER BY l.call_sign DESC".to_string(),
            SortOrder::Name => "ORDER BY e.entity_name ASC, e.last_name ASC".to_string(),
            SortOrder::State => "ORDER BY e.state ASC, e.city ASC".to_string(),
            SortOrder::GrantDate => "ORDER BY l.grant_date DESC".to_string(),
            SortOrder::ExpirationDate => "ORDER BY l.expired_date ASC".to_string(),
        }
    }

    /// Get the LIMIT clause.
    pub fn limit_clause(&self) -> String {
        match (self.limit, self.offset) {
            (Some(limit), Some(offset)) => format!("LIMIT {} OFFSET {}", limit, offset),
            (Some(limit), None) => format!("LIMIT {}", limit),
            (None, Some(offset)) => format!("LIMIT -1 OFFSET {}", offset),
            (None, None) => String::new(),
        }
    }
}

/// Sort order for search results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    /// Sort by callsign ascending (default).
    #[default]
    CallSign,
    /// Sort by callsign descending.
    CallSignDesc,
    /// Sort by name.
    Name,
    /// Sort by state, then city.
    State,
    /// Sort by grant date (newest first).
    GrantDate,
    /// Sort by expiration date (soonest first).
    ExpirationDate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callsign_filter() {
        let filter = SearchFilter::callsign("W1AW");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("call_sign"));
        assert_eq!(params, vec!["W1AW"]);
    }

    #[test]
    fn test_wildcard_filter() {
        let filter = SearchFilter::callsign("W1*");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("LIKE"));
        assert_eq!(params, vec!["W1%"]);
    }

    #[test]
    fn test_name_filter() {
        let filter = SearchFilter::name("SMITH");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("entity_name"));
        assert_eq!(params.len(), 3); // entity_name, first_name, last_name
    }

    #[test]
    fn test_combined_filter() {
        let filter = SearchFilter::new()
            .with_state("CT")
            .with_operator_class('E')
            .active_only();

        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("state"));
        assert!(clause.contains("operator_class"));
        assert!(clause.contains("license_status"));
        assert!(params.contains(&"CT".to_string()));
        // operator_class 'E' = OperatorClass::Extra = code 3
        assert!(params.contains(&OperatorClass::Extra.to_u8().to_string()));
    }

    #[test]
    fn test_limit_offset() {
        let filter = SearchFilter::new().with_limit(50).with_offset(100);
        assert_eq!(filter.limit_clause(), "LIMIT 50 OFFSET 100");
    }

    #[test]
    fn test_limit_only() {
        let filter = SearchFilter::new().with_limit(25);
        assert_eq!(filter.limit_clause(), "LIMIT 25");
    }

    #[test]
    fn test_offset_only() {
        let filter = SearchFilter::new().with_offset(50);
        assert_eq!(filter.limit_clause(), "LIMIT -1 OFFSET 50");
    }

    #[test]
    fn test_location_filter() {
        // Use wildcards for contains match
        let filter =
            SearchFilter::location(Some("*NEWINGTON*".to_string()), Some("CT".to_string()));
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("city"));
        assert!(clause.contains("state"));
        assert!(params.contains(&"%NEWINGTON%".to_string()));
        assert!(params.contains(&"CT".to_string()));
    }

    #[test]
    fn test_frn_filter() {
        let mut filter = SearchFilter::new();
        filter.frn = Some("0001234567".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("frn"));
        assert!(params.contains(&"0001234567".to_string()));
    }

    #[test]
    fn test_zip_filter() {
        let mut filter = SearchFilter::new();
        filter.zip_code = Some("06111".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("zip_code"));
        assert!(params.contains(&"06111%".to_string()));
    }

    #[test]
    fn test_radio_service_filter() {
        let mut filter = SearchFilter::new();
        filter.radio_service = Some(vec!["HA".to_string(), "HV".to_string()]);
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("radio_service_code IN"));
        // HA = RadioService::HA, HV = RadioService::HV - check for integer codes
        assert!(params.contains(&RadioService::HA.to_u8().to_string()));
        assert!(params.contains(&RadioService::HV.to_u8().to_string()));
    }

    #[test]
    fn test_sort_orders() {
        let filter = SearchFilter::new().with_sort(SortOrder::CallSign);
        assert!(filter.order_clause().contains("call_sign ASC"));

        let filter = SearchFilter::new().with_sort(SortOrder::CallSignDesc);
        assert!(filter.order_clause().contains("call_sign DESC"));

        let filter = SearchFilter::new().with_sort(SortOrder::Name);
        assert!(filter.order_clause().contains("entity_name"));

        let filter = SearchFilter::new().with_sort(SortOrder::State);
        assert!(filter.order_clause().contains("state"));

        let filter = SearchFilter::new().with_sort(SortOrder::GrantDate);
        assert!(filter.order_clause().contains("grant_date"));

        let filter = SearchFilter::new().with_sort(SortOrder::ExpirationDate);
        assert!(filter.order_clause().contains("expired_date"));
    }

    #[test]
    fn test_empty_filter() {
        let filter = SearchFilter::new();
        let (clause, params) = filter.to_where_clause();
        assert_eq!(clause, "1=1");
        assert!(params.is_empty());
    }

    #[test]
    fn test_single_char_wildcard() {
        let filter = SearchFilter::callsign("W1A?");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("LIKE"));
        assert_eq!(params, vec!["W1A_"]);
    }

    #[test]
    fn test_match_pattern_exact() {
        let pattern = MatchPattern::from_search_term("SMITH");
        assert_eq!(pattern, MatchPattern::Exact("SMITH".to_string()));
    }

    #[test]
    fn test_match_pattern_prefix() {
        let pattern = MatchPattern::from_search_term("SMITH*");
        assert_eq!(pattern, MatchPattern::Like("SMITH%".to_string()));
    }

    #[test]
    fn test_match_pattern_suffix() {
        let pattern = MatchPattern::from_search_term("*SMITH");
        assert_eq!(pattern, MatchPattern::Like("%SMITH".to_string()));
    }

    #[test]
    fn test_match_pattern_contains() {
        let pattern = MatchPattern::from_search_term("*SMITH*");
        assert_eq!(pattern, MatchPattern::Like("%SMITH%".to_string()));
    }

    #[test]
    fn test_text_match_condition_exact() {
        let (cond, params) = text_match_condition("name", "SMITH");
        assert_eq!(cond, "name = ?");
        assert_eq!(params, vec!["SMITH"]);
    }

    #[test]
    fn test_text_match_condition_like() {
        let (cond, params) = text_match_condition("name", "SMITH*");
        assert_eq!(cond, "name LIKE ?");
        assert_eq!(params, vec!["SMITH%"]);
    }

    #[test]
    fn test_multi_column_match_exact() {
        let (cond, params) = multi_column_match_condition(&["a", "b", "c"], "VALUE");
        assert_eq!(cond, "(a = ? OR b = ? OR c = ?)");
        assert_eq!(params, vec!["VALUE", "VALUE", "VALUE"]);
    }

    #[test]
    fn test_multi_column_match_like() {
        let (cond, params) = multi_column_match_condition(&["a", "b"], "*VALUE*");
        assert_eq!(cond, "(a LIKE ? OR b LIKE ?)");
        assert_eq!(params, vec!["%VALUE%", "%VALUE%"]);
    }

    #[test]
    fn test_exact_city_match() {
        // No wildcards = exact match
        let mut filter = SearchFilter::new();
        filter.city = Some("NEWINGTON".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("city = ?"));
        assert_eq!(params, vec!["NEWINGTON"]);
    }

    // Case-insensitive tests - inputs are passed through unchanged,
    // relying on COLLATE NOCASE in the database schema
    #[test]
    fn test_lowercase_name_filter() {
        let filter = SearchFilter::name("smith");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("entity_name"));
        // Should contain lowercase - DB handles case-insensitivity
        assert!(params.iter().any(|p| p == "smith"));
    }

    #[test]
    fn test_lowercase_city_filter() {
        let mut filter = SearchFilter::new();
        filter.city = Some("newington".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("city = ?"));
        assert_eq!(params, vec!["newington"]);
    }

    #[test]
    fn test_lowercase_callsign_filter() {
        let filter = SearchFilter::callsign("w1aw");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("call_sign"));
        assert_eq!(params, vec!["w1aw"]);
    }

    #[test]
    fn test_mixed_case_wildcard_name() {
        let filter = SearchFilter::name("*Smith*");
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("LIKE"));
        // Wildcards converted to SQL LIKE pattern
        assert!(params.iter().any(|p| p == "%Smith%"));
    }

    #[test]
    fn test_with_filter() {
        let filter = SearchFilter::new().with_filter("grant_date>2025-01-01");
        assert_eq!(filter.filters.len(), 1);
        assert_eq!(filter.filters[0].field, "grant_date");
        assert_eq!(filter.filters[0].op, crate::fields::FilterOp::Gt);
        assert_eq!(filter.filters[0].value, "2025-01-01");
    }

    #[test]
    fn test_with_filter_invalid_ignored() {
        // Invalid filter expressions should be silently ignored
        let filter = SearchFilter::new().with_filter("invalid");
        assert_eq!(filter.filters.len(), 0);
    }

    #[test]
    fn test_with_sort_field_descending() {
        let filter = SearchFilter::new().with_sort_field("-call_sign");
        assert_eq!(filter.sort_field, Some("call_sign".to_string()));
        assert!(filter.sort_desc);

        // Test that order_clause produces DESC
        let clause = filter.order_clause();
        assert!(clause.contains("DESC"), "Expected DESC in: {}", clause);
    }

    #[test]
    fn test_with_sort_field_ascending() {
        let filter = SearchFilter::new().with_sort_field("grant_date");
        assert_eq!(filter.sort_field, Some("grant_date".to_string()));
        assert!(!filter.sort_desc);

        // Test that order_clause produces ASC
        let clause = filter.order_clause();
        assert!(clause.contains("ASC"), "Expected ASC in: {}", clause);
    }

    #[test]
    fn test_granted_after_filter() {
        let mut filter = SearchFilter::new();
        filter.granted_after = Some("2025-01-01".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("grant_date >="));
        assert!(params.contains(&"2025-01-01".to_string()));
    }

    #[test]
    fn test_granted_before_filter() {
        let mut filter = SearchFilter::new();
        filter.granted_before = Some("2025-12-31".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("grant_date <="));
        assert!(params.contains(&"2025-12-31".to_string()));
    }

    #[test]
    fn test_expires_before_filter() {
        let mut filter = SearchFilter::new();
        filter.expires_before = Some("2026-01-01".to_string());
        let (clause, params) = filter.to_where_clause();
        assert!(clause.contains("expired_date <="));
        assert!(params.contains(&"2026-01-01".to_string()));
    }

    #[test]
    fn test_date_range_combined() {
        let mut filter = SearchFilter::new();
        filter.granted_after = Some("2025-01-01".to_string());
        filter.granted_before = Some("2025-12-31".to_string());
        filter.expires_before = Some("2030-01-01".to_string());
        let (clause, params) = filter.to_where_clause();

        assert!(clause.contains("grant_date >="));
        assert!(clause.contains("grant_date <="));
        assert!(clause.contains("expired_date <="));
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_generic_filter_expression_date() {
        let filter = SearchFilter::new().with_filter("grant_date>2025-01-01");
        let (clause, params) = filter.to_where_clause();

        // Should contain the date filter
        assert!(clause.contains("grant_date"));
        assert!(clause.contains(">"));
        assert!(params.contains(&"2025-01-01".to_string()));
    }

    #[test]
    fn test_generic_filter_expression_with_wildcard() {
        // Wildcards in filter values should trigger LIKE
        let filter = SearchFilter::new().with_filter("city=NEW*");
        let (clause, params) = filter.to_where_clause();

        // Should use LIKE due to wildcard
        assert!(clause.contains("LIKE"));
        assert!(params.contains(&"NEW%".to_string()));
    }

    #[test]
    fn test_generic_filter_unknown_field() {
        // Unknown fields should be silently ignored
        let filter = SearchFilter::new().with_filter("unknown_field=value");
        let (clause, _params) = filter.to_where_clause();

        // Should not include unknown field, just default 1=1
        assert_eq!(clause, "1=1");
    }

    #[test]
    fn test_zip_with_explicit_wildcard() {
        // ZIP with explicit wildcard should preserve it
        let mut filter = SearchFilter::new();
        filter.zip_code = Some("061*".to_string());
        let (clause, params) = filter.to_where_clause();

        assert!(clause.contains("LIKE"));
        // Should use the explicit wildcard pattern, not add another
        assert!(params.contains(&"061%".to_string()));
    }

    #[test]
    fn test_empty_radio_service_list() {
        // Empty radio service list should not add a condition
        let mut filter = SearchFilter::new();
        filter.radio_service = Some(vec![]);
        let (clause, params) = filter.to_where_clause();

        // Should be empty filter (1=1)
        assert_eq!(clause, "1=1");
        assert!(params.is_empty());
    }

    #[test]
    fn test_generic_filter_enum_value_conversion() {
        // status=A should be converted to integer code (1)
        let filter = SearchFilter::new().with_filter("status=A");
        let (clause, params) = filter.to_where_clause();

        assert!(clause.contains("l.license_status"));
        // LicenseStatus::Active is code 0
        assert!(params.contains(&"0".to_string()));
    }

    #[test]
    fn test_generic_filter_unknown_enum_fallback() {
        // Unknown status value should fall back to original string
        let filter = SearchFilter::new().with_filter("status=UNKNOWN");
        let (clause, params) = filter.to_where_clause();

        assert!(clause.contains("l.license_status"));
        // Falls back to original string since parse failed
        assert!(params.contains(&"UNKNOWN".to_string()));
    }
}
