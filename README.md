# StatsForge

A GitHub language-stats card generator written in Rust.  
Send a GET request → get back a beautiful animated SVG card showing a user's top languages.

```
GET /card?username=bllbtnd
```

---

## Quick start

```bash
# 1. Clone and enter the directory
git clone <repo-url> && cd statsforge

# 2. Create a .env file (copy the example)
cp .env.example .env
#    Then fill in GITHUB_TOKEN with a classic token that has read:user scope

# 3. Run
cargo run --release

# 4. Open in a browser
open "http://localhost:3000/card?username=bllbtnd"
```

---

## Query parameters

| Parameter            | Type    | Default | Allowed values          | Description                             |
|----------------------|---------|---------|-------------------------|-----------------------------------------|
| `username`           | string  | —       | any GitHub username     | **Required.** 1–39 alphanumeric/hyphen  |
| `theme`              | string  | `dark`  | `dark`, `light`         | Card colour scheme                      |
| `primaryColor`       | hex     | —       | `#rrggbb` or `#rgb`     | Override card background colour         |
| `accentColor`        | hex     | —       | `#rrggbb` or `#rgb`     | Override gradient end colour            |
| `barAnimationSpeed`  | integer | `1000`  | `100`–`5000` (ms)       | Bar grow animation duration             |
| `numberOfLanguages`  | integer | `5`     | `1`–`10`                | How many languages to show              |
| `barHeight`          | integer | `10`    | `4`–`40` (px)           | Height of each bar                      |
| `cardWidth`          | integer | `400`   | `200`–`800` (px)        | Total card width                        |
| `showPercentages`    | boolean | `true`  | `true`, `false`         | Show or hide the `xx.x%` labels         |
| `textSize`           | float   | `1.0`   | `0.5`–`2.0`             | Font scale multiplier                   |
| `sortBy`             | string  | `bytes` | `bytes`, `repos`        | Rank languages by bytes written or repo count |
| `borderRadius`       | integer | `12`    | `0`–`40` (px)           | Corner rounding of the card             |
| `excludeLanguages`   | string  | —       | comma-separated names   | Languages to hide (e.g. `HTML,CSS`)     |

### Full example — every parameter at once

```
https://statsforge.botond-balla.workers.dev/card
  ?username=bllbtnd
  &theme=dark
  &primaryColor=%231c1f26
  &accentColor=%232dc9a8
  &barAnimationSpeed=1000
  &numberOfLanguages=5
  &barHeight=10
  &cardWidth=400
  &showPercentages=true
  &textSize=1.0
  &sortBy=bytes
  &borderRadius=12
  &excludeLanguages=HTML,CSS,Dockerfile
```

As a single URL (copy-paste ready):

```
https://statsforge.botond-balla.workers.dev/card?username=bllbtnd&theme=dark&primaryColor=%231c1f26&accentColor=%232dc9a8&barAnimationSpeed=1000&numberOfLanguages=5&barHeight=10&cardWidth=400&showPercentages=true&textSize=1.0&sortBy=bytes&borderRadius=12&excludeLanguages=HTML,CSS,Dockerfile
```

### Other presets

```bash
# Light theme
?username=bllbtnd&theme=light

# Wide card with 8 languages sorted by repo count
?username=bllbtnd&cardWidth=600&numberOfLanguages=8&sortBy=repos

# Custom brand colours
?username=bllbtnd&primaryColor=%230d1117&accentColor=%23ff6b35

# Compact — thin bars, no percentages, tight corners
?username=bllbtnd&barHeight=6&showPercentages=false&borderRadius=4

# Slow dramatic animation, large text
?username=bllbtnd&barAnimationSpeed=3000&textSize=1.4

# Minimal square card
?username=bllbtnd&cardWidth=200&numberOfLanguages=3&borderRadius=0

# Exclude markup / config languages to focus on real code
?username=bllbtnd&excludeLanguages=HTML,CSS,Dockerfile,Shell,Makefile
```

> **Note:** `#` in hex colours must be URL-encoded as `%23`.

---

## Environment variables

| Variable        | Required | Default | Description                                   |
|-----------------|----------|---------|-----------------------------------------------|
| `GITHUB_TOKEN`  | Yes      | —       | GitHub personal access token (`read:user`)    |
| `PORT`          | No       | `3000`  | TCP port the server listens on (local only)   |

---

## Caching

Responses include `Cache-Control: public, max-age=3600` so CDNs and browsers
cache cards for one hour. The service is stateless — safe behind any reverse proxy or load balancer.

---

## Development

```bash
cargo test                                                         # unit + integration tests
cargo check                                                        # fast type-check
cargo build --release                                              # native binary
RUST_LOG=debug cargo run                                           # verbose logging

# Check wasm target compiles
cargo check --target wasm32-unknown-unknown --no-default-features --features workers

npx wrangler dev    # local Workers preview (uses real Cloudflare runtime)
npx wrangler deploy # deploy to https://statsforge.botond-balla.workers.dev
```
