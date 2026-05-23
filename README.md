# StatsForge

A GitHub language-stats card generator written in Rust.  
Paste a URL into your README and get a beautiful animated SVG card showing any user's top languages.

---

## Usage

No setup required. Just use the live URL:

```md
![GitHub Stats](https://statsforge.botond-balla.workers.dev/card?username=yourname)
```

Or in HTML:

```html
<img src="https://statsforge.botond-balla.workers.dev/card?username=yourname" alt="GitHub Stats" />
```

Swap `yourname` for any GitHub username. That's it.

---

## Query parameters

| Parameter            | Type    | Default | Allowed values          | Description                                   |
|----------------------|---------|---------|-------------------------|-----------------------------------------------|
| `username`           | string  | —       | any GitHub username     | **Required.** 1–39 alphanumeric/hyphen chars  |
| `theme`              | string  | `dark`  | `dark`, `light`         | Card colour scheme                            |
| `primaryColor`       | hex     | —       | `#rrggbb` or `#rgb`     | Override card background colour               |
| `accentColor`        | hex     | —       | `#rrggbb` or `#rgb`     | Override gradient end colour                  |
| `barAnimationSpeed`  | integer | `1000`  | `100`–`5000` (ms)       | Bar grow animation duration                   |
| `numberOfLanguages`  | integer | `5`     | `1`–`10`                | How many languages to show                    |
| `barHeight`          | integer | `10`    | `4`–`40` (px)           | Height of each bar                            |
| `cardWidth`          | integer | `400`   | `200`–`800` (px)        | Total card width                              |
| `showPercentages`    | boolean | `true`  | `true`, `false`         | Show or hide the `xx.x%` labels               |
| `textSize`           | float   | `1.0`   | `0.5`–`2.0`             | Font scale multiplier                         |
| `sortBy`             | string  | `bytes` | `bytes`, `repos`        | Rank languages by bytes written or repo count |
| `borderRadius`       | integer | `12`    | `0`–`40` (px)           | Corner rounding of the card                   |
| `excludeLanguages`   | string  | —       | comma-separated names   | Languages to hide (e.g. `HTML,CSS`)           |

### Full example — every parameter at once

```
https://statsforge.botond-balla.workers.dev/card
  ?username=yourname
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
https://statsforge.botond-balla.workers.dev/card?username=yourname&theme=dark&primaryColor=%231c1f26&accentColor=%232dc9a8&barAnimationSpeed=1000&numberOfLanguages=5&barHeight=10&cardWidth=400&showPercentages=true&textSize=1.0&sortBy=bytes&borderRadius=12&excludeLanguages=HTML,CSS,Dockerfile
```

### Presets

```
# Light theme
?username=yourname&theme=light

# Wide card, 8 languages, sorted by repo count
?username=yourname&cardWidth=600&numberOfLanguages=8&sortBy=repos

# Custom brand colours
?username=yourname&primaryColor=%230d1117&accentColor=%23ff6b35

# Compact — thin bars, no percentages, tight corners
?username=yourname&barHeight=6&showPercentages=false&borderRadius=4

# Slow dramatic animation, large text
?username=yourname&barAnimationSpeed=3000&textSize=1.4

# Exclude markup and config to focus on real code
?username=yourname&excludeLanguages=HTML,CSS,Dockerfile,Shell,Makefile
```

> **Note:** `#` in hex colours must be URL-encoded as `%23`.

---

## Self-hosting

Want to run your own instance?

```bash
# 1. Clone
git clone <repo-url> && cd statsforge

# 2. Set your GitHub token (classic token, read:user scope is enough)
cp .env.example .env

# 3. Run locally
cargo run --release
open "http://localhost:3000/card?username=yourname"

# 4. Deploy to Cloudflare Workers
wrangler secret put GITHUB_TOKEN
wrangler deploy
```

### Environment variables

| Variable        | Required | Default | Description                                 |
|-----------------|----------|---------|---------------------------------------------|
| `GITHUB_TOKEN`  | Yes      | —       | GitHub personal access token (`read:user`)  |
| `PORT`          | No       | `3000`  | TCP port for local server only              |

### Development

```bash
cargo test          # unit + integration tests
cargo check         # fast type-check
RUST_LOG=debug cargo run

cargo check --target wasm32-unknown-unknown --no-default-features --features workers

npx wrangler dev    # local Workers preview
npx wrangler deploy # deploy to Cloudflare
```
