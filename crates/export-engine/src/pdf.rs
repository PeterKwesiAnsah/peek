// PDF via wkhtmltopdf/weasyprint/Chromium. Logic moved from peek-cli `run_export_pdf`.

use anyhow::Result;

use crate::html::render_html;
use crate::snapshot::ProcessSnapshot;

/// Render a PDF report for the given snapshot and return the output filename.
///
/// The implementation searches for a supported renderer (wkhtmltopdf,
/// weasyprint, Chromium/Chrome). It writes an HTML file to the system temp
/// directory, invokes the renderer, then removes the temp HTML file.
pub fn export_pdf(snapshot: &ProcessSnapshot) -> Result<String> {
    let html = render_html(snapshot);
    let info = &snapshot.process;

    // Find a PDF renderer (portable: use which crate instead of shell "which")
    let renderer = ["wkhtmltopdf", "weasyprint", "chromium", "google-chrome"]
        .iter()
        .find(|cmd| which::which(cmd).is_ok())
        .copied();

    let Some(renderer) = renderer else {
        anyhow::bail!(
            "PDF export requires wkhtmltopdf, weasyprint, or Chromium. \
             Install one and try again, or use --export html."
        );
    };

    let filename = format!("peek-{}-{}.pdf", info.name, info.pid);
    let out_path = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join(&filename);
    let out_path_str = out_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("output path is not valid UTF-8"))?
        .to_string();

    // Write HTML to a temp file
    let tmp = std::env::temp_dir().join(format!("peek-{}.html", info.pid));
    std::fs::write(&tmp, &html)?;
    let tmp_str = tmp.to_string_lossy();

    let status = match renderer {
        "wkhtmltopdf" => std::process::Command::new("wkhtmltopdf")
            .args([tmp_str.as_ref(), out_path_str.as_str()])
            .status()?,
        "weasyprint" => std::process::Command::new("weasyprint")
            .args([tmp_str.as_ref(), out_path_str.as_str()])
            .status()?,
        _ => std::process::Command::new(renderer)
            .args([
                "--headless",
                "--disable-gpu",
                "--print-to-pdf",
                &out_path_str,
                &format!("file://{}", tmp.display()),
            ])
            .status()?,
    };

    let _ = std::fs::remove_file(&tmp);

    if !status.success() {
        anyhow::bail!("{} exited with status {:?}", renderer, status.code());
    }

    Ok(out_path_str)
}
