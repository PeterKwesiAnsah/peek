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

    // Find a PDF renderer
    let renderer = ["wkhtmltopdf", "weasyprint", "chromium", "google-chrome"]
        .iter()
        .find(|cmd| which_cmd(cmd))
        .copied();

    let Some(renderer) = renderer else {
        anyhow::bail!(
            "PDF export requires wkhtmltopdf, weasyprint, or Chromium. \
             Install one and try again, or use --export html."
        );
    };

    let filename = format!("peek-{}-{}.pdf", info.name, info.pid);

    // Write HTML to a temp file
    let tmp = std::env::temp_dir().join(format!("peek-{}.html", info.pid));
    std::fs::write(&tmp, &html)?;

    let status = match renderer {
        "wkhtmltopdf" => std::process::Command::new("wkhtmltopdf")
            .args([tmp.to_str().unwrap(), &filename])
            .status()?,
        "weasyprint" => std::process::Command::new("weasyprint")
            .args([tmp.to_str().unwrap(), &filename])
            .status()?,
        _ => std::process::Command::new(renderer)
            .args([
                "--headless",
                "--disable-gpu",
                "--print-to-pdf",
                &filename,
                &format!("file://{}", tmp.display()),
            ])
            .status()?,
    };

    let _ = std::fs::remove_file(&tmp);

    if !status.success() {
        anyhow::bail!("{} exited with status {:?}", renderer, status.code());
    }

    Ok(filename)
}

fn which_cmd(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
