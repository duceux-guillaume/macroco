use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct HistoricalDataPoint {
    pub year: f64,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoricalVariable {
    pub variable: String,
    pub source: String,
    pub units: String,
    pub transformation: String,
    pub data: Vec<HistoricalDataPoint>,
}

// ---------------------------------------------------------------------------
// CSV parser
// ---------------------------------------------------------------------------

/// Parse a historical CSV file with comment-header metadata.
///
/// Expected format:
/// ```text
/// # source: World Bank SP.POP.TOTL
/// # url: https://...
/// # units: persons
/// # transformation: none
/// # retrieved: 2026-03-04
/// year,value
/// 1960,3021512598
/// ```
///
/// Lines starting with `#` are metadata comments. The first non-comment,
/// non-empty line is treated as the header row and skipped. Subsequent lines
/// are parsed as `year,value` pairs; malformed lines are silently skipped.
pub fn parse_historical_csv(variable_id: &str, content: &str) -> HistoricalVariable {
    let mut source = String::new();
    let mut units = String::new();
    let mut transformation = String::new();
    let mut data = Vec::new();
    let mut header_seen = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Parse comment metadata
        if trimmed.starts_with('#') {
            let comment = trimmed.trim_start_matches('#').trim();
            if let Some(val) = comment.strip_prefix("source:") {
                source = val.trim().to_string();
            } else if let Some(val) = comment.strip_prefix("units:") {
                units = val.trim().to_string();
            } else if let Some(val) = comment.strip_prefix("transformation:") {
                transformation = val.trim().to_string();
            }
            // Other comment lines (url, retrieved, etc.) are ignored
            continue;
        }

        // First non-comment, non-empty line is the header — skip it
        if !header_seen {
            header_seen = true;
            continue;
        }

        // Parse data rows
        let mut parts = trimmed.splitn(2, ',');
        let year_str = match parts.next() {
            Some(s) => s.trim(),
            None => continue,
        };
        let value_str = match parts.next() {
            Some(s) => s.trim(),
            None => continue,
        };

        let year = match year_str.parse::<f64>() {
            Ok(y) if y.is_finite() => y,
            _ => continue,
        };
        let value = match value_str.parse::<f64>() {
            Ok(v) if v.is_finite() => v,
            _ => continue,
        };

        data.push(HistoricalDataPoint { year, value });
    }

    HistoricalVariable {
        variable: variable_id.to_string(),
        source,
        units,
        transformation,
        data,
    }
}

// ---------------------------------------------------------------------------
// Directory loader
// ---------------------------------------------------------------------------

/// Load all `.csv` files from `dir`, using the file stem as the variable id.
///
/// Returns an empty map if the directory does not exist or cannot be read.
/// Logs a warning for missing directories and info for each loaded variable.
pub fn load_historical_data(dir: &Path) -> HashMap<String, HistoricalVariable> {
    let mut map = HashMap::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(
                path = %dir.display(),
                error = %e,
                "Historical data directory not found or unreadable"
            );
            return map;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "Failed to read directory entry");
                continue;
            }
        };

        let path = entry.path();

        // Only process .csv files
        if path.extension().and_then(|ext| ext.to_str()) != Some("csv") {
            continue;
        }

        let variable_id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    file = %path.display(),
                    error = %e,
                    "Failed to read historical CSV file"
                );
                continue;
            }
        };

        let var = parse_historical_csv(&variable_id, &content);
        tracing::info!(
            variable = %var.variable,
            source = %var.source,
            points = var.data.len(),
            "Loaded historical data"
        );
        map.insert(variable_id, var);
    }

    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

