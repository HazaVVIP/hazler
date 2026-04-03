//! Context extraction for secret findings
//!
//! This module provides utilities to extract meaningful context
//! around detected secrets to help reduce false positives and
//! provide actionable information to the user.

/// Extract context lines around a match within a multi-line string.
///
/// Returns up to `num_lines` lines before and after the matching line,
/// along with the line number of the match.
pub fn extract_context_lines<'a>(
    lines: &[&'a str],
    line_idx: usize,
    num_lines: usize,
) -> Vec<(usize, &'a str)> {
    let start = line_idx.saturating_sub(num_lines);
    let end = (line_idx + num_lines + 1).min(lines.len());
    (start..end).map(|i| (i + 1, lines[i])).collect()
}

/// Extract inline context around a byte offset within a single line.
///
/// Returns up to `window` characters before and after the match,
/// trimming to line boundaries.
pub fn extract_inline_context(
    line: &str,
    match_start: usize,
    match_end: usize,
    window: usize,
) -> String {
    let ctx_start = match_start.saturating_sub(window);
    let ctx_end = (match_end + window).min(line.len());
    line[ctx_start..ctx_end].to_string()
}

/// Determine whether a string looks like it is in a test/example context
/// based on common keywords in the surrounding text, which helps reduce
/// false positives.
pub fn is_likely_test_context(context: &str) -> bool {
    let lower = context.to_lowercase();
    lower.contains("test")
        || lower.contains("example")
        || lower.contains("sample")
        || lower.contains("dummy")
        || lower.contains("fake")
        || lower.contains("mock")
        || lower.contains("placeholder")
        || lower.contains("your_")
        || lower.contains("_here")
        || lower.contains("changeme")
        || lower.contains("xxx")
        || lower.contains("todo")
}

/// Determine whether a string looks like it is a variable name or
/// template placeholder rather than a real secret value.
pub fn is_likely_placeholder(value: &str) -> bool {
    // Common placeholder patterns
    let lower = value.to_lowercase();
    lower.starts_with("your")
        || lower.starts_with("my_")
        || lower.ends_with("_here")
        || lower.ends_with("_key")
        || lower == "secret"
        || lower == "password"
        || lower == "changeme"
        || lower == "replace_me"
        || lower.starts_with("xxx")
        || lower.ends_with("xxx")
        // Repeating characters (e.g. "aaaaaaaaaa") are unlikely to be real secrets.
        // Obtain the first character once and compare the rest against it in a
        // single pass instead of calling `value.chars().next()` inside the closure.
        || (value.len() > 4 && {
            let mut chars = value.chars();
            chars
                .next()
                .is_some_and(|first| chars.all(|c| c == first))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_context_lines() {
        let text = "line1\nline2\nline3\nline4\nline5";
        let lines: Vec<&str> = text.lines().collect();
        let ctx = extract_context_lines(&lines, 2, 1);
        // line_idx=2 is "line3"; with 1 line before/after => lines 2,3,4 (1-indexed)
        assert_eq!(ctx.len(), 3);
        assert_eq!(ctx[0], (2, "line2"));
        assert_eq!(ctx[1], (3, "line3"));
        assert_eq!(ctx[2], (4, "line4"));
    }

    #[test]
    fn test_extract_inline_context() {
        let line = "prefix__SECRET__suffix";
        // "SECRET" starts at byte 8; window=3 gives ctx_start=5, ctx_end=17
        let ctx = extract_inline_context(line, 8, 14, 3);
        assert_eq!(ctx, "x__SECRET__s");
    }

    #[test]
    fn test_is_likely_test_context() {
        assert!(is_likely_test_context("const testApiKey = 'abc123';"));
        assert!(is_likely_test_context("# example usage"));
        assert!(!is_likely_test_context(
            "const apiKey = 'Xk7mP9qR8vB2nL4s';"
        ));
    }

    #[test]
    fn test_is_likely_placeholder() {
        assert!(is_likely_placeholder("your_api_key_here"));
        assert!(is_likely_placeholder("changeme"));
        assert!(is_likely_placeholder("aaaaaaaaaaaaaaaa"));
        assert!(!is_likely_placeholder("Xk7mP9qR8vB2nL4s"));
    }
}
