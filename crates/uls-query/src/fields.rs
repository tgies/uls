//! Field registry and generic filter expressions.
//!
//! Provides type-aware filtering and sorting for any registered field.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Field data types that determine allowed filter operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    /// String field: supports =, LIKE (wildcards * ?)
    String,
    /// Date field (YYYY-MM-DD): supports =, <, >, <=, >=
    Date,
    /// Single-char enum (status, class): supports =
    Char,
}

/// Comparison operators for filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOp {
    /// Exact match (=)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Le,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Ge,
    /// Pattern match (LIKE with wildcards)
    Like,
}

impl FilterOp {
    /// Parse operator from string prefix.
    pub fn parse(s: &str) -> (Self, &str) {
        if s.starts_with(">=") {
            (FilterOp::Ge, &s[2..])
        } else if s.starts_with("<=") {
            (FilterOp::Le, &s[2..])
        } else if s.starts_with("!=") {
            (FilterOp::Ne, &s[2..])
        } else if s.starts_with('>') {
            (FilterOp::Gt, &s[1..])
        } else if s.starts_with('<') {
            (FilterOp::Lt, &s[1..])
        } else if s.starts_with('=') {
            (FilterOp::Eq, &s[1..])
        } else {
            // No operator prefix = Eq
            (FilterOp::Eq, s)
        }
    }

    /// Check if this operator is valid for the given field type.
    pub fn valid_for(&self, field_type: FieldType) -> bool {
        match field_type {
            FieldType::String => matches!(self, FilterOp::Eq | FilterOp::Ne | FilterOp::Like),
            FieldType::Date => true, // All ops valid for dates
            FieldType::Char => matches!(self, FilterOp::Eq | FilterOp::Ne),
        }
    }

    /// Get SQL operator string.
    pub fn sql(&self) -> &'static str {
        match self {
            FilterOp::Eq => "=",
            FilterOp::Ne => "!=",
            FilterOp::Lt => "<",
            FilterOp::Le => "<=",
            FilterOp::Gt => ">",
            FilterOp::Ge => ">=",
            FilterOp::Like => "LIKE",
        }
    }
}

/// A single filter expression: field op value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterExpr {
    pub field: String,
    pub op: FilterOp,
    pub value: String,
}

impl FilterExpr {
    /// Parse a filter expression like "grant_date>2025-01-01" or "state=TX".
    pub fn parse(s: &str) -> Option<Self> {
        // Find the operator
        let op_chars = ['>', '<', '=', '!'];
        let op_pos = s.find(|c: char| op_chars.contains(&c))?;

        let field = s[..op_pos].trim().to_lowercase();
        let rest = &s[op_pos..];
        let (op, value_str) = FilterOp::parse(rest);
        let value = value_str.trim().to_string();

        if field.is_empty() || value.is_empty() {
            return None;
        }

        Some(FilterExpr { field, op, value })
    }
}

/// Field definition with SQL column mapping.
#[derive(Debug, Clone)]
pub struct FieldDef {
    /// User-facing field name (lowercase).
    pub name: &'static str,
    /// SQL column expression.
    pub column: &'static str,
    /// Field type for validation.
    pub field_type: FieldType,
    /// Aliases for this field.
    pub aliases: &'static [&'static str],
}

/// Registry of all searchable/sortable fields.
pub struct FieldRegistry {
    fields: HashMap<&'static str, FieldDef>,
}

