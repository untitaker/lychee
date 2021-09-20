use std::path::Path;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// `FileType` defines which file types lychee can handle
pub enum FileType {
    /// File in HTML format
    Html,
    /// File in Markdown format
    Markdown,
    /// Generic text file without syntax-specific parsing
    Plaintext,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Plaintext
    }
}

impl<P: AsRef<Path>> From<P> for FileType {
    /// Detect if the given path points to a Markdown, HTML, or plaintext file.
    fn from(p: P) -> FileType {
        let path = p.as_ref();
        // Assume HTML in case of no extension.
        // Note: this is only reasonable for URLs; not paths on disk.
        // For example, `README` without an extension is more likely to be a plaintext file.
        // A better solution would be to also implement `From<Url> for FileType`.
        // Unfortunately that's not possible without refactoring, as
        // `AsRef<Path>` could be implemented for `Url` in the future, which is why
        // `From<Url> for FileType` is not allowed.
        // As a workaround, we check if we got a known web-protocol
        let is_url = path.starts_with("http");

        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some("md" | "markdown") => FileType::Markdown,
            Some("htm" | "html") => FileType::Html,
            None if is_url => FileType::Html,
            _ => FileType::Plaintext,
        }
    }
}