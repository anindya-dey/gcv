//! The terminal renderer: turns a `Resume` into a styled document.

use crate::ansi::{self, vwidth, ColorMode, Style};
use crate::avatar;
use crate::banner;
use crate::inline;
use crate::model::{ExtraSection, Resume};
use crate::qr;
use crate::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Compact,
    Comfortable,
    Spacious,
}

pub struct RenderOpts {
    pub theme: Theme,
    pub width: usize,
    pub banner: bool,
    pub avatar: bool,
    pub qr: bool,
    pub links: bool,
    pub icons: bool,
    pub density: Density,
    pub source: Option<String>,
}

fn sep(theme: &Theme) -> String {
    Style::new().fg(theme.subtle).paint(" · ")
}

fn icon_for(key: &str) -> &'static str {
    match key {
        "summary" => "●",
        "capabilities" => "✦",
        "experience" => "◆",
        "projects" => "▲",
        "skills" => "▸",
        "education" => "△",
        "certifications" => "❯",
        "languages" => "◈",
        "interests" => "✧",
        "awards" => "★",
        _ => "◇",
    }
}

fn title_for(key: &str) -> String {
    match key {
        "summary" => "Summary",
        "capabilities" => "What I Do",
        "experience" => "Experience",
        "projects" => "Projects",
        "skills" => "Skills",
        "education" => "Education",
        "certifications" => "Certifications",
        "languages" => "Languages",
        "interests" => "Interests",
        "awards" => "Awards",
        other => other,
    }
    .to_string()
}

pub fn render(r: &Resume, opts: &RenderOpts) -> String {
    let mut out: Vec<String> = Vec::new();

    render_header(r, opts, &mut out);
    out.push(String::new());

    for key in r.section_order() {
        match key.as_str() {
            "summary" => section_summary(r, opts, &mut out),
            "capabilities" => section_capabilities(r, opts, &mut out),
            "experience" => section_experience(r, opts, &mut out),
            "projects" => section_projects(r, opts, &mut out),
            "skills" => section_skills(r, opts, &mut out),
            "education" => section_education(r, opts, &mut out),
            "certifications" => section_certifications(r, opts, &mut out),
            "languages" => section_languages(r, opts, &mut out),
            "interests" => section_interests(r, opts, &mut out),
            "awards" => section_awards(r, opts, &mut out),
            "extra" => section_extra(r, opts, &mut out),
            _ => {}
        }
        out.push(String::new());
        if opts.density == Density::Spacious {
            out.push(String::new());
        }
    }

    render_footer(r, opts, &mut out);

    while out.last().map(|l| l.is_empty()).unwrap_or(false) {
        out.pop();
    }
    let mut s = out.join("\n");
    s.push('\n');
    s
}

fn render_header(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    let name = r
        .basics
        .name
        .clone()
        .unwrap_or_else(|| "Resume".to_string());

    let av_src = r.meta.avatar.clone().or_else(|| r.basics.image.clone());
    let avatar_art = if opts.avatar && ansi::mode() == ColorMode::TrueColor {
        av_src.as_deref().and_then(|s| avatar::render(s, 22))
    } else {
        None
    };
    let av_w = avatar_art
        .as_ref()
        .map(|a| a.iter().map(|l| vwidth(l)).max().unwrap_or(0))
        .unwrap_or(0);
    let gap = 3;
    let text_w = if avatar_art.is_some() {
        w.saturating_sub(av_w + gap)
    } else {
        w
    };

    let mut text: Vec<String> = Vec::new();

    let mut banner_done = false;
    if opts.banner {
        let bl = banner::render(&name);
        if !bl.is_empty() && banner::width(&name) <= text_w {
            let n = bl.len().max(2);
            for (i, line) in bl.iter().enumerate() {
                let t = i as f32 / (n - 1) as f32;
                let c = ansi::lerp(theme.accent, theme.accent2, t);
                text.push(Style::new().fg(c).bold().paint(line));
            }
            banner_done = true;
        }
    }
    if !banner_done {
        text.push(Style::new().fg(theme.heading).bold().paint(&name));
    }

    if let Some(label) = &r.basics.label {
        text.push(Style::new().fg(theme.accent).bold().paint(label));
    }

    let mut contacts: Vec<String> = Vec::new();
    if let Some(loc) = r.location_text() {
        contacts.push(Style::new().fg(theme.muted).paint(&loc));
    }
    if let Some(email) = &r.basics.email {
        contacts.push(Style::new().fg(theme.muted).paint(email));
    }
    if let Some(url) = &r.basics.url {
        contacts.push(Style::new().fg(theme.muted).underline().paint(url));
    }
    if !contacts.is_empty() {
        let joined = contacts.join(&sep(theme));
        for line in textwrap::wrap(&joined, text_w.max(10)) {
            text.push(line.to_string());
        }
    }

    if opts.links && !r.basics.profiles.is_empty() {
        let chips: Vec<(String, usize)> = r
            .basics
            .profiles
            .iter()
            .map(|p| {
                let label = p
                    .network
                    .clone()
                    .or_else(|| p.username.clone())
                    .unwrap_or_default();
                chip(&label, theme)
            })
            .collect();
        text.extend(wrap_chips(chips, text_w.max(10)));
    }

    match avatar_art {
        Some(av) => {
            let n = av.len().max(text.len());
            for i in 0..n {
                let left = av.get(i).cloned().unwrap_or_default();
                let lpad = av_w.saturating_sub(vwidth(&left));
                let right = text.get(i).cloned().unwrap_or_default();
                out.push(format!(
                    "{left}{}{}{right}",
                    " ".repeat(lpad),
                    " ".repeat(gap)
                ));
            }
        }
        None => {
            for line in text {
                out.push(line);
            }
        }
    }
}