impl FieldRegistry {
    /// Create the default field registry with all license fields.
    pub fn new() -> Self {
        let mut fields = HashMap::new();

        let defs = [
            FieldDef {
                name: "call_sign",
                column: "l.call_sign",
                field_type: FieldType::String,
                aliases: &["callsign", "call"],
            },
            FieldDef {
                name: "name",
                column: "e.entity_name",
                field_type: FieldType::String,
                aliases: &["entity_name", "licensee"],
            },
            FieldDef {
                name: "first_name",
                column: "e.first_name",
                field_type: FieldType::String,
                aliases: &["first"],
            },
            FieldDef {
                name: "last_name",
                column: "e.last_name",
                field_type: FieldType::String,
                aliases: &["last"],
            },
            FieldDef {
                name: "city",
                column: "e.city",
                field_type: FieldType::String,
                aliases: &[],
            },
            FieldDef {
                name: "state",
                column: "e.state",
                field_type: FieldType::String,
                aliases: &[],
            },
            FieldDef {
                name: "zip_code",
                column: "e.zip_code",
                field_type: FieldType::String,
                aliases: &["zip"],
            },
            FieldDef {
                name: "frn",
                column: "e.frn",
                field_type: FieldType::String,
                aliases: &[],
            },
            FieldDef {
                name: "status",
                column: "l.license_status",
                field_type: FieldType::Char,
                aliases: &["license_status"],
            },
            FieldDef {
                name: "class",
                column: "a.operator_class",
                field_type: FieldType::Char,
                aliases: &["operator_class"],
            },
            FieldDef {
                name: "service",
                column: "l.radio_service_code",
                field_type: FieldType::String,
                aliases: &["radio_service", "radio_service_code"],
            },
            FieldDef {
                name: "grant_date",
                column: "l.grant_date",
                field_type: FieldType::Date,
                aliases: &["granted"],
            },
            FieldDef {
                name: "expired_date",
                column: "l.expired_date",
                field_type: FieldType::Date,
                aliases: &["expires", "expiration"],
            },
            FieldDef {
                name: "cancellation_date",
                column: "l.cancellation_date",
                field_type: FieldType::Date,
                aliases: &["cancelled"],
            },
        ];

        for def in defs {
            // Register by name
            fields.insert(def.name, def.clone());
            // Register by aliases
            for &alias in def.aliases {
                fields.insert(alias, def.clone());
            }
        }

        FieldRegistry { fields }
    }

    /// Look up a field by name or alias.
    pub fn get(&self, name: &str) -> Option<&FieldDef> {
        self.fields.get(name.to_lowercase().as_str())
    }

    /// Get all canonical field names (no aliases).
    pub fn field_names(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self.fields.values().map(|f| f.name).collect();
        names.sort();
        names.dedup();
        names
    }
}

impl Default for FieldRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_op_parse() {
        assert_eq!(FilterOp::parse(">=2025"), (FilterOp::Ge, "2025"));
        assert_eq!(FilterOp::parse("<=2025"), (FilterOp::Le, "2025"));
        assert_eq!(FilterOp::parse(">2025"), (FilterOp::Gt, "2025"));
        assert_eq!(FilterOp::parse("<2025"), (FilterOp::Lt, "2025"));
        assert_eq!(FilterOp::parse("=TX"), (FilterOp::Eq, "TX"));
        assert_eq!(FilterOp::parse("TX"), (FilterOp::Eq, "TX"));
    }

    #[test]
    fn test_filter_expr_parse() {
        let expr = FilterExpr::parse("grant_date>2025-01-01").unwrap();
        assert_eq!(expr.field, "grant_date");
        assert_eq!(expr.op, FilterOp::Gt);
        assert_eq!(expr.value, "2025-01-01");

        let expr = FilterExpr::parse("state=TX").unwrap();
        assert_eq!(expr.field, "state");
        assert_eq!(expr.op, FilterOp::Eq);
        assert_eq!(expr.value, "TX");
    }

    #[test]
    fn test_field_registry() {
        let reg = FieldRegistry::new();

        // By name
        assert!(reg.get("call_sign").is_some());
        assert!(reg.get("grant_date").is_some());

        // By alias
        assert!(reg.get("callsign").is_some());
        assert!(reg.get("granted").is_some());
        assert!(reg.get("zip").is_some());

        // Unknown
        assert!(reg.get("unknown_field").is_none());
    }

    #[test]
    fn test_op_validity() {
        // String: only =, !=, LIKE
        assert!(FilterOp::Eq.valid_for(FieldType::String));
        assert!(FilterOp::Like.valid_for(FieldType::String));
        assert!(!FilterOp::Gt.valid_for(FieldType::String));

        // Date: all ops
        assert!(FilterOp::Gt.valid_for(FieldType::Date));
        assert!(FilterOp::Le.valid_for(FieldType::Date));

        // Char: only =, !=
        assert!(FilterOp::Eq.valid_for(FieldType::Char));
        assert!(!FilterOp::Gt.valid_for(FieldType::Char));
    }
}
