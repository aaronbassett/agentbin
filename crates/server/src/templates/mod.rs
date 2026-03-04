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

/// A single file entry displayed on a collection overview page.
pub struct CollectionMemberView {
    pub uid: String,
    pub filename: String,
    pub latest_version: u32,
    /// Human-readable timestamp (e.g. "2024-01-15 09:30 UTC").
    pub added_at: String,
    /// Full URL to the latest version of this upload.
    pub url: String,
    /// Position along the timeline, 0–100.
    pub timeline_pct: u8,
}

/// Askama template for a collection overview page.
#[derive(Template)]
#[template(path = "collection.html")]
pub struct CollectionTemplate {
    pub name: String,
    pub members: Vec<CollectionMemberView>,
    /// Formatted timestamp of the oldest member (used as left timeline label).
    pub oldest_date: String,
    /// Formatted timestamp of the newest member (used as right timeline label).
    pub newest_date: String,
}
