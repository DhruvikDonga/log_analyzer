use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub date_formats: Vec<LogMatcher>,
    pub error_indicators: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogMatcher {
    pub format: String,
}

pub fn parsed_with_dynamic_format(
    line: &str,
    config: &LogConfig,
) -> Option<(DateTime<Utc>, String)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let chars = [' ', '"', '\''];

    for matcher in &config.date_formats {
        if let Ok(ts) = DateTime::parse_from_str(line, &matcher.format) {
            return Some((ts.with_timezone(&Utc), line.to_string()));
        }

        if let (Some(sp), Some(ep)) = (line.find('['), line.find(']')) {
            let inner = &line[sp..ep + 1];
            if let Ok(ts) = DateTime::parse_from_str(inner, &matcher.format) {
                return Some((ts.with_timezone(&Utc), line.to_string()));
            }
        }

        for word in line.split(|c| chars.contains(&c)) {
            let cleaned = word.trim_matches(|c| c == ',' || c == ':' || c == '{' || c == '}');

            if cleaned.is_empty() {
                continue;
            }
            if let Ok(ts) = DateTime::parse_from_str(cleaned, &matcher.format) {
                return Some((ts.with_timezone(&Utc), line.to_string()));
            }
        }
    }
    None
}
