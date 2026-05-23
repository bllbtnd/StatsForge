pub mod config;
pub mod error;
pub mod github;
pub mod params;
pub mod svg;

// ── Cloudflare Workers entry point ────────────────────────────────────────────

#[cfg(feature = "workers")]
use worker::{event, Context, Env, Request, Response, Result, Router};

#[cfg(feature = "workers")]
#[event(fetch)]
async fn fetch_handler(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::ok("ok"))
        .get_async("/card", handle_card)
        .run(req, env)
        .await
}

#[cfg(feature = "workers")]
async fn handle_card(req: Request, ctx: worker::RouteContext<()>) -> Result<Response> {
    match try_handle_card(req, ctx).await {
        Ok(r) => Ok(r),
        Err(e) => {
            let msg = e.to_string();
            worker::console_error!("statsforge error: {}", msg);
            // Return an error card (SVG) so it still renders inside <img> tags.
            let svg = error_svg(&msg);
            let mut resp = Response::ok(svg)?;
            resp.headers_mut().set("Content-Type", "image/svg+xml")?;
            Ok(resp)
        }
    }
}

#[cfg(feature = "workers")]
async fn try_handle_card(req: Request, ctx: worker::RouteContext<()>) -> Result<Response> {
    let token = ctx.env.secret("GITHUB_TOKEN")?.to_string();

    let url = req.url()?;
    let query_str = url.query().unwrap_or("");
    let raw: params::RawParams = serde_urlencoded::from_str(query_str)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let card_params = params::CardParams::from_raw(raw)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let stats = github::fetch_language_stats_workers(
        &token,
        &card_params.username,
        card_params.number_of_languages,
        card_params.sort_by,
        &card_params.exclude_languages,
    )
    .await
    .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let svg_body = svg::render(&card_params, &stats);

    let mut response = Response::ok(svg_body)?;
    response.headers_mut().set("Content-Type", "image/svg+xml")?;
    response.headers_mut().set("Cache-Control", "public, max-age=3600, s-maxage=3600")?;

    Ok(response)
}

/// Minimal SVG that displays an error message — renders gracefully in browsers.
#[cfg(feature = "workers")]
fn error_svg(message: &str) -> String {
    let safe_msg = message
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    // Truncate long messages so they don't overflow the card.
    let display: String = safe_msg.chars().take(80).collect();
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="80" viewBox="0 0 400 80"><rect width="400" height="80" rx="12" fill="#1c1f26" stroke="#2d3140" stroke-width="1"/><text x="20" y="28" font-family="system-ui,sans-serif" font-size="13" font-weight="600" fill="#f85149">StatsForge Error</text><text x="20" y="52" font-family="system-ui,sans-serif" font-size="11" fill="#8b949e">{msg}</text></svg>"##,
        msg = display,
    )
}
