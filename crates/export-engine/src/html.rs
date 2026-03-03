// Self-contained HTML report. Logic moved from peek-cli `render_html`.

use crate::markdown::render_markdown;
use crate::snapshot::ProcessSnapshot;

/// Render a process snapshot as a dark-themed standalone HTML document.
pub fn render_html(snapshot: &ProcessSnapshot) -> String {
    let md = render_markdown(snapshot);
    let name = &snapshot.process.name;
    let pid = snapshot.process.pid;
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>peek — {name} ({pid})</title>
<style>
  body{{font-family:monospace;max-width:960px;margin:2rem auto;padding:1rem;background:#0d1117;color:#c9d1d9}}
  h1{{color:#58a6ff}} h2{{color:#79c0ff;border-bottom:1px solid #30363d;padding-bottom:.3rem;margin-top:2rem}}
  table{{border-collapse:collapse;width:100%;margin:1rem 0}}
  th,td{{border:1px solid #30363d;padding:.4rem .8rem;text-align:left}}
  th{{background:#161b22;color:#58a6ff}}
  code{{background:#161b22;padding:.1rem .3rem;border-radius:3px;color:#79c0ff}}
  pre{{background:#161b22;padding:1rem;overflow-x:auto;border-radius:6px}}
  blockquote{{border-left:4px solid #388bfd;padding-left:1rem;color:#8b949e}}
</style>
</head>
<body>
<pre><code>{md_esc}</code></pre>
</body>
</html>"#,
        name = name,
        pid = pid,
        md_esc = md
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;"),
    )
}
