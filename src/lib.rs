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
    )
    .await?;

    let svg_body = svg::render(&card_params, &stats);

    let mut response = Response::ok(svg_body)?;
    response.headers_mut().set("Content-Type", "image/svg+xml")?;
    response.headers_mut().set("Cache-Control", "public, max-age=3600, s-maxage=3600")?;

    Ok(response)
}
