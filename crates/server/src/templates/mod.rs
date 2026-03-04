#![deny(unsafe_code)]

use askama::Template;

/// Askama template for rendering HTTP error responses.
#[allow(dead_code)] // constructed by error handlers once implemented
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub status: u16,
    pub message: String,
}

/// Askama template for rendered content (markdown, syntax highlighted).
#[allow(dead_code)] // constructed by view route once implemented
#[derive(Template)]
#[template(path = "rendered.html")]
pub struct RenderedTemplate {
    pub title: String,
    pub content: String,
    pub badge_html: String,
    pub version_banner_html: Option<String>,
}

/// Askama template for plain text content.
#[allow(dead_code)] // constructed by view route once implemented
#[derive(Template)]
#[template(path = "plain.html")]
pub struct PlainTemplate {
    pub title: String,
    pub content: String,
    pub badge_html: String,
    pub version_banner_html: Option<String>,
}
