use comrak::{markdown_to_html, Options};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use crate::error::CoreError;

/// Convert Markdown to HTML using comrak with GFM extensions enabled.
pub fn render_markdown(input: &str) -> Result<String, CoreError> {
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.tagfilter = true;

    Ok(markdown_to_html(input, &options))
}

/// Syntax highlight code using syntect, returning an HTML string.
///
/// Falls back to plain text if the extension is not recognized.
/// The returned HTML is wrapped in `<pre><code>` if not already wrapped.
pub fn highlight_code(code: &str, extension: &str) -> Result<String, CoreError> {
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ss
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let theme = ts
        .themes
        .get("InspiredGitHub")
        .or_else(|| ts.themes.get("base16-ocean.dark"))
        .ok_or_else(|| CoreError::RenderError("No suitable theme found".to_string()))?;

    let html = highlighted_html_for_string(code, &ss, syntax, theme)
        .map_err(|e| CoreError::RenderError(e.to_string()))?;

    if html.trim_start().starts_with("<pre") {
        Ok(html)
    } else {
        Ok(format!("<pre><code>{html}</code></pre>"))
    }
}

/// Wrap plain text in HTML-safe `<pre><code>` tags.
///
/// HTML special characters are escaped before wrapping.
pub fn wrap_plain_text(text: &str) -> String {
    let escaped = html_escape(text);
    format!("<pre><code>{escaped}</code></pre>")
}

fn html_escape(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown() {
        let result = render_markdown("# Hello").unwrap();
        assert!(result.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_render_markdown_gfm_table() {
        let input = "| Col1 | Col2 |\n|------|------|\n| A    | B    |";
        let result = render_markdown(input).unwrap();
        assert!(result.contains("<table>"));
        assert!(result.contains("<th>"));
    }

    #[test]
    fn test_highlight_code() {
        let code = r#"{"key": "value"}"#;
        let result = highlight_code(code, "json").unwrap();
        // syntect returns highlighted HTML with spans or pre tags
        assert!(result.contains("<pre") || result.contains("<span"));
    }

    #[test]
    fn test_wrap_plain_text() {
        let result = wrap_plain_text("Hello <world> & \"things\"");
        assert!(result.starts_with("<pre><code>"));
        assert!(result.ends_with("</code></pre>"));
        assert!(result.contains("&lt;world&gt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("&quot;"));
    }

    #[test]
    fn test_html_escape() {
        let result = html_escape("a & b < c > d \"e\" 'f'");
        assert!(result.contains("&amp;"));
        assert!(result.contains("&lt;"));
        assert!(result.contains("&gt;"));
        assert!(result.contains("&quot;"));
        assert!(result.contains("&#x27;"));
        assert!(!result.contains('&') || result.contains("&amp;"));
    }
}
