/// Integration tests for the SVG renderer — no network calls required.
use statsforge::{
    github::LanguageStat,
    params::{CardParams, RawParams, Theme},
    svg,
};

fn sample_stats() -> Vec<LanguageStat> {
    vec![
        LanguageStat {
            name: "Rust".into(),
            color: "#dea584".into(),
            bytes: 60000,
            repo_count: 8,
            percentage: 60.0,
        },
        LanguageStat {
            name: "Python".into(),
            color: "#3572A5".into(),
            bytes: 25000,
            repo_count: 5,
            percentage: 25.0,
        },
        LanguageStat {
            name: "TypeScript".into(),
            color: "#3178c6".into(),
            bytes: 15000,
            repo_count: 3,
            percentage: 15.0,
        },
    ]
}

fn default_params() -> CardParams {
    CardParams::from_raw(RawParams {
        username: Some("testuser".into()),
        theme: None,
        primary_color: None,
        accent_color: None,
        bar_animation_speed: None,
        number_of_languages: None,
        bar_height: None,
        card_width: None,
        show_percentages: None,
        text_size: None,
        sort_by: None,
        border_radius: None,
    })
    .unwrap()
}

#[test]
fn svg_contains_svg_root() {
    let out = svg::render(&default_params(), &sample_stats());
    assert!(out.starts_with("<svg"), "output must start with <svg");
    assert!(out.ends_with("</svg>"), "output must end with </svg>");
}

#[test]
fn svg_contains_language_names() {
    let out = svg::render(&default_params(), &sample_stats());
    assert!(out.contains("Rust"), "Rust must appear in output");
    assert!(out.contains("Python"), "Python must appear in output");
    assert!(out.contains("TypeScript"), "TypeScript must appear in output");
}

#[test]
fn svg_contains_username() {
    let out = svg::render(&default_params(), &sample_stats());
    assert!(out.contains("testuser"), "username must appear in title");
}

#[test]
fn svg_contains_percentages_when_enabled() {
    let params = default_params();
    assert!(params.show_percentages);
    let out = svg::render(&params, &sample_stats());
    assert!(out.contains("60.0%"), "top language percentage must appear");
}

#[test]
fn svg_hides_percentages_when_disabled() {
    let mut params = default_params();
    params.show_percentages = false;
    let out = svg::render(&params, &sample_stats());
    assert!(!out.contains("60.0%"), "percentages must be absent");
}

#[test]
fn svg_light_theme_uses_light_background() {
    let mut params = default_params();
    params.theme = Theme::Light;
    let out = svg::render(&params, &sample_stats());
    assert!(out.contains("#f6f8fa"), "light background colour must appear");
}

#[test]
fn svg_dark_theme_uses_dark_background() {
    let params = default_params();
    assert_eq!(params.theme, Theme::Dark);
    let out = svg::render(&params, &sample_stats());
    assert!(out.contains("#1c1f26"), "dark background colour must appear");
}

#[test]
fn svg_custom_accent_color_applied() {
    let mut params = default_params();
    params.accent_color = Some("#ff0055".into());
    let out = svg::render(&params, &sample_stats());
    assert!(out.contains("#ff0055"), "custom accent colour must appear in gradient");
}

#[test]
fn svg_escapes_xml_in_username() {
    let mut params = default_params();
    params.username = "user&<>".into();
    let out = svg::render(&params, &sample_stats());
    assert!(out.contains("user&amp;&lt;&gt;"), "XML special chars must be escaped");
    assert!(!out.contains("user&<>"), "raw XML special chars must not appear");
}

#[test]
fn svg_empty_stats_produces_valid_svg() {
    let out = svg::render(&default_params(), &[]);
    assert!(out.starts_with("<svg"));
    assert!(out.ends_with("</svg>"));
}
