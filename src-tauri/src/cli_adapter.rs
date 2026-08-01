//! CLI adapters: render a delivered path into the text string that gets
//! injected into the target terminal. This is the seam where per-CLI
//! rendering will live (v0.5) — e.g. a CLI that needs WSL-style paths, or one
//! that wants a bare path instead of Markdown.
//!
//! Step 10 ships only the two generic adapters. CLI detection (which adapter
//! fits the focused terminal) is deferred to v0.5; for now `adapter_for` picks
//! one from the configured output format, preserving existing behavior.

/// Render a delivered path into the string to inject.
pub trait CliAdapter: Send + Sync {
    /// Stable identifier for diagnostics / future detection scoring.
    fn name(&self) -> &'static str;
    /// Turn the delivered path into the final text fragment.
    fn render(&self, path: &str) -> String;
}

/// Rich rendering: Markdown by default, `<img>` for the `html` output format.
/// For AI CLIs that render rich content.
pub struct GenericMarkdownAdapter {
    html: bool,
}

impl CliAdapter for GenericMarkdownAdapter {
    fn name(&self) -> &'static str {
        "generic_markdown"
    }
    fn render(&self, path: &str) -> String {
        if self.html {
            format!("<img src=\"{}\" />", path)
        } else {
            format!("![image]({})", path)
        }
    }
}

/// Bare-path rendering: emit the location verbatim. For CLIs / pipelines that
/// want just the path (no Markdown wrapping).
pub struct GenericRawPathAdapter;

impl CliAdapter for GenericRawPathAdapter {
    fn name(&self) -> &'static str {
        "generic_raw_path"
    }
    fn render(&self, path: &str) -> String {
        path.to_string()
    }
}

/// Pick a generic adapter for the configured output format. (Base64 short-
/// circuits earlier in the pipeline and never reaches here.) Future CLI
/// detection will replace this with a scored selection over real adapters.
pub fn adapter_for(output_format: &str) -> Box<dyn CliAdapter> {
    match output_format.to_lowercase().as_str() {
        "html" => Box::new(GenericMarkdownAdapter { html: true }),
        "markdown" => Box::new(GenericMarkdownAdapter { html: false }),
        _ => Box::new(GenericRawPathAdapter),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_renders_image_link() {
        assert_eq!(GenericMarkdownAdapter { html: false }.render("/x/y.jpg"), "![image](/x/y.jpg)");
    }

    #[test]
    fn html_renders_img_tag() {
        assert_eq!(GenericMarkdownAdapter { html: true }.render("/x/y.jpg"), "<img src=\"/x/y.jpg\" />");
    }

    #[test]
    fn raw_emits_bare_path() {
        assert_eq!(GenericRawPathAdapter.render("/x/y.jpg"), "/x/y.jpg");
    }

    #[test]
    fn adapter_for_picks_by_format() {
        assert_eq!(adapter_for("markdown").name(), "generic_markdown");
        assert_eq!(adapter_for("html").name(), "generic_markdown");
        assert_eq!(adapter_for("raw").name(), "generic_raw_path");
        // unknown / empty → raw path (never accidentally Markdown-wraps)
        assert_eq!(adapter_for("").name(), "generic_raw_path");
        assert_eq!(adapter_for("nonsense").name(), "generic_raw_path");
    }

    #[test]
    fn adapter_for_is_case_insensitive() {
        assert_eq!(adapter_for("MARKDOWN").name(), "generic_markdown");
        assert_eq!(adapter_for("Html").name(), "generic_markdown");
    }
}