fn section_header(opts: &RenderOpts, key: &str, title: &str) -> String {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    let prefix = if opts.icons {
        format!("{} ", icon_for(key))
    } else {
        String::new()
    };
    let label = format!("{prefix}{title}");
    let used = vwidth(&label);
    let rule_len = w.saturating_sub(used + 2);
    let styled = Style::new().fg(theme.accent).bold().paint(&label);
    let rule = Style::new().fg(theme.rule).paint(&"─".repeat(rule_len));
    format!("{styled} {rule}")
}

fn push_para(
    out: &mut Vec<String>,
    text: &str,
    _theme: &Theme,
    width: usize,
    indent: usize,
    style: Style,
) {
    let avail = width.saturating_sub(indent).max(10);
    for line in textwrap::wrap(text, avail) {
        out.push(format!("{}{}", " ".repeat(indent), style.paint(&line)));
    }
}

fn push_bullet(out: &mut Vec<String>, text: &str, theme: &Theme, width: usize, indent: usize) {
    let marker = Style::new().fg(theme.accent).paint("• ");
    let avail = width.saturating_sub(indent + 2).max(10);
    let wrapped = textwrap::wrap(text, avail);
    for (i, line) in wrapped.iter().enumerate() {
        if i == 0 {
            out.push(format!(
                "{}{}{}",
                " ".repeat(indent),
                marker,
                Style::new().fg(theme.fg).paint(line)
            ));
        } else {
            out.push(format!(
                "{}{}{}",
                " ".repeat(indent),
                "  ",
                Style::new().fg(theme.fg).paint(line)
            ));
        }
    }
}

fn push_labeled(
    out: &mut Vec<String>,
    label: &str,
    text: &str,
    label_w: usize,
    width: usize,
    indent: usize,
    theme: &Theme,
) {
    let avail = width.saturating_sub(indent + label_w).max(10);
    let wrapped = textwrap::wrap(text, avail);
    let padded = format!("{:<width$}", label, width = label_w);
    for (i, line) in wrapped.iter().enumerate() {
        if i == 0 {
            out.push(format!(
                "{}{}{}",
                " ".repeat(indent),
                Style::new().fg(theme.accent2).bold().paint(&padded),
                Style::new().fg(theme.fg).paint(line)
            ));
        } else {
            out.push(format!(
                "{}{}{}",
                " ".repeat(indent),
                " ".repeat(label_w),
                Style::new().fg(theme.fg).paint(line)
            ));
        }
    }
}

fn push_two_col(out: &mut Vec<String>, left: &str, right: &str, width: usize, indent: usize) {
    let pad = " ".repeat(indent);
    if right.is_empty() {
        out.push(format!("{pad}{left}"));
        return;
    }
    let iw = width.saturating_sub(indent);
    let lw = vwidth(left);
    let rw = vwidth(right);
    if lw + rw + 2 <= iw {
        let fill = iw - lw - rw;
        out.push(format!("{pad}{left}{}{right}", " ".repeat(fill)));
    } else {
        out.push(format!("{pad}{left}"));
        out.push(format!("{pad}{right}"));
    }
}

