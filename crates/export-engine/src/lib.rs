//! Export process snapshots to JSON, Markdown, HTML, and PDF.
//!
//! Pure formatting; depends on peek-core for `ProcessInfo` and `ProcessSnapshot`.

pub mod html;
pub mod json;
pub mod markdown;
pub mod pdf;
pub mod snapshot;

pub use html::render_html;
pub use json::to_json;
pub use markdown::render_markdown;
pub use pdf::export_pdf;
pub use snapshot::ProcessSnapshot;