// REQ: REQ-012
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const SAMPLE_CSV: &str = "\
# source: World Bank SP.POP.TOTL
# url: https://data.worldbank.org/indicator/SP.POP.TOTL
# units: persons
# transformation: none
# retrieved: 2026-03-04
year,value
1960,3021512598
1970,3680589045
1980,4437602892
";

    #[test]
    fn parse_extracts_metadata() {
        let result = parse_historical_csv("population", SAMPLE_CSV);
        assert_eq!(result.variable, "population");
        assert_eq!(result.source, "World Bank SP.POP.TOTL");
        assert_eq!(result.units, "persons");
        assert_eq!(result.transformation, "none");
    }

    #[test]
    fn parse_extracts_data_points() {
        let csv = "\
# source: Test
# units: index
# transformation: scale by 1e-3
year,value
2000,1.5e3
2010,2.0e3
2020,3.14159
";
        let result = parse_historical_csv("test_var", csv);
        assert_eq!(result.data.len(), 3);

        assert!((result.data[0].year - 2000.0).abs() < f64::EPSILON);
        assert!((result.data[0].value - 1500.0).abs() < f64::EPSILON);

        assert!((result.data[1].year - 2010.0).abs() < f64::EPSILON);
        assert!((result.data[1].value - 2000.0).abs() < f64::EPSILON);

        assert!((result.data[2].year - 2020.0).abs() < f64::EPSILON);
        assert!((result.data[2].value - 3.14159).abs() < 1e-10);
    }

    #[test]
    fn parse_handles_empty_content() {
        let result = parse_historical_csv("empty", "");
        assert!(result.data.is_empty());
        assert_eq!(result.variable, "empty");
        assert_eq!(result.source, "");
        assert_eq!(result.units, "");
        assert_eq!(result.transformation, "");
    }

    #[test]
    fn parse_skips_malformed_lines() {
        let csv = "\
# source: Test
year,value
1960,3021512598
bad_year,100
1970,not_a_number
1980
,5000
1990,4437602892
";
        let result = parse_historical_csv("test", csv);
        // Only 1960 and 1990 should parse successfully
        assert_eq!(result.data.len(), 2);
        assert!((result.data[0].year - 1960.0).abs() < f64::EPSILON);
        assert!((result.data[1].year - 1990.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_skips_nan_and_infinity() {
        let csv = "\
# source: Test
year,value
1960,100
NaN,200
1970,NaN
inf,300
1980,inf
-inf,400
1990,-inf
2000,500
";
        let result = parse_historical_csv("test", csv);
        // Only 1960 and 2000 should parse (all NaN/inf rows skipped)
        assert_eq!(result.data.len(), 2);
        assert!((result.data[0].year - 1960.0).abs() < f64::EPSILON);
        assert!((result.data[0].value - 100.0).abs() < f64::EPSILON);
        assert!((result.data[1].year - 2000.0).abs() < f64::EPSILON);
        assert!((result.data[1].value - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn load_returns_empty_for_missing_dir() {
        let dir = PathBuf::from("/tmp/nonexistent_historical_data_test_dir");
        let result = load_historical_data(&dir);
        assert!(result.is_empty());
    }

    #[test]
    fn load_reads_multiple_csv_files() {
        let dir = std::env::temp_dir().join(format!(
            "historical_test_multi_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Write two CSV files
        std::fs::write(
            dir.join("population.csv"),
            "# source: Test Pop\n# units: persons\n# transformation: none\nyear,value\n1960,3e9\n",
        ).unwrap();
        std::fs::write(
            dir.join("resources.csv"),
            "# source: Test Res\n# units: fraction\n# transformation: cumulative\nyear,value\n1900,0.99\n",
        ).unwrap();
        // Write a non-CSV file that should be ignored
        std::fs::write(dir.join("notes.txt"), "ignore me").unwrap();

        let result = load_historical_data(&dir);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key("population"));
        assert!(result.contains_key("resources"));
        assert_eq!(result["population"].data.len(), 1);
        assert_eq!(result["population"].source, "Test Pop");
        assert_eq!(result["resources"].data.len(), 1);
        assert_eq!(result["resources"].source, "Test Res");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_handles_windows_line_endings() {
        let csv = "# source: Test\r\n# units: index\r\nyear,value\r\n2000,42\r\n2010,84\r\n";
        let result = parse_historical_csv("win", csv);
        assert_eq!(result.source, "Test");
        assert_eq!(result.units, "index");
        assert_eq!(result.data.len(), 2);
        assert!((result.data[0].value - 42.0).abs() < f64::EPSILON);
    }
}
