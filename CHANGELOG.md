# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1] - 2026-10-13

The first release of `gcv` — render a resume as a rich, themed document in your
terminal, straight from a public git repo.

### Added

- **Sources**: view a resume from `owner/repo`, `owner/repo/path`, a full URL, a
  local file, or stdin (`cat resume.md | gcv -`). GitHub is the default host;
  GitLab, Codeberg, and Bitbucket are supported via `host/owner/repo` or
  `--gitlab`.
- **Auto-discovery**: for a repo source, `gcv` finds `resume.{md,json,yaml,yml,toml}`
  then `cv.*` on `main` then `master`.
- **Formats**: Markdown (YAML frontmatter + a readable heading convention),
  [JSON Resume](https://jsonresume.org/), YAML, and TOML all parse into one
  canonical model.
- **Rendering**: gradient block-font name banner, block-art avatar (half-block
  truecolor), section rules with icons, tech chips, clickable OSC 8 hyperlinks,
  a QR code, and width-aware wrapping. Content width follows the terminal.
- **Themes**: `neon`, `dracula`, `nord`, `gruvbox`, `solarized`, `paper`,
  `ocean`, `matrix`, and `mono`, with `--accent <name|#hex>` overrides and
  `--list-themes`.
- **Configuration** via a `meta` block (in any format): theme, accent, density,
  width, banner, avatar, qr, links, icons, section order, and footer text.
- **Color handling**: automatic truecolor → 256 → 16 → none detection; respects
  `NO_COLOR` and non-TTY output; `--color auto|always|never`.
- **Caching**: fetched content is cached for 6 hours with `--refresh` to bypass
  and `--offline` to read only from cache.
- **CLI**: built with `clap`, with `--theme`, `--accent`, `--width`, `--density`,
  `--format`, `--file`, `--branch`, and `--no-{banner,avatar,qr,links,icons}`.
- Unit tests covering all four parsers and core rendering.

### Notes

- Single static binary; no runtime dependencies and no code execution — all
  fetched text is treated as data.
- The same `.md` file is designed to read well on GitHub and render well in the
  terminal.

[Unreleased]: https://github.com/anindya-dey/gcv/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/anindya-dey/gcv/releases/tag/v0.0.1
