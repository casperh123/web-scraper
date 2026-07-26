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
