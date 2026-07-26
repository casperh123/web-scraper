use reqwest::Url;


pub(super) fn is_image_or_file(url: &Url) -> bool {
    const FORBIDDEN_EXTENSIONS: [&str; 24] = [
        "jpg", "jpeg", "png", "gif", "webp", "svg", "ico", "bmp",
        "pdf", "zip", "gz", "tar", "rar",
        "mp4", "mp3", "wav", "avi", "mov",
        "css", "js", "json", "xml", "rss", "atom",
    ];

    url.path()
        .rsplit('.')
        .next()
        .map(|ext| FORBIDDEN_EXTENSIONS.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_forbidden_extensions_case_insensitively() {
        let cases = [
            ("https://example.dk/a.PDF", true),
            ("https://example.dk/a.jpg", true),
            ("https://example.dk/a.html", false),
            ("https://example.dk/noext", false),
            ("https://example.dk/", false),
        ];

        for (input, expected) in cases {
            let url = Url::parse(input).unwrap();

            assert_eq!(
                is_image_or_file(&url), expected,
                "failed for input: {input}"
            );
        }
    }
}
