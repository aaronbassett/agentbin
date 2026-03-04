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
