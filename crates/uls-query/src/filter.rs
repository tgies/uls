//! Search filters for license queries.

use serde::{Deserialize, Serialize};

/// Filter criteria for license searches.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
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
    /// Sort order.
    pub sort: SortOrder,
}

impl SearchFilter {
    /// Create an empty filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter for callsign lookup.
    pub fn callsign(callsign: impl Into<String>) -> Self {
        Self {
            callsign: Some(callsign.into().to_uppercase()),
            ..Default::default()
        }
    }

    /// Create a filter for name search.
    pub fn name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into().to_uppercase()),
            ..Default::default()
        }
    }

    /// Create a filter for location search.
    pub fn location(city: Option<String>, state: Option<String>) -> Self {
        Self {
            city: city.map(|s| s.to_uppercase()),
            state: state.map(|s| s.to_uppercase()),
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
        self.state = Some(state.into().to_uppercase());
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

    /// Build the SQL WHERE clause for this filter.
    pub fn to_where_clause(&self) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(ref callsign) = self.callsign {
            if callsign.contains('*') || callsign.contains('?') {
                // Wildcard search
                let pattern = callsign.replace('*', "%").replace('?', "_");
                conditions.push("l.call_sign LIKE ?".to_string());
                params.push(pattern);
            } else {
                conditions.push("l.call_sign = ?".to_string());
                params.push(callsign.clone());
            }
        }

        if let Some(ref name) = self.name {
            conditions.push("(e.entity_name LIKE ? OR e.first_name LIKE ? OR e.last_name LIKE ?)".to_string());
            let pattern = format!("%{}%", name);
            params.push(pattern.clone());
            params.push(pattern.clone());
            params.push(pattern);
        }

        if let Some(ref city) = self.city {
            conditions.push("e.city LIKE ?".to_string());
            params.push(format!("%{}%", city));
        }

        if let Some(ref state) = self.state {
            conditions.push("e.state = ?".to_string());
            params.push(state.clone());
        }

        if let Some(ref zip) = self.zip_code {
            conditions.push("e.zip_code LIKE ?".to_string());
            params.push(format!("{}%", zip));
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

        let where_clause = if conditions.is_empty() {
            "1=1".to_string()
        } else {
            conditions.join(" AND ")
        };

        (where_clause, params)
    }

    /// Get the ORDER BY clause.
    pub fn order_clause(&self) -> &str {
        match self.sort {
            SortOrder::CallSign => "ORDER BY l.call_sign ASC",
            SortOrder::CallSignDesc => "ORDER BY l.call_sign DESC",
            SortOrder::Name => "ORDER BY e.entity_name ASC, e.last_name ASC",
            SortOrder::State => "ORDER BY e.state ASC, e.city ASC",
            SortOrder::GrantDate => "ORDER BY l.grant_date DESC",
            SortOrder::ExpirationDate => "ORDER BY l.expired_date ASC",
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
}
