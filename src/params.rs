use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    #[default]
    Bytes,
    Repos,
}

/// Raw query parameters as deserialized from the request URL.
#[derive(Debug, Deserialize)]
pub struct RawParams {
    pub username: Option<String>,
    pub theme: Option<Theme>,
    #[serde(rename = "primaryColor")]
    pub primary_color: Option<String>,
    #[serde(rename = "accentColor")]
    pub accent_color: Option<String>,
    #[serde(rename = "barAnimationSpeed")]
    pub bar_animation_speed: Option<u32>,
    #[serde(rename = "numberOfLanguages")]
    pub number_of_languages: Option<u8>,
    #[serde(rename = "barHeight")]
    pub bar_height: Option<u32>,
    #[serde(rename = "cardWidth")]
    pub card_width: Option<u32>,
    #[serde(rename = "showPercentages")]
    pub show_percentages: Option<bool>,
    #[serde(rename = "textSize")]
    pub text_size: Option<f32>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<SortBy>,
    #[serde(rename = "borderRadius")]
    pub border_radius: Option<u32>,
    /// Comma-separated language names to exclude, e.g. `HTML,CSS,Markdown`.
    #[serde(rename = "excludeLanguages")]
    pub exclude_languages: Option<String>,
}

/// Validated, ready-to-use parameters.
#[derive(Debug, Clone)]
pub struct CardParams {
    pub username: String,
    pub theme: Theme,
    pub primary_color: Option<String>,
    pub accent_color: Option<String>,
    pub bar_animation_speed: u32,
    pub number_of_languages: u8,
    pub bar_height: u32,
    pub card_width: u32,
    pub show_percentages: bool,
    pub text_size: f32,
    pub sort_by: SortBy,
    pub border_radius: u32,
    /// Language names (lowercased) that should be excluded from results.
    pub exclude_languages: Vec<String>,
}

impl CardParams {
    pub fn from_raw(raw: RawParams) -> Result<Self, AppError> {
        let username = raw
            .username
            .filter(|u| !u.trim().is_empty())
            .ok_or_else(|| AppError::MissingParam("username".into()))?;

        // Sanitize username: only alphanumeric, hyphens, max 39 chars (GitHub limit).
        if username.len() > 39
            || !username
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err(AppError::InvalidParam {
                param: "username",
                reason: "must be 1–39 alphanumeric/hyphen characters".into(),
            });
        }

        let primary_color = raw
            .primary_color
            .map(|c| validate_hex_color(c, "primaryColor"))
            .transpose()?;

        let accent_color = raw
            .accent_color
            .map(|c| validate_hex_color(c, "accentColor"))
            .transpose()?;

        let number_of_languages = raw.number_of_languages.unwrap_or(5);
        if !(1..=10).contains(&number_of_languages) {
            return Err(AppError::InvalidParam {
                param: "numberOfLanguages",
                reason: "must be between 1 and 10".into(),
            });
        }

        let bar_animation_speed = raw.bar_animation_speed.unwrap_or(1000);
        if !(100..=5000).contains(&bar_animation_speed) {
            return Err(AppError::InvalidParam {
                param: "barAnimationSpeed",
                reason: "must be between 100 and 5000 ms".into(),
            });
        }

        let bar_height = raw.bar_height.unwrap_or(10);
        if !(4..=40).contains(&bar_height) {
            return Err(AppError::InvalidParam {
                param: "barHeight",
                reason: "must be between 4 and 40 px".into(),
            });
        }

        let card_width = raw.card_width.unwrap_or(400);
        if !(200..=800).contains(&card_width) {
            return Err(AppError::InvalidParam {
                param: "cardWidth",
                reason: "must be between 200 and 800 px".into(),
            });
        }

        let text_size = raw.text_size.unwrap_or(1.0);
        if !(0.5..=2.0).contains(&text_size) {
            return Err(AppError::InvalidParam {
                param: "textSize",
                reason: "must be between 0.5 and 2.0".into(),
            });
        }

        let border_radius = raw.border_radius.unwrap_or(12);
        if border_radius > 40 {
            return Err(AppError::InvalidParam {
                param: "borderRadius",
                reason: "must be between 0 and 40 px".into(),
            });
        }

        // Parse comma-separated exclusion list; normalise to lowercase for
        // case-insensitive matching against GitHub language names.
        let exclude_languages = raw
            .exclude_languages
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(CardParams {
            username,
            theme: raw.theme.unwrap_or_default(),
            primary_color,
            accent_color,
            bar_animation_speed,
            number_of_languages,
            bar_height,
            card_width,
            show_percentages: raw.show_percentages.unwrap_or(true),
            text_size,
            sort_by: raw.sort_by.unwrap_or_default(),
            border_radius,
            exclude_languages,
        })
    }
}

/// Validates and normalises a hex colour string to `#RRGGBB`.
fn validate_hex_color(color: String, param: &'static str) -> Result<String, AppError> {
    let s = color.trim_start_matches('#');
    let normalized = match s.len() {
        3 => {
            // Expand shorthand: `abc` → `aabbcc`
            s.chars()
                .flat_map(|c| [c, c])
                .collect::<String>()
        }
        6 => s.to_string(),
        _ => {
            return Err(AppError::InvalidParam {
                param,
                reason: "must be a valid 3- or 6-digit hex colour (e.g. #1a2b3c)".into(),
            })
        }
    };

    if !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::InvalidParam {
            param,
            reason: "contains invalid hex characters".into(),
        });
    }

    Ok(format!("#{}", normalized.to_lowercase()))
}
