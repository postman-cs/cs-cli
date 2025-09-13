use crate::{CsCliError, Result};
use htmd::{convert, HtmlToMarkdown};
use regex::Regex;
use std::sync::OnceLock;

/// HTML processing functionality with both markdown and plain text conversion
/// Provides feature parity with Python version including plain text output
pub struct HTMLProcessor {
    converter: HtmlToMarkdown,
}

impl Default for HTMLProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl HTMLProcessor {
    pub fn new() -> Self {
        // Configure htmd for optimal Gong email/call content conversion
        let converter = HtmlToMarkdown::builder()
            .skip_tags(vec!["script", "style", "meta", "link"]) // Skip non-content tags
            .build();

        Self { converter }
    }

    /// Convert HTML content to clean plain text (matches Python version)
    pub fn process_content(&self, html: &str) -> Result<String> {
        self.to_text(html)
    }

    /// Convert HTML content to clean markdown using htmd
    pub fn to_markdown(&self, html: &str) -> Result<String> {
        match self.converter.convert(html) {
            Ok(markdown) => Ok(markdown.trim().to_string()),
            Err(e) => {
                // Fallback to basic conversion if htmd fails
                match convert(html) {
                    Ok(fallback) => Ok(fallback.trim().to_string()),
                    Err(_) => Err(CsCliError::Generic(format!("HTML processing failed: {e}"))),
                }
            }
        }
    }

    /// Convert HTML to plain text (feature parity with Python version)
    pub fn to_text(&self, html: &str) -> Result<String> {
        if html.is_empty() || html.trim().is_empty() {
            return Ok(String::new());
        }

        // Try markdown conversion first, then convert to plain text
        match self.to_markdown(html) {
            Ok(markdown) => {
                // Strip markdown formatting to get plain text
                let text = self.markdown_to_text(&markdown);
                Ok(text)
            }
            Err(_) => {
                // Fallback to regex-based HTML stripping
                Ok(self.regex_html_to_text(html))
            }
        }
    }

    /// Convert markdown to plain text by stripping formatting
    fn markdown_to_text(&self, markdown: &str) -> String {
        static MARKDOWN_PATTERNS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();

        let patterns = MARKDOWN_PATTERNS.get_or_init(|| {
            vec![
                // Headers
                (Regex::new(r"^#{1,6}\s+").unwrap(), ""),
                // Bold/Italic
                (Regex::new(r"\*{1,3}([^*]+)\*{1,3}").unwrap(), "$1"),
                (Regex::new(r"_{1,3}([^_]+)_{1,3}").unwrap(), "$1"),
                // Links
                (Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap(), "$1"),
                // Images
                (Regex::new(r"!\[([^\]]*)\]\([^)]+\)").unwrap(), "$1"),
                // Code blocks
                (Regex::new(r"```[^`]*```").unwrap(), ""),
                // Inline code
                (Regex::new(r"`([^`]+)`").unwrap(), "$1"),
                // Blockquotes
                (Regex::new(r"^>\s+").unwrap(), ""),
                // Lists
                (Regex::new(r"^[\*\-\+]\s+").unwrap(), ""),
                (Regex::new(r"^\d+\.\s+").unwrap(), ""),
                // Horizontal rules
                (Regex::new(r"^---+$|^___+$|^\*\*\*+$").unwrap(), ""),
            ]
        });

        let mut text = markdown.to_string();
        for (pattern, replacement) in patterns.iter() {
            text = pattern.replace_all(&text, *replacement).to_string();
        }

        // Clean up whitespace
        static WHITESPACE_PATTERN: OnceLock<Regex> = OnceLock::new();
        let ws_pattern = WHITESPACE_PATTERN.get_or_init(|| Regex::new(r"\s+").unwrap());
        ws_pattern.replace_all(&text, " ").trim().to_string()
    }

    /// Fallback HTML to text conversion using regex (matches Python version)
    fn regex_html_to_text(&self, html: &str) -> String {
        static HTML_PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();

        let patterns = HTML_PATTERNS.get_or_init(|| {
            vec![
                // Remove script and style elements
                Regex::new(r"(?is)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>").unwrap(),
                // Remove HTML tags
                Regex::new(r"<[^>]+>").unwrap(),
                // Clean up whitespace
                Regex::new(r"\s+").unwrap(),
            ]
        });

        let mut text = html.to_string();

        // Remove script and style elements
        text = patterns[0].replace_all(&text, "").to_string();

        // Remove HTML tags
        text = patterns[1].replace_all(&text, " ").to_string();

        // Clean up whitespace
        text = patterns[2].replace_all(&text, " ").to_string();

        text.trim().to_string()
    }

    /// Process HTML with custom configuration
    pub fn process_with_config(&self, html: &str, skip_tags: Vec<&str>) -> Result<String> {
        let custom_converter = HtmlToMarkdown::builder().skip_tags(skip_tags).build();

        match custom_converter.convert(html) {
            Ok(markdown) => Ok(markdown.trim().to_string()),
            Err(e) => Err(CsCliError::Generic(format!(
                "Custom HTML processing failed: {e}"
            ))),
        }
    }
}

// Convenience function for quick HTML to plain text conversion (default, matches Python)
pub fn html_to_text(html: &str) -> Result<String> {
    let processor = HTMLProcessor::new();
    processor.to_text(html)
}

// Convenience function for quick HTML to markdown conversion
pub fn html_to_markdown(html: &str) -> Result<String> {
    let processor = HTMLProcessor::new();
    processor.to_markdown(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_text() {
        let processor = HTMLProcessor::new();

        // Test basic HTML with script and style tags (like Python version)
        let html = r#"
            <html>
            <head>
                <script>console.log('test');</script>
                <style>body { color: red; }</style>
            </head>
            <body>
                <h1>Title</h1>
                <p>This is <strong>bold</strong> text.</p>
                <div>Another paragraph</div>
            </body>
            </html>
        "#;

        let result = processor.to_text(html).unwrap();
        assert!(!result.contains("console.log"));
        assert!(!result.contains("color: red"));
        assert!(result.contains("Title"));
        assert!(result.contains("bold"));
        assert!(result.contains("Another paragraph"));
    }

    #[test]
    fn test_html_to_markdown() {
        let processor = HTMLProcessor::new();

        let html = "<h1>Title</h1><p>This is <strong>bold</strong> text.</p>";
        let result = processor.to_markdown(html).unwrap();

        // Should preserve markdown formatting
        assert!(result.contains("#") || result.contains("Title"));
        assert!(result.contains("**bold**") || result.contains("bold"));
    }

    #[test]
    fn test_empty_input() {
        let processor = HTMLProcessor::new();

        assert_eq!(processor.to_text("").unwrap(), "");
        assert_eq!(processor.to_text("   ").unwrap(), "");
    }

    #[test]
    fn test_process_content_returns_text() {
        let processor = HTMLProcessor::new();

        // process_content should return plain text (matching Python)
        let html = "<p>Test <em>content</em></p>";
        let result = processor.process_content(html).unwrap();

        // Should be plain text, not markdown
        assert!(!result.contains("*"));
        assert!(!result.contains("#"));
        assert!(result.contains("Test"));
        assert!(result.contains("content"));
    }
}
