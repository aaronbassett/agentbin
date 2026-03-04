#![deny(unsafe_code)]

use std::path::Path;

use agentbin_core::{highlight_code, render_markdown, FileType};

/// The result of rendering document content for display.
pub enum RenderResult {
    /// HTML passthrough — serve uploaded HTML directly (badge injection handled by caller).
    HtmlPassthrough(String),
    /// Rendered content (markdown or syntax-highlighted) to wrap in `rendered.html`.
    Rendered { title: String, content: String },
    /// Plain text to wrap in `plain.html`.
    PlainText { title: String, content: String },
}

/// Render document content based on its file type.
///
/// Dispatches to the appropriate rendering strategy:
/// - [`FileType::Html`] — returned as-is for passthrough
/// - [`FileType::Markdown`] — converted to HTML via comrak
/// - Structured text types — syntax-highlighted via syntect
/// - [`FileType::PlainText`] — returned as-is for plain display
pub fn render_content(
    content: &str,
    file_type: &FileType,
    filename: &str,
) -> Result<RenderResult, anyhow::Error> {
    let title = filename.to_string();

    match file_type {
        FileType::Html => Ok(RenderResult::HtmlPassthrough(content.to_string())),

        FileType::Markdown => {
            let html = render_markdown(content)?;
            Ok(RenderResult::Rendered {
                title,
                content: html,
            })
        }

        FileType::Json
        | FileType::Jsonc
        | FileType::Toml
        | FileType::Yaml
        | FileType::Xml
        | FileType::Rst => {
            let ext = Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let html = highlight_code(content, ext)?;
            Ok(RenderResult::Rendered {
                title,
                content: html,
            })
        }

        FileType::PlainText => Ok(RenderResult::PlainText {
            title,
            content: content.to_string(),
        }),
    }
}
