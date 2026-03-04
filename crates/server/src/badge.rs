use agentbin_core::metadata::VersionMeta;
use serde_json::{json, Value};

/// Generate the badge HTML snippet to inject into pages.
///
/// Returns a `<script>` tag pointing to `/_static/badge.js` followed by an
/// `<agentbin-badge>` custom element whose `data-meta` attribute carries a
/// JSON object describing the upload.
pub fn generate_badge_html(meta: &VersionMeta, base_url: &str, uid: &str) -> String {
    let url = format!("{base_url}/{uid}");
    let raw_url = format!("{base_url}/{uid}/raw");

    let mut obj = json!({
        "uid": uid,
        "version": meta.version,
        "filename": meta.filename,
        "uploaded_by": meta.uploaded_by,
        "uploaded_at": meta.uploaded_at.format("%Y-%m-%d %H:%M UTC").to_string(),
        "content_type": meta.content_type,
        "url": url,
        "raw_url": raw_url,
    });

    let m = &meta.metadata;

    if let Some(title) = &m.title {
        obj["title"] = Value::String(title.clone());
    }
    if let Some(description) = &m.description {
        obj["description"] = Value::String(description.clone());
    }
    if !m.tags.is_empty() {
        obj["tags"] = Value::Array(m.tags.iter().map(|t| Value::String(t.clone())).collect());
    }
    if let Some(agent) = &m.agent {
        let mut agent_obj = serde_json::Map::new();
        if let Some(model) = &agent.model {
            agent_obj.insert("model".to_owned(), Value::String(model.clone()));
        }
        if let Some(provider) = &agent.provider {
            agent_obj.insert("provider".to_owned(), Value::String(provider.clone()));
        }
        if let Some(tool) = &agent.tool {
            agent_obj.insert("tool".to_owned(), Value::String(tool.clone()));
        }
        obj["agent"] = Value::Object(agent_obj);
    }
    if let Some(trigger) = &m.trigger {
        obj["trigger"] = Value::String(trigger.clone());
    }
    if !m.custom.is_empty() {
        let custom_obj: serde_json::Map<String, Value> = m
            .custom
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        obj["custom"] = Value::Object(custom_obj);
    }

    // Serialize and HTML-escape for use inside a single-quoted HTML attribute.
    let json_str = obj.to_string();
    let escaped = html_escape_attr(&json_str);

    format!(
        "<script src=\"/_static/badge.js\"></script>\n\
         <agentbin-badge data-meta='{escaped}'></agentbin-badge>"
    )
}

/// Inject `badge_html` into `html` just before `</body>` (or appended if absent).
pub fn inject_badge_into_html(html: &str, badge_html: &str) -> String {
    if let Some(pos) = html.rfind("</body>") {
        let mut result = String::with_capacity(html.len() + badge_html.len() + 1);
        result.push_str(&html[..pos]);
        result.push('\n');
        result.push_str(badge_html);
        result.push('\n');
        result.push_str(&html[pos..]);
        result
    } else {
        let mut result = String::with_capacity(html.len() + badge_html.len() + 1);
        result.push_str(html);
        result.push('\n');
        result.push_str(badge_html);
        result
    }
}

/// HTML-escape a string for safe embedding inside a single-quoted attribute value.
fn html_escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' => out.push_str("&#39;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentbin_core::metadata::{AgentInfo, Metadata, VersionMeta};
    use chrono::Utc;

    fn make_meta() -> VersionMeta {
        VersionMeta {
            version: 1,
            filename: "report.md".to_owned(),
            content_type: "text/html".to_owned(),
            size_bytes: 1024,
            uploaded_at: Utc::now(),
            uploaded_by: "alice".to_owned(),
            expires_at: None,
            metadata: Metadata {
                title: Some("My Report".to_owned()),
                description: Some("A test report".to_owned()),
                tags: vec!["rust".to_owned(), "test".to_owned()],
                agent: Some(AgentInfo {
                    model: Some("claude-sonnet-4-6".to_owned()),
                    provider: Some("anthropic".to_owned()),
                    tool: Some("agentbin".to_owned()),
                }),
                trigger: Some("manual".to_owned()),
                custom: [("env".to_owned(), "prod".to_owned())].into(),
            },
        }
    }

    #[test]
    fn badge_html_contains_script_and_element() {
        let html = generate_badge_html(&make_meta(), "https://example.com", "abc123");
        assert!(html.contains("<script src=\"/_static/badge.js\">"));
        assert!(html.contains("<agentbin-badge data-meta='"));
        assert!(html.contains("abc123"));
        assert!(html.contains("My Report"));
    }

    #[test]
    fn inject_before_body_close() {
        let page = "<html><body><p>hi</p></body></html>";
        let badge = "<script></script>";
        let result = inject_badge_into_html(page, badge);
        assert!(result.find("<script>").unwrap() < result.find("</body>").unwrap());
    }

    #[test]
    fn inject_appends_when_no_body_tag() {
        let page = "<p>hi</p>";
        let badge = "BADGE";
        let result = inject_badge_into_html(page, badge);
        assert!(result.ends_with("BADGE"));
    }

    #[test]
    fn html_escape_handles_special_chars() {
        let s = html_escape_attr("a & b < c > d' e\"");
        assert_eq!(s, "a &amp; b &lt; c &gt; d&#39; e&quot;");
    }
}
