# StatsForge

A GitHub language-stats card generator written in Rust.  
Send a GET request → get back a beautiful animated SVG card showing a user's top languages.

```
GET /card?username=botond
```

![example card](https://i.imgur.com/placeholder.png)

---

## Quick start

```bash
# 1. Clone and enter the directory
git clone <repo-url> && cd statsforge

# 2. Create a .env file (copy the example)
cp .env.example .env
#    Then fill in GITHUB_TOKEN with a classic token that has `read:user` scope

# 3. Run
cargo run --release

# 4. Open in a browser
open "http://localhost:3000/card?username=botond"
```

---

## Query parameters

| Parameter            | Type    | Default | Description                                      |
|----------------------|---------|---------|--------------------------------------------------|
| `username`           | string  | —       | **Required.** GitHub username (1–39 chars)       |
| `theme`              | string  | `dark`  | `dark` or `light`                                |
| `primaryColor`       | hex     | —       | Override card background colour                  |
| `accentColor`        | hex     | —       | Override gradient end colour                     |
| `barAnimationSpeed`  | ms      | `1000`  | Animation duration (100–5000 ms)                 |
| `numberOfLanguages`  | integer | `5`     | How many languages to show (1–10)                |
| `barHeight`          | px      | `10`    | Bar height in pixels (4–40)                      |
| `cardWidth`          | px      | `400`   | Card width in pixels (200–800)                   |
| `showPercentages`    | boolean | `true`  | Show or hide the percentage labels               |
| `textSize`           | float   | `1.0`   | Font scale multiplier (0.5–2.0)                  |
| `sortBy`             | string  | `bytes` | `bytes` or `repos`                               |
| `borderRadius`       | px      | `12`    | Corner radius (0–40)                             |

### Examples

```
# Light theme, wider card
/card?username=botond&theme=light&cardWidth=500

# Top 3 languages sorted by repo count, custom accent
/card?username=botond&numberOfLanguages=3&sortBy=repos&accentColor=%23ff6b35

# Minimal — no percentages, thin bars
/card?username=botond&showPercentages=false&barHeight=6
```

---

## Environment variables

| Variable        | Required | Default | Description                                   |
|-----------------|----------|---------|-----------------------------------------------|
| `GITHUB_TOKEN`  | Yes      | —       | GitHub personal access token (`read:user`)    |
| `PORT`          | No       | `3000`  | TCP port the server listens on                |

---

## Caching

The response includes `Cache-Control: public, max-age=3600` so CDNs and browsers
cache cards for one hour. The service itself is stateless — safe to run behind any
reverse proxy or load balancer.

---

## Development

```bash
cargo test          # run unit + integration tests
cargo check         # fast type-check without codegen
cargo build --release
RUST_LOG=debug cargo run  # verbose logging
```
