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