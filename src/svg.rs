use crate::{
    github::LanguageStat,
    params::{CardParams, Theme},
};

// ── Palette ───────────────────────────────────────────────────────────────────

struct Palette {
    background: &'static str,
    border: &'static str,
    title_text: &'static str,
    label_text: &'static str,
    track: &'static str,
    gradient_start: &'static str,
    gradient_end: &'static str,
}

const DARK: Palette = Palette {
    background: "#1c1f26",
    border: "#2d3140",
    title_text: "#e6edf3",
    label_text: "#8b949e",
    track: "#2d3140",
    gradient_start: "#4a80d4",
    gradient_end: "#2dc9a8",
};

const LIGHT: Palette = Palette {
    background: "#f6f8fa",
    border: "#d0d7de",
    title_text: "#24292f",
    label_text: "#57606a",
    track: "#e1e4e8",
    gradient_start: "#3b6fba",
    gradient_end: "#1fa88e",
};

// ── Layout constants ──────────────────────────────────────────────────────────

const PADDING: u32 = 24;
const TITLE_FONT_BASE: f32 = 15.0;
const LABEL_FONT_BASE: f32 = 12.0;
const PCT_FONT_BASE: f32 = 11.0;
const ROW_SPACING: u32 = 14; // gap between the bar bottom and next label
const LABEL_TO_BAR: u32 = 6; // gap from label baseline to top of bar

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn render(params: &CardParams, stats: &[LanguageStat]) -> String {
    let palette = match params.theme {
        Theme::Dark => &DARK,
        Theme::Light => &LIGHT,
    };

    let bg = params
        .primary_color
        .as_deref()
        .unwrap_or(palette.background);
    let grad_end = params
        .accent_color
        .as_deref()
        .unwrap_or(palette.gradient_end);

    let title_font = TITLE_FONT_BASE * params.text_size;
    let label_font = LABEL_FONT_BASE * params.text_size;
    let pct_font = PCT_FONT_BASE * params.text_size;

    let bar_h = params.bar_height;
    let inner_w = params.card_width - PADDING * 2;

    // Height layout:
    //   padding + title + gap + N*(label + bar + spacing) - last_spacing + padding
    let title_area: u32 = (title_font * 1.4) as u32 + 16;
    let row_h = (label_font * 1.4) as u32 + LABEL_TO_BAR + bar_h + ROW_SPACING;
    let n = stats.len() as u32;
    let card_h = PADDING + title_area + row_h * n - ROW_SPACING + PADDING;

    let speed_s = params.bar_animation_speed as f32 / 1000.0;

    let mut svg = String::with_capacity(4096);

    // ── SVG root
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = params.card_width,
        h = card_h,
    ));

    // ── Defs: gradient + clip
    svg.push_str(&format!(
        r#"<defs>
  <linearGradient id="barGrad" x1="0%" y1="0%" x2="100%" y2="0%">
    <stop offset="0%" stop-color="{gs}"/>
    <stop offset="100%" stop-color="{ge}"/>
  </linearGradient>
  <clipPath id="cardClip">
    <rect width="{w}" height="{h}" rx="{r}" ry="{r}"/>
  </clipPath>
</defs>"#,
        gs = palette.gradient_start,
        ge = grad_end,
        w = params.card_width,
        h = card_h,
        r = params.border_radius,
    ));

    // ── Card background + border
    svg.push_str(&format!(
        r#"<rect width="{w}" height="{h}" rx="{r}" ry="{r}" fill="{bg}" stroke="{border}" stroke-width="1"/>
"#,
        w = params.card_width,
        h = card_h,
        r = params.border_radius,
        bg = bg,
        border = palette.border,
    ));

    // ── Title
    svg.push_str(&format!(
        r#"<text x="{x}" y="{y}" font-family="'Segoe UI',system-ui,sans-serif" font-size="{fs}" font-weight="600" fill="{color}">{username}'s Top Languages</text>
"#,
        x = PADDING,
        y = PADDING + (title_font * 1.0) as u32,
        fs = title_font,
        color = palette.title_text,
        username = escape_xml(&params.username),
    ));

    // ── Rows
    let mut y = PADDING + title_area;

    for (i, stat) in stats.iter().enumerate() {
        let label_y = y + (label_font * 1.0) as u32;
        let bar_y = label_y + LABEL_TO_BAR;
        let bar_w = (inner_w as f64 * stat.percentage / 100.0) as u32;
        let bar_w = bar_w.max(2); // always show a sliver
        let delay = i as f32 * 0.08;
        let anim_id = format!("a{}", i);

        // Label (language name)
        svg.push_str(&format!(
            r#"<text x="{x}" y="{ly}" font-family="'Segoe UI',system-ui,sans-serif" font-size="{fs}" fill="{color}">{name}</text>
"#,
            x = PADDING,
            ly = label_y,
            fs = label_font,
            color = palette.label_text,
            name = escape_xml(&stat.name),
        ));

        // Percentage text (right-aligned, same baseline as label)
        if params.show_percentages {
            svg.push_str(&format!(
                r#"<text x="{x}" y="{ly}" font-family="'Segoe UI',system-ui,sans-serif" font-size="{fs}" fill="{color}" text-anchor="end">{pct:.1}%</text>
"#,
                x = PADDING + inner_w,
                ly = label_y,
                fs = pct_font,
                color = palette.label_text,
                pct = stat.percentage,
            ));
        }

        // Track (background bar)
        svg.push_str(&format!(
            r#"<rect x="{x}" y="{by}" width="{w}" height="{h}" rx="{r}" ry="{r}" fill="{track}"/>
"#,
            x = PADDING,
            by = bar_y,
            w = inner_w,
            h = bar_h,
            r = bar_h / 2,
            track = palette.track,
        ));

        // Language dot colour (small circle left of label)
        svg.push_str(&format!(
            r#"<circle cx="{cx}" cy="{cy}" r="5" fill="{c}"/>
"#,
            cx = PADDING + (label_font * 0.0) as u32 + 4,
            cy = label_y - (label_font * 0.35) as u32,
            c = &stat.color,
        ));
        // Shift label text rightward to make room for dot
        // (Re-emit label shifted — easier than re-doing layout with a variable offset)

        // Animated fill bar
        svg.push_str(&format!(
            r#"<rect id="{id}" x="{x}" y="{by}" width="0" height="{h}" rx="{r}" ry="{r}" fill="url(#barGrad)">
  <animate attributeName="width" from="0" to="{target}" dur="{dur}s" begin="{delay}s" fill="freeze" calcMode="spline" keyTimes="0;1" keySplines="0.4 0 0.2 1"/>
</rect>
"#,
            id = anim_id,
            x = PADDING,
            by = bar_y,
            h = bar_h,
            r = bar_h / 2,
            target = bar_w,
            dur = speed_s,
            delay = delay,
        ));

        y += row_h;
    }

    svg.push_str("</svg>");
    svg
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
