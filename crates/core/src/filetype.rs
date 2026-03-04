use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    Html,
    Markdown,
    Json,
    Jsonc,
    Toml,
    Yaml,
    Xml,
    Rst,
    PlainText,
}

impl FileType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "html" | "htm" => Self::Html,
            "md" | "markdown" => Self::Markdown,
            "json" => Self::Json,
            "jsonc" => Self::Jsonc,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "xml" => Self::Xml,
            "rst" => Self::Rst,
            _ => Self::PlainText,
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        Self::from_extension(ext)
    }

    pub fn content_type(&self) -> &'static str {
        match self {
            Self::Html => "text/html",
            Self::Markdown => "text/markdown",
            Self::Json | Self::Jsonc => "application/json",
            Self::Toml => "application/toml",
            Self::Yaml => "text/yaml",
            Self::Xml => "application/xml",
            Self::Rst => "text/x-rst",
            Self::PlainText => "text/plain",
        }
    }

    pub fn is_syntax_highlighted(&self) -> bool {
        matches!(
            self,
            Self::Json | Self::Jsonc | Self::Toml | Self::Yaml | Self::Xml | Self::Rst
        )
    }

    pub fn is_rendered(&self) -> bool {
        matches!(self, Self::Markdown)
    }

    pub fn is_passthrough(&self) -> bool {
        matches!(self, Self::Html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_extension_known_types() {
        assert_eq!(FileType::from_extension("html"), FileType::Html);
        assert_eq!(FileType::from_extension("htm"), FileType::Html);
        assert_eq!(FileType::from_extension("md"), FileType::Markdown);
        assert_eq!(FileType::from_extension("markdown"), FileType::Markdown);
        assert_eq!(FileType::from_extension("json"), FileType::Json);
        assert_eq!(FileType::from_extension("jsonc"), FileType::Jsonc);
        assert_eq!(FileType::from_extension("toml"), FileType::Toml);
        assert_eq!(FileType::from_extension("yaml"), FileType::Yaml);
        assert_eq!(FileType::from_extension("yml"), FileType::Yaml);
        assert_eq!(FileType::from_extension("xml"), FileType::Xml);
        assert_eq!(FileType::from_extension("rst"), FileType::Rst);
        assert_eq!(FileType::from_extension("txt"), FileType::PlainText);
        assert_eq!(FileType::from_extension("rs"), FileType::PlainText);
    }

    #[test]
    fn from_extension_case_insensitive() {
        assert_eq!(FileType::from_extension("HTML"), FileType::Html);
        assert_eq!(FileType::from_extension("MD"), FileType::Markdown);
    }

    #[test]
    fn content_type_values() {
        assert_eq!(FileType::Html.content_type(), "text/html");
        assert_eq!(FileType::Markdown.content_type(), "text/markdown");
        assert_eq!(FileType::Json.content_type(), "application/json");
        assert_eq!(FileType::Jsonc.content_type(), "application/json");
        assert_eq!(FileType::Toml.content_type(), "application/toml");
        assert_eq!(FileType::Yaml.content_type(), "text/yaml");
        assert_eq!(FileType::Xml.content_type(), "application/xml");
        assert_eq!(FileType::Rst.content_type(), "text/x-rst");
        assert_eq!(FileType::PlainText.content_type(), "text/plain");
    }
}
