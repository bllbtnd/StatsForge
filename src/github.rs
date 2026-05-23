use std::collections::HashMap;

use serde::Deserialize;

use crate::{error::AppError, params::SortBy};

const GITHUB_GRAPHQL: &str = "https://api.github.com/graphql";

/// A language entry returned to the SVG layer.
#[derive(Debug, Clone)]
pub struct LanguageStat {
    pub name: String,
    pub color: String,
    #[allow(dead_code)]
    pub bytes: u64,
    #[allow(dead_code)]
    pub repo_count: u32,
    pub percentage: f64,
}

// ── GraphQL response types ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GqlResponse {
    data: Option<GqlData>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Deserialize)]
struct GqlData {
    user: Option<GqlUser>,
}

#[derive(Deserialize)]
struct GqlUser {
    repositories: GqlRepoConnection,
}

#[derive(Deserialize)]
struct GqlRepoConnection {
    nodes: Vec<GqlRepo>,
}

#[derive(Deserialize)]
struct GqlRepo {
    languages: Option<GqlLanguageConnection>,
}

#[derive(Deserialize)]
struct GqlLanguageConnection {
    edges: Vec<GqlLanguageEdge>,
}

#[derive(Deserialize)]
struct GqlLanguageEdge {
    size: u64,
    node: GqlLanguageNode,
}

#[derive(Deserialize)]
struct GqlLanguageNode {
    name: String,
    color: Option<String>,
}

#[derive(Deserialize)]
struct GqlError {
    message: String,
}

// ── Shared aggregation logic ──────────────────────────────────────────────────

fn aggregate(gql: GqlResponse, username: &str, limit: u8, sort_by: SortBy, exclude: &[String]) -> Result<Vec<LanguageStat>, AppError> {
    if let Some(errors) = gql.errors {
        let msg = errors.into_iter().map(|e| e.message).collect::<Vec<_>>().join("; ");
        return Err(AppError::GitHub(msg));
    }

    let user = gql
        .data
        .and_then(|d| d.user)
        .ok_or_else(|| AppError::GitHub(format!("user '{}' not found", username)))?;

    struct Agg { bytes: u64, repos: u32, color: String }

    let mut map: HashMap<String, Agg> = HashMap::new();
    for repo in user.repositories.nodes {
        let Some(langs) = repo.languages else { continue };
        for edge in langs.edges {
            let entry = map.entry(edge.node.name.clone()).or_insert(Agg {
                bytes: 0,
                repos: 0,
                color: edge.node.color.unwrap_or_else(|| "#8b949e".to_string()),
            });
            entry.bytes += edge.size;
            entry.repos += 1;
        }
    }

    if map.is_empty() {
        return Err(AppError::GitHub(format!(
            "no public repository language data found for '{}'", username
        )));
    }

    // Drop excluded languages (case-insensitive — `exclude` is pre-lowercased).
    let mut entries: Vec<(String, Agg)> = map
        .into_iter()
        .filter(|(name, _)| !exclude.contains(&name.to_lowercase()))
        .collect();
    match sort_by {
        SortBy::Bytes => entries.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes)),
        SortBy::Repos => entries.sort_by(|a, b| b.1.repos.cmp(&a.1.repos)),
    }

    let total_bytes: u64 = entries.iter().map(|(_, a)| a.bytes).sum();

    Ok(entries
        .into_iter()
        .take(limit as usize)
        .map(|(name, agg)| LanguageStat {
            percentage: if total_bytes > 0 {
                (agg.bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            },
            name,
            color: agg.color,
            bytes: agg.bytes,
            repo_count: agg.repos,
        })
        .collect())
}

fn graphql_body(username: &str) -> serde_json::Value {
    // No privacy/fork filters — include all repos the token can see so that
    // accounts with only private repos or forked repos still get results.
    serde_json::json!({
        "query": r#"
            query($login: String!) {
              user(login: $login) {
                repositories(
                  first: 100
                  ownerAffiliations: [OWNER, COLLABORATOR]
                ) {
                  nodes {
                    languages(first: 10, orderBy: { field: SIZE, direction: DESC }) {
                      edges { size node { name color } }
                    }
                  }
                }
              }
            }
        "#,
        "variables": { "login": username }
    })
}

// ── Native (reqwest) implementation ───────────────────────────────────────────

#[cfg(feature = "native")]
pub async fn fetch_language_stats(
    client: &reqwest::Client,
    token: &str,
    username: &str,
    limit: u8,
    sort_by: SortBy,
    exclude: &[String],
) -> Result<Vec<LanguageStat>, AppError> {
    let resp = client
        .post(GITHUB_GRAPHQL)
        .bearer_auth(token)
        .header("User-Agent", "statsforge/0.1")
        .json(&graphql_body(username))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(AppError::GitHub(format!("GitHub API returned status {}", resp.status())));
    }

    let gql: GqlResponse = resp.json().await?;
    aggregate(gql, username, limit, sort_by, exclude)
}

// ── Workers (worker::Fetch) implementation ────────────────────────────────────

#[cfg(feature = "workers")]
pub async fn fetch_language_stats_workers(
    token: &str,
    username: &str,
    limit: u8,
    sort_by: SortBy,
    exclude: &[String],
) -> Result<Vec<LanguageStat>, worker::Error> {
    use worker::{wasm_bindgen::JsValue, Fetch, Headers, Method, Request as WReq, RequestInit};

    let body_str = serde_json::to_string(&graphql_body(username))
        .map_err(|e| worker::Error::RustError(e.to_string()))?;

    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {}", token))?;
    headers.set("Content-Type", "application/json")?;
    headers.set("User-Agent", "statsforge/0.1")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(JsValue::from_str(&body_str)));

    let request = WReq::new_with_init(GITHUB_GRAPHQL, &init)?;
    let mut resp = Fetch::Request(request).send().await?;

    if resp.status_code() != 200 {
        return Err(worker::Error::RustError(format!(
            "GitHub API returned status {}", resp.status_code()
        )));
    }

    let gql: GqlResponse = resp.json().await?;
    aggregate(gql, username, limit, sort_by, exclude)
        .map_err(|e| worker::Error::RustError(e.to_string()))
}