fn section_summary(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    out.push(section_header(opts, "summary", &title_for("summary")));
    out.push(String::new());
    if let Some(s) = &r.basics.summary {
        push_para(
            out,
            s,
            theme,
            opts.width.max(40),
            2,
            Style::new().fg(theme.fg),
        );
    }
}

fn section_capabilities(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(
        opts,
        "capabilities",
        &title_for("capabilities"),
    ));
    out.push(String::new());
    for (i, c) in r.capabilities.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        let name = Style::new().fg(theme.accent2).bold().paint(&c.name);
        out.push(format!("  {name}"));
        if let Some(d) = &c.description {
            push_para(out, d, theme, w, 4, Style::new().fg(theme.fg));
        }
        if let Some(m) = &c.metrics {
            let marker = Style::new().fg(theme.accent).paint("▸ ");
            let avail = w.saturating_sub(6).max(10);
            for (j, line) in textwrap::wrap(m, avail).iter().enumerate() {
                if j == 0 {
                    out.push(format!(
                        "    {marker}{}",
                        Style::new().fg(theme.accent).italic().paint(line)
                    ));
                } else {
                    out.push(format!(
                        "      {}",
                        Style::new().fg(theme.accent).italic().paint(line)
                    ));
                }
            }
        }
    }
}

fn section_experience(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(opts, "experience", &title_for("experience")));
    out.push(String::new());
    for (i, job) in r.work.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        let role = job.position.clone().unwrap_or_else(|| job.name.clone());
        let dates = Resume::date_range(job.start_date.as_deref(), job.end_date.as_deref());
        let left = Style::new().fg(theme.heading).bold().paint(&role);
        let right = Style::new().fg(theme.muted).paint(&dates);
        push_two_col(out, &left, &right, w, 2);

        let mut meta: Vec<String> = Vec::new();
        if !job.name.is_empty() {
            meta.push(Style::new().fg(theme.accent2).bold().paint(&job.name));
        }
        if let Some(loc) = &job.location {
            meta.push(Style::new().fg(theme.muted).paint(loc));
        }
        if !meta.is_empty() {
            out.push(format!("  {}", meta.join(&sep(theme))));
        }
        if let Some(d) = &job.description {
            push_para(out, d, theme, w, 4, Style::new().fg(theme.fg));
        }
        for h in &job.highlights {
            push_bullet(out, h, theme, w, 4);
        }
        if !job.keywords.is_empty() {
            push_chips(out, &job.keywords, theme, w, 4);
        }
    }
}

fn section_projects(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(opts, "projects", &title_for("projects")));
    out.push(String::new());
    for (i, p) in r.projects.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        let year = p
            .start_date
            .as_deref()
            .map(Resume::humanize_date)
            .unwrap_or_default();
        let left = Style::new().fg(theme.heading).bold().paint(&p.name);
        let right = Style::new().fg(theme.muted).paint(&year);
        push_two_col(out, &left, &right, w, 2);

        let mut meta: Vec<String> = Vec::new();
        if !p.roles.is_empty() {
            meta.push(Style::new().fg(theme.accent2).paint(&p.roles.join(" & ")));
        }
        if opts.links {
            if !p.links.is_empty() {
                for link in &p.links {
                    let styled = Style::new().fg(theme.accent).underline().paint(&link.label);
                    meta.push(ansi::hyperlink(&link.url, &styled));
                }
            } else if let Some(u) = &p.url {
                let styled = Style::new().fg(theme.accent).underline().paint(u);
                meta.push(ansi::hyperlink(u, &styled));
            }
        }
        if !meta.is_empty() {
            out.push(format!("  {}", meta.join(&sep(theme))));
        }
        if let Some(d) = p.description.as_ref().or(p.summary.as_ref()) {
            push_para(out, d, theme, w, 4, Style::new().fg(theme.fg));
        }
        for h in &p.highlights {
            push_bullet(out, h, theme, w, 4);
        }
        if !p.keywords.is_empty() {
            push_chips(out, &p.keywords, theme, w, 4);
        }
    }
}

