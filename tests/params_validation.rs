use statsforge::params::{CardParams, RawParams};

fn raw(username: &str) -> RawParams {
    RawParams {
        username: Some(username.into()),
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
        exclude_languages: None,
    }
}

#[test]
fn valid_username_accepted() {
    assert!(CardParams::from_raw(raw("botond")).is_ok());
    assert!(CardParams::from_raw(raw("my-user-123")).is_ok());
}

#[test]
fn empty_username_rejected() {
    assert!(CardParams::from_raw(raw("")).is_err());
    assert!(CardParams::from_raw(raw("   ")).is_err());
    let mut r = raw("x");
    r.username = None;
    assert!(CardParams::from_raw(r).is_err());
}

#[test]
fn username_with_invalid_chars_rejected() {
    assert!(CardParams::from_raw(raw("hello world")).is_err());
    assert!(CardParams::from_raw(raw("user@example")).is_err());
    assert!(CardParams::from_raw(raw("user/path")).is_err());
}

#[test]
fn username_too_long_rejected() {
    let long = "a".repeat(40);
    assert!(CardParams::from_raw(raw(&long)).is_err());
}

#[test]
fn valid_hex_colors_accepted() {
    let mut r = raw("user");
    r.primary_color = Some("#1a2b3c".into());
    r.accent_color = Some("abc".into()); // shorthand without #
    assert!(CardParams::from_raw(r).is_ok());
}

#[test]
fn invalid_hex_colors_rejected() {
    let mut r = raw("user");
    r.primary_color = Some("gggggg".into());
    assert!(CardParams::from_raw(r).is_err());

    let mut r2 = raw("user");
    r2.accent_color = Some("#12345".into()); // 5-digit is invalid
    assert!(CardParams::from_raw(r2).is_err());
}

#[test]
fn language_count_bounds_enforced() {
    let mut r = raw("user");
    r.number_of_languages = Some(0);
    assert!(CardParams::from_raw(r).is_err());

    let mut r2 = raw("user");
    r2.number_of_languages = Some(11);
    assert!(CardParams::from_raw(r2).is_err());

    let mut r3 = raw("user");
    r3.number_of_languages = Some(10);
    assert!(CardParams::from_raw(r3).is_ok());
}

#[test]
fn defaults_are_sensible() {
    let p = CardParams::from_raw(raw("user")).unwrap();
    assert_eq!(p.number_of_languages, 5);
    assert_eq!(p.card_width, 400);
    assert!(p.show_percentages);
    assert_eq!(p.border_radius, 12);
    assert!((p.text_size - 1.0).abs() < f32::EPSILON);
    assert!(p.exclude_languages.is_empty());
}

#[test]
fn exclude_languages_parsed_and_lowercased() {
    let mut r = raw("user");
    r.exclude_languages = Some("HTML,CSS, Markdown , dockerfile".into());
    let p = CardParams::from_raw(r).unwrap();
    assert_eq!(p.exclude_languages, vec!["html", "css", "markdown", "dockerfile"]);
}

#[test]
fn exclude_languages_empty_string_gives_empty_vec() {
    let mut r = raw("user");
    r.exclude_languages = Some("".into());
    let p = CardParams::from_raw(r).unwrap();
    assert!(p.exclude_languages.is_empty());
}

#[test]
fn exclude_languages_none_gives_empty_vec() {
    let p = CardParams::from_raw(raw("user")).unwrap();
    assert!(p.exclude_languages.is_empty());
}
