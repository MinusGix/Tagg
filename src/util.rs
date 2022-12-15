use std::path::{Path, PathBuf};

pub fn expand_path(path: impl AsRef<str>) -> PathBuf {
    // TODO: Do i need to specialize this to just linux/unix/bsd?
    let path = shellexpand::tilde(path.as_ref());
    PathBuf::from(path.as_ref())
}

pub(crate) fn extract_pdf_title(path: &Path, page_number: usize) -> Option<String> {
    let res = std::process::Command::new("pdftitle")
        // Max2 managed to extract the most in my experience
        .arg("-a")
        .arg("max2")
        .arg("-p")
        .arg(path)
        .arg("--page-number")
        .arg(page_number.to_string())
        .output()
        .ok()?;
    if res.status.success() {
        Some(String::from_utf8_lossy(&res.stdout).trim().to_string())
    } else {
        None
    }
}

pub(crate) fn is_pdf_title_bad(title: Option<&str>) -> bool {
    if let Some(title) = title {
        title == "This page intentionally left blank"
            || title == "This page intentionally left blank."
            // Most titles below 4 characters are garbage
            || title.len() <= 4
            // If it is greater than 120 characters than it is probably garbage.
            || title.len() >= 120
    } else {
        // None is bad
        true
    }
}

pub(crate) fn fixup_pdf_title(title: String) -> String {
    title
        // Pdfs sometimes use this merged character rather than the separate letters
        .replace('ﬁ', "fi")
        .replace('ﬂ', "fl")
        // Garbage?
        .replace('■', "")
        // Bad translation of diacritic
        .replace("¨o", "ö")
        .trim()
        .to_string()
}

pub(crate) fn extract_title(path: &Path, extension: &str) -> Option<String> {
    if extension == "pdf" {
        // TODO: let this be customizable
        // We try extracting the pdf title from the first three pages.
        // Various documents have an image on their first page, rather than an actual title
        // but then have the title on one of the next pages,
        // but some also have things like 'this page left blank' right before the title page
        let mut title = extract_pdf_title(path, 1);
        if is_pdf_title_bad(title.as_deref()) {
            title = extract_pdf_title(path, 2);

            if is_pdf_title_bad(title.as_deref()) {
                title = extract_pdf_title(path, 3);

                if is_pdf_title_bad(title.as_deref()) {
                    title = None;
                }
            }
        }

        title.map(fixup_pdf_title)
    } else {
        None
    }
}
