/// Output format for CLI responses.
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Human,
    Json,
}

/// Format a successful result for output.
pub fn format_success(format: &OutputFormat, data: &serde_json::Value) -> String {
    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_owned())
        }
        OutputFormat::Human => format_value_human(data),
    }
}

/// Format an error message for output.
pub fn format_error(format: &OutputFormat, message: &str) -> String {
    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&serde_json::json!({ "error": message }))
                .unwrap_or_else(|_| format!(r#"{{"error":"{message}"}}"#))
        }
        OutputFormat::Human => format!("Error: {message}"),
    }
}

/// Format a list of items for output.
pub fn format_list(format: &OutputFormat, items: &[serde_json::Value]) -> String {
    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(items).unwrap_or_else(|_| "[]".to_owned())
        }
        OutputFormat::Human => {
            if items.is_empty() {
                return "No items found.".to_owned();
            }
            items
                .iter()
                .enumerate()
                .map(|(i, item)| format!("{}. {}", i + 1, format_value_human(item)))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

/// Print a result to stdout (success) or stderr (error).
pub fn print_result(format: &OutputFormat, result: Result<serde_json::Value, anyhow::Error>) {
    match result {
        Ok(data) => println!("{}", format_success(format, &data)),
        Err(err) => eprintln!("{}", format_error(format, &err.to_string())),
    }
}

/// Render a JSON value in a human-readable key: value format.
fn format_value_human(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| format!("{k}: {}", human_scalar(v)))
            .collect::<Vec<_>>()
            .join("\n"),
        serde_json::Value::Array(arr) => arr
            .iter()
            .map(format_value_human)
            .collect::<Vec<_>>()
            .join("\n---\n"),
        other => human_scalar(other),
    }
}

/// Convert a scalar JSON value to a compact human-readable string.
fn human_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "(none)".to_owned(),
        other => other.to_string(),
    }
}