fn section_skills(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(opts, "skills", &title_for("skills")));
    out.push(String::new());
    let label_w = r
        .skills
        .iter()
        .map(|s| vwidth(&s.name))
        .max()
        .unwrap_or(0)
        .min(18)
        + 2;
    for s in &r.skills {
        let text = s.keywords.join(&Style::new().fg(theme.subtle).paint(" · "));
        push_labeled(out, &s.name, &text, label_w, w, 2, theme);
    }
}

fn section_education(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(opts, "education", &title_for("education")));
    out.push(String::new());
    for (i, e) in r.education.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        let dates = Resume::date_range(e.start_date.as_deref(), e.end_date.as_deref());
        let mut head = String::new();
        if let Some(st) = &e.study_type {
            head.push_str(&Style::new().fg(theme.heading).bold().paint(st));
        }
        if let Some(area) = &e.area {
            if !head.is_empty() {
                head.push_str(&sep(theme));
            }
            head.push_str(&Style::new().fg(theme.fg).paint(area));
        }
        if head.is_empty() {
            head = Style::new().fg(theme.heading).bold().paint(&e.institution);
        }
        let right = Style::new().fg(theme.muted).paint(&dates);
        push_two_col(out, &head, &right, w, 2);
        if !e.institution.is_empty() && (e.study_type.is_some() || e.area.is_some()) {
            out.push(format!(
                "  {}",
                Style::new().fg(theme.accent2).paint(&e.institution)
            ));
        }
    }
}

fn section_certifications(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(
        opts,
        "certifications",
        &title_for("certifications"),
    ));
    out.push(String::new());
    for c in &r.certificates {
        let tick = Style::new().fg(theme.good).paint("✓ ");
        let name = Style::new().fg(theme.fg).paint(&c.name);
        let mut line = format!("  {tick}{name}");
        if let Some(issuer) = &c.issuer {
            line.push_str(&sep(theme));
            line.push_str(&Style::new().fg(theme.muted).paint(issuer));
        }
        let _ = w;
        out.push(line);
    }
}

fn section_languages(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(opts, "languages", &title_for("languages")));
    out.push(String::new());
    let label_w = r
        .languages
        .iter()
        .map(|l| vwidth(&l.language))
        .max()
        .unwrap_or(0)
        .min(18)
        + 2;
    for l in &r.languages {
        let padded = format!("{:<width$}", l.language, width = label_w);
        let fluency = l.fluency.clone().unwrap_or_default();
        let _ = w;
        out.push(format!(
            "  {}{}",
            Style::new().fg(theme.accent2).bold().paint(&padded),
            Style::new().fg(theme.muted).paint(&fluency)
        ));
    }
}

fn section_interests(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    out.push(section_header(opts, "interests", &title_for("interests")));
    out.push(String::new());
    let chips: Vec<(String, usize)> = r.interests.iter().map(|i| chip(&i.name, theme)).collect();
    for line in wrap_chips(chips, w.saturating_sub(2)) {
        out.push(format!("  {line}"));
    }
}

fn section_awards(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    out.push(section_header(opts, "awards", &title_for("awards")));
    out.push(String::new());
    for a in &r.awards {
        let star = Style::new().fg(theme.good).paint("★ ");
        out.push(format!(
            "  {star}{}",
            Style::new().fg(theme.fg).paint(&a.title)
        ));
    }
}

fn section_extra(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);
    for (i, ex) in r.extra.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        let title = if ex.title.is_empty() {
            "Notes".to_string()
        } else {
            ex.title.clone()
        };
        out.push(section_header(opts, "extra", &title));
        out.push(String::new());
        render_extra_lines(ex, theme, w, out);
    }
}

fn render_extra_lines(ex: &ExtraSection, theme: &Theme, w: usize, out: &mut Vec<String>) {
    for line in &ex.lines {
        let t = line.trim();
        if let Some(item) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            push_bullet(out, &inline::to_plain(item), theme, w, 2);
        } else if !t.is_empty() {
            push_para(
                out,
                &inline::to_plain(t),
                theme,
                w,
                2,
                Style::new().fg(theme.fg),
            );
        }
    }
}

