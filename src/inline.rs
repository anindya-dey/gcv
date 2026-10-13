//! Tiny inline-Markdown tokenizer used for both plain-text extraction and
//! styled rendering. Supports **bold**, _italic_, `code`, and [links](url).

#[derive(Debug, Clone, Default)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub link: Option<String>,
}

pub fn parse(s: &str) -> Vec<Span> {
    let chars: Vec<char> = s.chars().collect();
    let mut spans: Vec<Span> = Vec::new();
    let mut buf = String::new();
    let mut i = 0;

    let flush = |buf: &mut String, spans: &mut Vec<Span>| {
        if !buf.is_empty() {
            spans.push(Span {
                text: std::mem::take(buf),
                ..Default::default()
            });
        }
    };

    while i < chars.len() {
        let c = chars[i];
        // code
        if c == '`' {
            if let Some(end) = find_char(&chars, i + 1, '`') {
                flush(&mut buf, &mut spans);
                spans.push(Span {
                    text: chars[i + 1..end].iter().collect(),
                    code: true,
                    ..Default::default()
                });
                i = end + 1;
                continue;
            }
        }
        // link [text](url)
        if c == '[' {
            if let Some(close) = find_char(&chars, i + 1, ']') {
                if close + 1 < chars.len() && chars[close + 1] == '(' {
                    if let Some(paren) = find_char(&chars, close + 2, ')') {
                        flush(&mut buf, &mut spans);
                        let text: String = chars[i + 1..close].iter().collect();
                        let url: String = chars[close + 2..paren].iter().collect();
                        spans.push(Span {
                            text,
                            link: Some(url),
                            ..Default::default()
                        });
                        i = paren + 1;
                        continue;
                    }
                }
            }
        }
        // bold ** or __
        if (c == '*' || c == '_') && i + 1 < chars.len() && chars[i + 1] == c {
            let marker = c;
            if let Some(end) = find_double(&chars, i + 2, marker) {
                flush(&mut buf, &mut spans);
                let inner: String = chars[i + 2..end].iter().collect();
                for mut sp in parse(&inner) {
                    sp.bold = true;
                    spans.push(sp);
                }
                i = end + 2;
                continue;
            }
        }
        // italic * or _
        if c == '*' || c == '_' {
            let boundary_ok = c == '*' || i == 0 || !chars[i - 1].is_alphanumeric();
            if boundary_ok {
                if let Some(end) = find_char(&chars, i + 1, c) {
                    let after_ok = end + 1 >= chars.len() || !chars[end + 1].is_alphanumeric();
                    if after_ok {
                        flush(&mut buf, &mut spans);
                        let inner: String = chars[i + 1..end].iter().collect();
                        for mut sp in parse(&inner) {
                            sp.italic = true;
                            spans.push(sp);
                        }
                        i = end + 1;
                        continue;
                    }
                }
            }
        }
        buf.push(c);
        i += 1;
    }
    flush(&mut buf, &mut spans);
    spans
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&j| chars[j] == target)
}

fn find_double(chars: &[char], from: usize, target: char) -> Option<usize> {
    let mut j = from;
    while j + 1 < chars.len() {
        if chars[j] == target && chars[j + 1] == target {
            return Some(j);
        }
        j += 1;
    }
    None
}

pub fn to_plain(s: &str) -> String {
    parse(s)
        .into_iter()
        .map(|sp| sp.text)
        .collect::<Vec<_>>()
        .join("")
}

pub fn extract_links(s: &str) -> Vec<(String, String)> {
    parse(s)
        .into_iter()
        .filter_map(|sp| sp.link.map(|u| (sp.text, u)))
        .collect()
}

pub fn first_bold(s: &str) -> Option<String> {
    parse(s).into_iter().find(|sp| sp.bold).map(|sp| sp.text)
}

pub fn first_italic(s: &str) -> Option<String> {
    parse(s).into_iter().find(|sp| sp.italic).map(|sp| sp.text)
}
