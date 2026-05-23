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

// ── Public API ────────────────────────────────────────────────────────────────

pub async fn fetch_language_stats(
    client: &reqwest::Client,
    token: &str,
    username: &str,
    limit: u8,
    sort_by: SortBy,
) -> Result<Vec<LanguageStat>, AppError> {
    // Fetch up to 100 public repos; for each collect language breakdown.
    let query = r#"
        query($login: String!) {
          user(login: $login) {
            repositories(
              first: 100
              isFork: false
              ownerAffiliations: OWNER
              privacy: PUBLIC
            ) {
              nodes {
                languages(first: 10, orderBy: { field: SIZE, direction: DESC }) {
                  edges {
                    size
                    node { name color }
                  }
                }
              }
            }
          }
        }
    "#;

    let body = serde_json::json!({
        "query": query,
        "variables": { "login": username }
    });

    let resp = client
        .post(GITHUB_GRAPHQL)
        .bearer_auth(token)
        .header("User-Agent", "statsforge/0.1")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(AppError::GitHub(format!(
            "GitHub API returned status {}",
            resp.status()
        )));
    }

    let gql: GqlResponse = resp.json().await?;

    if let Some(errors) = gql.errors {
        let msg = errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::GitHub(msg));
    }

    let user = gql
        .data
        .and_then(|d| d.user)
        .ok_or_else(|| AppError::GitHub(format!("user '{}' not found", username)))?;

    // Aggregate across repos.
    struct Agg {
        bytes: u64,
        repos: u32,
        color: String,
    }

    let mut map: HashMap<String, Agg> = HashMap::new();

    for repo in user.repositories.nodes {
        let Some(langs) = repo.languages else {
            continue;
        };
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
            "no public repository language data found for '{}'",
            username
        )));
    }

    // Sort by the requested field.
    let mut entries: Vec<(String, Agg)> = map.into_iter().collect();
    match sort_by {
        SortBy::Bytes => entries.sort_by(|a, b| b.1.bytes.cmp(&a.1.bytes)),
        SortBy::Repos => entries.sort_by(|a, b| b.1.repos.cmp(&a.1.repos)),
    }

    let total_bytes: u64 = entries.iter().map(|(_, a)| a.bytes).sum();

    let stats: Vec<LanguageStat> = entries
        .into_iter()
        .take(limit as usize)
        .map(|(name, agg)| {
            let percentage = if total_bytes > 0 {
                (agg.bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            LanguageStat {
                name,
                color: agg.color,
                bytes: agg.bytes,
                repo_count: agg.repos,
                percentage,
            }
        })
        .collect();

    Ok(stats)
}
