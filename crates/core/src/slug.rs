/// Derive a URL-friendly slug from a filename.
///
/// Strips the file extension, lowercases, replaces non-alphanumeric characters
/// with hyphens, collapses runs, and truncates to 48 characters. Returns `None`
/// for empty, numeric-only, or single-character results.
pub fn slugify_filename(filename: &str) -> Option<String> {
    // Strip extension — if the only dot is at position 0 (e.g. ".md"), treat
    // the entire input as extension-only (no usable stem).
    let stem = match filename.rfind('.') {
        Some(0) => return None,
        Some(pos) => &filename[..pos],
        None => filename,
    };

    let slug: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Collapse runs of hyphens and trim leading/trailing hyphens
    let mut collapsed = String::with_capacity(slug.len());
    let mut prev_hyphen = true; // treat start as hyphen to trim leading
    for ch in slug.chars() {
        if ch == '-' {
            if !prev_hyphen {
                collapsed.push('-');
            }
            prev_hyphen = true;
        } else {
            collapsed.push(ch);
            prev_hyphen = false;
        }
    }
    // Trim trailing hyphen
    if collapsed.ends_with('-') {
        collapsed.pop();
    }

    // Truncate to 48 chars (on a hyphen boundary if possible)
    if collapsed.len() > 48 {
        collapsed.truncate(48);
        if let Some(last_hyphen) = collapsed.rfind('-') {
            collapsed.truncate(last_hyphen);
        }
    }

    // Reject empty, single-char, or purely numeric slugs
    if collapsed.len() <= 1 || collapsed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    Some(collapsed)
}

/// Extract the 10-character UID prefix from a path segment that may contain a slug suffix.
///
/// Given `"1vjmeRjNdi-stdlib-fix-plan"`, returns `"1vjmeRjNdi"`.
/// Short inputs pass through unchanged to fail at `validate_uid`.
pub fn extract_uid(path_segment: &str) -> &str {
    if path_segment.len() >= 10 {
        &path_segment[..10]
    } else {
        path_segment
    }
}

/// Combine a UID with an optional slug to form a URL path segment.
///
/// Returns `"{uid}-{slug}"` when a slug is present, or just `"{uid}"` otherwise.
pub fn uid_with_slug(uid: &str, slug: Option<&str>) -> String {
    match slug {
        Some(s) if !s.is_empty() => format!("{uid}-{s}"),
        _ => uid.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic_filename() {
        assert_eq!(
            slugify_filename("stdlib-fix-plan.md"),
            Some("stdlib-fix-plan".to_string())
        );
    }

    #[test]
    fn slugify_strips_extension() {
        assert_eq!(
            slugify_filename("My Report.html"),
            Some("my-report".to_string())
        );
    }

    #[test]
    fn slugify_collapses_special_chars() {
        assert_eq!(
            slugify_filename("foo___bar--baz.txt"),
            Some("foo-bar-baz".to_string())
        );
    }

    #[test]
    fn slugify_trims_leading_trailing_hyphens() {
        assert_eq!(slugify_filename("--hello--.md"), Some("hello".to_string()));
    }

    #[test]
    fn slugify_truncates_long_names() {
        let long_name = "a".repeat(60) + ".md";
        let result = slugify_filename(&long_name).unwrap();
        assert!(result.len() <= 48);
    }

    #[test]
    fn slugify_truncates_on_hyphen_boundary() {
        // 45 chars of 'a', then '-bbb' = 49 chars total, should truncate at the hyphen
        let name = format!("{}-bbb.md", "a".repeat(45));
        let result = slugify_filename(&name).unwrap();
        assert!(result.len() <= 48);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn slugify_returns_none_for_empty() {
        assert_eq!(slugify_filename(".md"), None);
    }

    #[test]
    fn slugify_returns_none_for_single_char() {
        assert_eq!(slugify_filename("a.md"), None);
    }

    #[test]
    fn slugify_returns_none_for_numeric_only() {
        assert_eq!(slugify_filename("12345.txt"), None);
    }

    #[test]
    fn slugify_returns_none_for_no_extension_single_char() {
        assert_eq!(slugify_filename("x"), None);
    }

    #[test]
    fn slugify_no_extension() {
        assert_eq!(slugify_filename("readme"), Some("readme".to_string()));
    }

    #[test]
    fn extract_uid_with_slug() {
        assert_eq!(extract_uid("1vjmeRjNdi-stdlib-fix-plan"), "1vjmeRjNdi");
    }

    #[test]
    fn extract_uid_plain() {
        assert_eq!(extract_uid("1vjmeRjNdi"), "1vjmeRjNdi");
    }

    #[test]
    fn extract_uid_short_input() {
        assert_eq!(extract_uid("abc"), "abc");
    }

    #[test]
    fn uid_with_slug_some() {
        assert_eq!(
            uid_with_slug("1vjmeRjNdi", Some("stdlib-fix-plan")),
            "1vjmeRjNdi-stdlib-fix-plan"
        );
    }

    #[test]
    fn uid_with_slug_none() {
        assert_eq!(uid_with_slug("1vjmeRjNdi", None), "1vjmeRjNdi");
    }

    #[test]
    fn uid_with_slug_empty() {
        assert_eq!(uid_with_slug("1vjmeRjNdi", Some("")), "1vjmeRjNdi");
    }
}
