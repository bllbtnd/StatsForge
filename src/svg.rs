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
/// Space from the bottom of one bar to the top of the next row's label.
const ROW_GAP: u32 = 16;
/// Space between the label baseline and the top edge of the bar.
const LABEL_TO_BAR: u32 = 7;
/// Dot radius and the horizontal space it takes (dot_r*2 + gap).
const DOT_R: u32 = 4;
const DOT_AREA: u32 = DOT_R * 2 + 6; // 14 px

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

    // Each row: label line height + gap-to-bar + bar height + row gap.
    // The last row omits the trailing ROW_GAP.
    let label_h = (label_font * 1.4) as u32;
    let row_h = label_h + LABEL_TO_BAR + bar_h + ROW_GAP;
    let n = stats.len() as u32;
    let title_area = (title_font * 1.4) as u32 + 16;
    let card_h = PADDING + title_area + row_h * n - ROW_GAP + PADDING;

    let speed_s = params.bar_animation_speed as f32 / 1000.0;

    let mut svg = String::with_capacity(4096);

    // Root element
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#,
        w = params.card_width,
        h = card_h,
    ));

    // Gradient definition
    svg.push_str(&format!(
        r#"<defs>
  <linearGradient id="barGrad" x1="0%" y1="0%" x2="100%" y2="0%">
    <stop offset="0%" stop-color="{gs}"/>
    <stop offset="100%" stop-color="{ge}"/>
  </linearGradient>
</defs>
"#,
        gs = palette.gradient_start,
        ge = grad_end,
    ));

    // Card background and border
    svg.push_str(&format!(
        r#"<rect width="{w}" height="{h}" rx="{r}" ry="{r}" fill="{bg}" stroke="{border}" stroke-width="1"/>
"#,
        w = params.card_width,
        h = card_h,
        r = params.border_radius,
        bg = bg,
        border = palette.border,
    ));

    // Title
    svg.push_str(&format!(
        r#"<text x="{x}" y="{y}" font-family="'Segoe UI',system-ui,sans-serif" font-size="{fs:.1}" font-weight="600" fill="{color}">{name}'s Top Languages</text>
"#,
        x = PADDING,
        y = PADDING + (title_font * 1.0) as u32,
        fs = title_font,
        color = palette.title_text,
        name = escape_xml(&params.username),
    ));

    // Language rows
    let mut row_top = PADDING + title_area;

    for (i, stat) in stats.iter().enumerate() {
        // Vertical anchors for this row
        let label_baseline = row_top + label_h;
        let bar_top = label_baseline + LABEL_TO_BAR;
        // Dot vertically centred on the label cap-height (≈ 70 % of font size above baseline)
        let dot_cy = label_baseline - (label_font * 0.35) as u32;
        let dot_cx = PADDING + DOT_R;
        // Label and percentage share the same baseline; label starts after dot
        let label_x = PADDING + DOT_AREA;
        let pct_x = PADDING + inner_w;

        // Bar width is proportional to this language's share of total bytes
        let bar_fill_w = ((inner_w as f64 * stat.percentage / 100.0) as u32).max(2);

        let delay_s = i as f32 * 0.10;

        // Colour dot
        svg.push_str(&format!(
            r#"<circle cx="{cx}" cy="{cy}" r="{r}" fill="{c}"/>
"#,
            cx = dot_cx,
            cy = dot_cy,
            r = DOT_R,
            c = &stat.color,
        ));

        // Language name
        svg.push_str(&format!(
            r#"<text x="{x}" y="{y}" font-family="'Segoe UI',system-ui,sans-serif" font-size="{fs:.1}" fill="{color}">{name}</text>
"#,
            x = label_x,
            y = label_baseline,
            fs = label_font,
            color = palette.label_text,
            name = escape_xml(&stat.name),
        ));

        // Percentage label (right-aligned)
        if params.show_percentages {
            svg.push_str(&format!(
                r#"<text x="{x}" y="{y}" font-family="'Segoe UI',system-ui,sans-serif" font-size="{fs:.1}" fill="{color}" text-anchor="end">{pct:.1}%</text>
"#,
                x = pct_x,
                y = label_baseline,
                fs = pct_font,
                color = palette.label_text,
                pct = stat.percentage,
            ));
        }

        // Track (background pill)
        svg.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" ry="{r}" fill="{track}"/>
"#,
            x = PADDING,
            y = bar_top,
            w = inner_w,
            h = bar_h,
            r = bar_h / 2,
            track = palette.track,
        ));

        // Animated fill pill
        svg.push_str(&format!(
            r#"<rect x="{x}" y="{y}" width="0" height="{h}" rx="{r}" ry="{r}" fill="url(#barGrad)">
  <animate attributeName="width" from="0" to="{target}" dur="{dur:.2}s" begin="{delay:.2}s" fill="freeze" calcMode="spline" keyTimes="0;1" keySplines="0.4 0 0.2 1"/>
</rect>
"#,
            x = PADDING,
            y = bar_top,
            h = bar_h,
            r = bar_h / 2,
            target = bar_fill_w,
            dur = speed_s,
            delay = delay_s,
        ));

        row_top += row_h;
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
