//! gcv — a beautiful terminal resume renderer.

mod ansi;
mod avatar;
mod banner;
mod fetch;
mod inline;
mod model;
mod parse;
mod qr;
mod render;
mod theme;

use std::io::IsTerminal;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use ansi::Style;
use fetch::FetchOpts;
use render::{Density, RenderOpts};

#[derive(Parser, Debug)]
#[command(
    name = "gcv",
    version,
    about = "A beautiful terminal resume renderer",
    long_about = "Render a resume from a public git repo, URL, local file, or stdin.\n\n\
                  Examples:\n  \
                  gcv anindya-dey/anindya-dey\n  \
                  gcv anindya-dey/anindya-dey/resume.json\n  \
                  gcv https://example.com/resume.md\n  \
                  gcv ./resume.md\n  \
                  cat resume.md | gcv -",
    after_help = "Run `gcv --list-themes` to see available themes."
)]
struct Cli {
    /// Source: owner/repo, owner/repo/path, URL, local file, or - for stdin
    #[arg(value_name = "SOURCE")]
    source: Option<String>,

    /// Theme name (see --list-themes)
    #[arg(short, long)]
    theme: Option<String>,

    /// Accent color override (name like `cyan` or hex like `#7c3aed`)
    #[arg(short, long)]
    accent: Option<String>,

    /// Content width in columns
    #[arg(long)]
    width: Option<usize>,

    /// Force the input format instead of detecting it
    #[arg(short = 'F', long, value_enum)]
    format: Option<FormatArg>,

    /// Layout density
    #[arg(short, long, value_enum)]
    density: Option<DensityArg>,

    /// Color output mode
    #[arg(long, value_enum, default_value_t = ColorArg::Auto)]
    color: ColorArg,

    /// Disable the big name banner
    #[arg(long)]
    no_banner: bool,

    /// Disable avatar art
    #[arg(long)]
    no_avatar: bool,

    /// Disable the QR code
    #[arg(long)]
    no_qr: bool,

    /// Hide links in the footer
    #[arg(long)]
    no_links: bool,

    /// Hide section icons
    #[arg(long)]
    no_icons: bool,

    /// List available themes and exit
    #[arg(long)]
    list_themes: bool,

    /// Specific filename within the repo (e.g. resume.json)
    #[arg(long)]
    file: Option<String>,

    /// Git branch to read from (default: main, then master)
    #[arg(long)]
    branch: Option<String>,

    /// Bypass the local cache and re-fetch
    #[arg(long)]
    refresh: bool,

    /// Use only cached data; never touch the network
    #[arg(long)]
    offline: bool,

    /// Treat a bare owner/repo as a GitLab repository
    #[arg(long)]
    gitlab: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum ColorArg {
    Auto,
    Always,
    Never,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum FormatArg {
    Md,
    Json,
    Yaml,
    Toml,
}

impl From<FormatArg> for parse::Format {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Md => parse::Format::Markdown,
            FormatArg::Json => parse::Format::Json,
            FormatArg::Yaml => parse::Format::Yaml,
            FormatArg::Toml => parse::Format::Toml,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, ValueEnum)]
enum DensityArg {
    Compact,
    Comfortable,
    Spacious,
}

impl From<DensityArg> for Density {
    fn from(d: DensityArg) -> Self {
        match d {
            DensityArg::Compact => Density::Compact,
            DensityArg::Comfortable => Density::Comfortable,
            DensityArg::Spacious => Density::Spacious,
        }
    }
}

fn parse_density(s: &str) -> Option<Density> {
    match s.to_lowercase().as_str() {
        "compact" | "tight" => Some(Density::Compact),
        "comfortable" | "normal" => Some(Density::Comfortable),
        "spacious" | "roomy" | "relaxed" => Some(Density::Spacious),
        _ => None,
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.list_themes {
        list_themes();
        return ExitCode::SUCCESS;
    }

    ansi::set_mode(ansi::detect(
        cli.color == ColorArg::Always,
        cli.color == ColorArg::Never,
    ));

    let source = match cli.source.clone() {
        Some(s) => s,
        None => {
            if std::io::stdin().is_terminal() {
                eprintln!(
                    "error: no source given. Try `gcv anindya-dey/anindya-dey` or `gcv --help`."
                );
                return ExitCode::from(2);
            }
            "-".to_string()
        }
    };

    let fopts = FetchOpts {
        file: cli.file.clone(),
        branch: cli.branch.clone(),
        refresh: cli.refresh,
        offline: cli.offline,
        gitlab: cli.gitlab,
    };

    let loaded = match fetch::load(&source, &fopts) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let format = cli.format.map(parse::Format::from).unwrap_or(loaded.format);

    let resume = match parse::parse(&loaded.content, format) {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "error: could not parse {} as {}: {e}",
                loaded.label,
                format.label()
            );
            return ExitCode::FAILURE;
        }
    };

    let theme_name = cli
        .theme
        .clone()
        .or_else(|| resume.meta.theme.clone())
        .unwrap_or_else(|| "neon".to_string());
    let mut theme = match theme::Theme::get(&theme_name) {
        Some(t) => t,
        None => {
            eprintln!(
                "warning: unknown theme `{theme_name}`; using `neon`. Available: {}",
                theme::Theme::list().join(", ")
            );
            theme::Theme::get("neon").unwrap()
        }
    };
    if let Some(accent) = cli.accent.clone().or_else(|| resume.meta.accent.clone()) {
        theme = theme.with_accent(&accent);
    }

    let term_w = terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .unwrap_or(100);
    let width = cli
        .width
        .or(resume.meta.width)
        .unwrap_or(term_w.min(96))
        .clamp(40, 240);

    let banner = !cli.no_banner && resume.meta.banner.unwrap_or(true);
    let avatar = !cli.no_avatar;
    let qr = !cli.no_qr && resume.meta.qr.unwrap_or(true);
    let links = !cli.no_links && resume.meta.links.unwrap_or(true);
    let icons = !cli.no_icons && resume.meta.icons.unwrap_or(true);
    let density = cli
        .density
        .map(Density::from)
        .or_else(|| resume.meta.density.as_deref().and_then(parse_density))
        .unwrap_or(Density::Comfortable);

    let opts = RenderOpts {
        theme,
        width,
        banner,
        avatar,
        qr,
        links,
        icons,
        density,
        source: Some(loaded.label.clone()),
    };

    print!("{}", render::render(&resume, &opts));
    ExitCode::SUCCESS
}

fn list_themes() {
    println!("{}", Style::new().bold().paint("Available themes"));
    println!();
    for name in theme::Theme::list() {
        let t = theme::Theme::get(name).unwrap();
        let swatch = format!(
            "{}{}{}{}{}",
            Style::new().fg(t.accent).paint("██"),
            Style::new().fg(t.accent2).paint("██"),
            Style::new().fg(t.heading).paint("██"),
            Style::new().fg(t.fg).paint("██"),
            Style::new().fg(t.muted).paint("██"),
        );
        println!("  {swatch}  {}", Style::new().fg(t.fg).bold().paint(t.name));
    }
    println!();
    println!(
        "{}",
        Style::new()
            .fg((120, 120, 120))
            .paint("Override the accent with `--accent <color|#hex>`.")
    );
}