fn render_footer(r: &Resume, opts: &RenderOpts, out: &mut Vec<String>) {
    let theme = &opts.theme;
    let w = opts.width.max(40);

    if opts.links {
        let mut links: Vec<String> = Vec::new();
        if let Some(u) = &r.basics.url {
            links.push(u.clone());
        }
        for p in &r.basics.profiles {
            if let Some(u) = &p.url {
                if !links.contains(u) {
                    links.push(u.clone());
                }
            }
        }
        if !links.is_empty() {
            out.push(Style::new().fg(theme.rule).paint(&"─".repeat(w)));
            out.push(String::new());
            for u in &links {
                let styled = Style::new().fg(theme.accent).underline().paint(u);
                out.push(format!("  {}", ansi::hyperlink(u, &styled)));
            }
        }
    }

    if opts.qr {
        let target = r
            .basics
            .url
            .clone()
            .or_else(|| r.basics.profiles.iter().find_map(|p| p.url.clone()));
        if let Some(t) = target {
            let qr_lines = qr::render(&t);
            if !qr_lines.is_empty() {
                out.push(String::new());
                for line in qr_lines {
                    out.push(format!("  {}", Style::new().fg(theme.fg).paint(&line)));
                }
            }
        }
    }

    if let Some(f) = &r.meta.footer {
        out.push(String::new());
        push_para(out, f, theme, w, 2, Style::new().fg(theme.muted).italic());
    }

    out.push(String::new());
    let src = opts
        .source
        .as_ref()
        .map(|s| format!(" · {s}"))
        .unwrap_or_default();
    out.push(
        Style::new()
            .fg(theme.subtle)
            .paint(&format!("  rendered with gcv{src}")),
    );
}

fn push_chips(out: &mut Vec<String>, items: &[String], theme: &Theme, width: usize, indent: usize) {
    let chips: Vec<(String, usize)> = items.iter().map(|k| chip(k, theme)).collect();
    for line in wrap_chips(chips, width.saturating_sub(indent)) {
        out.push(format!("{}{}", " ".repeat(indent), line));
    }
}

fn chip(text: &str, theme: &Theme) -> (String, usize) {
    match ansi::mode() {
        ColorMode::TrueColor | ColorMode::Ansi256 => {
            let inner = format!(" {text} ");
            let w = vwidth(&inner);
            (
                Style::new()
                    .fg(theme.chip_fg)
                    .bg(theme.chip_bg)
                    .paint(&inner),
                w,
            )
        }
        ColorMode::Ansi16 => {
            let inner = format!("[{text}]");
            let w = vwidth(&inner);
            (Style::new().fg(theme.chip_fg).paint(&inner), w)
        }
        ColorMode::None => {
            let inner = format!("[{text}]");
            let w = vwidth(&inner);
            (inner, w)
        }
    }
}

fn wrap_chips(chips: Vec<(String, usize)>, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut curw = 0usize;
    for (s, cw) in chips {
        let add = if curw == 0 { cw } else { cw + 1 };
        if curw + add > width && curw > 0 {
            lines.push(std::mem::take(&mut cur));
            curw = 0;
        }
        if curw > 0 {
            cur.push(' ');
            curw += 1;
        }
        cur.push_str(&s);
        curw += cw;
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Resume, Work};
    use crate::theme::Theme;

    fn opts() -> RenderOpts {
        RenderOpts {
            theme: Theme::get("neon").unwrap(),
            width: 80,
            banner: false,
            avatar: false,
            qr: false,
            links: true,
            icons: true,
            density: Density::Comfortable,
            source: None,
        }
    }

    #[test]
    fn renders_core_sections_without_color() {
        let mut r = Resume::default();
        r.basics.name = Some("Ada Lovelace".into());
        r.basics.label = Some("Mathematician".into());
        r.basics.summary = Some("First programmer.".into());
        r.work.push(Work {
            name: "Analytical Engines".into(),
            position: Some("Mathematician".into()),
            highlights: vec!["Wrote the first algorithm".into()],
            ..Default::default()
        });
        let out = render(&r, &opts());
        assert!(out.contains("Ada Lovelace"));
        assert!(out.contains("Summary"));
        assert!(out.contains("Experience"));
        assert!(out.contains("Analytical Engines"));
        assert!(out.contains("Wrote the first algorithm"));
        assert!(!out.contains('\u{1b}'), "no ANSI when color is disabled");
    }

    #[test]
    fn date_range_humanizes() {
        assert_eq!(
            Resume::date_range(Some("2024-02"), None),
            "Feb 2024 – Present"
        );
        assert_eq!(
            Resume::date_range(Some("2019-10"), Some("2024-01")),
            "Oct 2019 – Jan 2024"
        );
        assert_eq!(Resume::date_range(Some("2025"), None), "2025 – Present");
    }
}
