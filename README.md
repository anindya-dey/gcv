# gcv — a beautiful terminal resume

[![CI](https://github.com/anindya-dey/gcv/actions/workflows/ci.yml/badge.svg)](https://github.com/anindya-dey/gcv/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/gcv.svg)](https://crates.io/crates/gcv)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`gcv` renders a resume as a rich, themed document right in your terminal. Your
resume lives as a single file in a public git repo; anyone can view it with one
command.

```console
$ gcv anindya-dey/anindya-dey
```

```
 ███  █   █ █████ █   █ ████  █   █  ███        ████  █████ █   █
█   █ ██  █   █   ██  █ █   █  █ █  █   █       █   █ █      █ █
█████ █ █ █   █   █ █ █ █   █   █   █████       █   █ ████    █
█   █ █  ██   █   █  ██ █   █   █   █   █       █   █ █       █
█   █ █   █ █████ █   █ ████    █   █   █       ████  █████   █
Senior Machine Learning Engineer
Mumbai, India · hello@anindyadey.com · https://www.anindyadey.com
 anindyadey.com   Blog   GitHub   LinkedIn

● Summary ──────────────────────────────────────────────────────────────

  Senior Machine Learning Engineer with 10+ years of software development
  experience, specializing in full-stack platforms, MLOps pipelines, and
  scalable cloud infrastructure.

◆ Experience ───────────────────────────────────────────────────────────

  Senior Machine Learning Engineer                        Feb 2024 – Present
  Vista (A Cimpress Company) · Mumbai, India (WFH)
    • Architected and deployed MLOps pipeline reducing model deployment time by 60%
     Python   AWS SageMaker   Docker   Kubernetes   Snowflake
```

## Features

- **Git-native sharing** — your resume is just a file in a repo. No backend, no accounts.
- **Four formats** — Markdown (with YAML frontmatter), JSON Resume, YAML, and TOML all parse to the same model.
- **Beautiful by default** — gradient name banner, block-art avatar, clickable links, tech chips, section rules, and a QR code.
- **9 themes + accent override** — `neon`, `dracula`, `nord`, `gruvbox`, `solarized`, `paper`, `ocean`, `matrix`, `mono`.
- **Graceful degradation** — detects truecolor / 256 / 16 / no-color and adapts; respects `NO_COLOR`.
- **Fast and safe** — single static binary, no code execution, all fetched text is treated as data.
- **Cached and offline-friendly** — repeat views are instant; `--offline` never touches the network.

## Install

```console
cargo install gcv
# or, from a checkout
cargo install --path .
# or build a release binary
cargo build --release
```

## Usage

```console
gcv <SOURCE> [OPTIONS]
```

`SOURCE` can be any of:

| Source                        | Example                                   |
| ----------------------------- | ----------------------------------------- |
| `owner/repo` (profile repo)   | `gcv anindya-dey/anindya-dey`             |
| `owner/repo/path`             | `gcv anindya-dey/anindya-dey/resume.json` |
| Full URL                      | `gcv https://example.com/resume.md`       |
| Local file                    | `gcv ./resume.md`                         |
| Stdin                         | `cat resume.md \| gcv -`                  |

For `owner/repo`, `gcv` looks for `resume.{md,json,yaml,yml,toml}` then
`cv.*` on the `main` then `master` branch. Point at a specific file with
`--file` or a path.

### Examples

```console
gcv anindya-dey/anindya-dey                 # view from GitHub
gcv anindya-dey/anindya-dey --theme dracula # override the theme
gcv anindya-dey/anindya-dey --accent '#ff6b6b'
gcv gitlab.com/user/repo --gitlab           # GitLab
gcv ./resume.toml --no-avatar --width 100   # local, custom width
gcv --list-themes                           # see the palettes
```

## The file format

### Markdown (recommended)

The `.md` file is designed to look good on GitHub **and** render well in the
terminal. Put appearance config in YAML frontmatter and write the resume as
readable markdown:

```markdown
---
meta:
  theme: neon
  accent: cyan
  density: comfortable
  banner: true
  avatar: https://example.com/avatar.jpg
  qr: true
  sections: [summary, experience, projects, skills, education]
---

# Ada Lovelace

**Mathematician** · London, UK
[ada@example.com](mailto:ada@example.com) · [github.com/ada](https://github.com/ada)

## Summary

First programmer.

## Experience

### Mathematician · Analytical Engines
_1842 – 1843 · London_

Wrote the first algorithm.

- Programmed Bernoulli numbers on the Analytical Engine

`Notes` `Math`
```

Sections are recognized by heading: `Summary`, `Capabilities`, `Experience`,
`Projects`, `Skills`, `Education`, `Certifications`, `Languages`, `Interests`,
`Awards`. Unknown `##` sections are preserved and rendered as-is. Within
`Experience`/`Projects`/`Education`, use `###` for entries, an italic line for
dates/location, `-` bullets for highlights, and an inline-code line for tech
chips.

### JSON / YAML / TOML

These use the [JSON Resume](https://jsonresume.org/) schema (with an optional
`meta` block for the same appearance options), so they interoperate with the
wider ecosystem:

```json
{
  "meta": { "theme": "nord", "accent": "blue" },
  "basics": { "name": "Ada Lovelace", "label": "Mathematician" },
  "work": [
    {
      "name": "Analytical Engines",
      "position": "Mathematician",
      "startDate": "1842",
      "highlights": ["Wrote the first algorithm"]
    }
  ]
}
```

## Configuration (`meta`)

| Key        | Values                                              | Default       |
| ---------- | --------------------------------------------------- | ------------- |
| `theme`    | theme name                                          | `neon`        |
| `accent`   | color name or `#hex`                                | theme accent  |
| `density`  | `compact`, `comfortable`, `spacious`                | `comfortable` |
| `width`    | columns                                             | min(term, 96) |
| `banner`   | `true` / `false`                                    | `true`        |
| `avatar`   | image URL or path                                   | `basics.image`|
| `qr`       | `true` / `false`                                    | `true`        |
| `links`    | `true` / `false`                                    | `true`        |
| `icons`    | `true` / `false`                                    | `true`        |
| `sections` | ordered list of section keys                        | all present   |
| `footer`   | free-form footer text                               | none          |

CLI flags override the file's `meta`.

## Flags

```
--theme <NAME>       --accent <COLOR|#HEX>     --width <N>
--density <MODE>     --format <md|json|yaml|toml>
--file <NAME>        --branch <BRANCH>         --gitlab
--refresh            --offline
--no-banner --no-avatar --no-qr --no-links --no-icons
--color <auto|always|never>                    --list-themes
```

## How it works

1. Resolve `SOURCE` to raw content (git host, URL, file, or stdin).
2. Parse it into a single canonical model.
3. Render with the chosen theme, adapting to your terminal's capabilities.

Fetched content is cached under the OS cache dir (`~/.cache/gcv`,
`~/Library/Caches/gcv` on macOS) for 6 hours. Use `--refresh` to bypass and
`--offline` to read only from cache.

## License

MIT © Anindya Dey
