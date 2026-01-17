//! Search filters for license queries.

use serde::{Deserialize, Serialize};

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
        MatchPattern::Exact(v) => {
            (format!("{} = ?", column), vec![v])
        }
        MatchPattern::Like(pattern) => {
            (format!("{} LIKE ?", column), vec![pattern])
        }
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
            city: city,
            state: state,
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
        if field_str.starts_with('-') {
            self.sort_field = Some(field_str[1..].to_string());
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

        if let Some(status) = self.status {
            conditions.push("l.license_status = ?".to_string());
            params.push(status.to_string());
        } else if self.active_only {
            conditions.push("l.license_status = 'A'".to_string());
        }

        if let Some(class) = self.operator_class {
            conditions.push("a.operator_class = ?".to_string());
            params.push(class.to_string());
        }

        if let Some(ref frn) = self.frn {
            conditions.push("e.frn = ?".to_string());
            params.push(frn.clone());
        }

        if let Some(ref services) = self.radio_service {
            if !services.is_empty() {
                let placeholders: Vec<String> = services.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("l.radio_service_code IN ({})", placeholders.join(", ")));
                params.extend(services.iter().cloned());
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
                        conditions.push(format!("{} {} ?", field_def.column, op.sql()));
                        params.push(expr.value.clone());
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
        assert!(params.contains(&"E".to_string()));
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
        let filter = SearchFilter::location(Some("*NEWINGTON*".to_string()), Some("CT".to_string()));
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
        assert!(params.contains(&"HA".to_string()));
        assert!(params.contains(&"HV".to_string()));
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
}


