/// Embedded 404 Error Page HTML compiled directly into the binary
pub fn get_error_page_html(config_path: &str) -> String {
    // Escape HTML special characters to prevent broken markup from paths with quotes/brackets
    let escaped_path = config_path
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    const TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>404 - Not Found | Packer</title>
    <style>
        :root {
            --bg: #0f172a;
            --card-bg: #1e293b;
            --text-main: #f8fafc;
            --text-muted: #94a3b8;
            --accent: #38bdf8;
            --accent-hover: #0284c7;
            --border: #334155;
            --code-bg: #0f172a;
            --error-badge: #ef4444;
        }

        @media (prefers-color-scheme: light) {
            :root {
                --bg: #f8fafc;
                --card-bg: #ffffff;
                --text-main: #0f172a;
                --text-muted: #64748b;
                --accent: #0284c7;
                --accent-hover: #0369a1;
                --border: #e2e8f0;
                --code-bg: #f1f5f9;
                --error-badge: #dc2626;
            }
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
            font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }

        body {
            background-color: var(--bg);
            color: var(--text-main);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            padding: 1.5rem;
            transition: background-color 0.2s, color 0.2s;
        }

        .card {
            background-color: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 2.5rem;
            max-width: 480px;
            width: 100%;
            box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
            text-align: center;
        }

        .badge {
            display: inline-block;
            background-color: rgba(239, 68, 68, 0.15);
            color: var(--error-badge);
            font-size: 0.875rem;
            font-weight: 700;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            margin-bottom: 1rem;
        }

        h1 {
            font-size: 1.75rem;
            font-weight: 700;
            letter-spacing: -0.025em;
            margin-bottom: 0.5rem;
        }

        p.description {
            color: var(--text-muted);
            font-size: 0.95rem;
            line-height: 1.5;
            margin-bottom: 1.75rem;
        }

        .hint-box {
            background-color: var(--code-bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 1rem;
            font-size: 0.85rem;
            color: var(--text-muted);
            text-align: left;
            margin-bottom: 1.75rem;
            line-height: 1.4;
        }

        .hint-box code {
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            color: var(--accent);
            background: rgba(56, 189, 248, 0.1);
            padding: 0.15rem 0.35rem;
            border-radius: 4px;
        }
    </style>
</head>
<body>
    <div class="card">
        <span class="badge">404 ERROR</span>
        <h1>Content Not Found</h1>
        <p class="description">The requested URL or resource could not be found on this server.</p>

        <div class="hint-box">
            💡 <strong>Developer Tip:</strong> You can edit host directories and server rules in <code>{config_path}</code> or run <code>packer serve &lt;path&gt;</code> to serve a different directory.
        </div>
    </div>
</body>
</html>"#;

    TEMPLATE.replace("{config_path}", &escaped_path)
}
