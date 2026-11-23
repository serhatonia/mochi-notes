//! Text helpers shared by views, search and export.

use chrono::{DateTime, Datelike, Local, Timelike};

/// Returns a rough word count, ignoring the most common markdown syntax.
pub fn word_count(content: &str) -> usize {
    strip_markdown(content)
        .split_whitespace()
        .filter(|word| word.chars().any(|c| c.is_alphanumeric()))
        .count()
}

pub fn reading_minutes(words: usize) -> usize {
    if words == 0 {
        0
    } else {
        ((words as f32) / 200.0).ceil() as usize
    }
}

/// Converts markdown into a compact, readable preview.
pub fn strip_markdown(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut in_code_block = false;

    for raw_line in content.lines() {
        let mut line = raw_line.trim();

        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            output.push_str(line);
            output.push(' ');
            continue;
        }

        // Horizontal rules are not useful in a preview.
        if line == "---" || line == "***" || line == "___" {
            continue;
        }

        // Headings
        line = line.trim_start_matches('#').trim_start();

        // Blockquotes and list bullets
        while let Some(rest) = line.strip_prefix('>') {
            line = rest.trim_start();
        }

        for prefix in ["- [ ] ", "- [x] ", "- [X] ", "* ", "- ", "+ ", "• "] {
            if let Some(rest) = line.strip_prefix(prefix) {
                line = rest;
                break;
            }
        }

        // Ordered list markers like `1. `
        if let Some(dot) = line.find(". ") {
            if dot <= 2 && line[..dot].chars().all(|c| c.is_ascii_digit()) {
                line = &line[dot + 2..];
            }
        }

        let cleaned = strip_inline(line);
        if !cleaned.is_empty() {
            output.push_str(&cleaned);
            output.push(' ');
        }
    }

    collapse_whitespace(&output)
}

fn strip_inline(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Images: ![alt](url) -> alt
        if c == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            if let Some((text, next)) = parse_link(&chars, i + 1) {
                out.push_str(&text);
                i = next;
                continue;
            }
        }

        // Links: [text](url) -> text
        if c == '[' {
            if let Some((text, next)) = parse_link(&chars, i) {
                out.push_str(&text);
                i = next;
                continue;
            }
        }

        // Escaped characters
        if c == '\\' && i + 1 < chars.len() {
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if matches!(c, '*' | '_' | '`' | '~' | '#') {
            i += 1;
            continue;
        }

        out.push(c);
        i += 1;
    }

    out.trim().to_string()
}

/// Parses `[label](target)` starting at `start` (which must be `[`).
fn parse_link(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut depth = 0;
    let mut label_end = None;

    for (index, c) in chars.iter().enumerate().skip(start) {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    label_end = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }

    let label_end = label_end?;
    if chars.get(label_end + 1) != Some(&'(') {
        return None;
    }

    let mut depth = 0;
    let mut target_end = None;
    for (index, c) in chars.iter().enumerate().skip(label_end + 1) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    target_end = Some(index);
                    break;
                }
            }
            _ => {}
        }
    }

    let target_end = target_end?;
    let label: String = chars[start + 1..label_end].iter().collect();
    Some((strip_inline(&label), target_end + 1))
}

pub fn collapse_whitespace(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut last_was_space = false;

    for c in value.chars() {
        if c.is_whitespace() {
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

pub fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let mut result = String::with_capacity(max_chars.min(value.len()));
    let mut count = 0;

    for c in chars.by_ref() {
        if count >= max_chars {
            break;
        }
        result.push(c);
        count += 1;
    }

    if chars.next().is_some() {
        // Prefer cutting at a word boundary when it is close enough.
        if let Some(space) = result.rfind(' ') {
            if space > max_chars / 2 {
                result.truncate(space);
            }
        }
        while result.ends_with([' ', ',', '.', ';', ':']) {
            result.pop();
        }
        result.push('…');
    }

    result
}

pub fn relative_time(moment: DateTime<Local>) -> String {
    let now = Local::now();
    let delta = now.signed_duration_since(moment);

    if delta.num_seconds() < 0 {
        return "just now".into();
    }

    let seconds = delta.num_seconds();
    if seconds < 45 {
        return "just now".into();
    }

    let minutes = delta.num_minutes();
    if minutes < 60 {
        return format!("{minutes}m ago");
    }

    let hours = delta.num_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }

    let days = delta.num_days();
    if days == 1 {
        return "yesterday".into();
    }
    if days < 7 {
        return format!("{days}d ago");
    }
    if days < 365 && moment.year() == now.year() {
        return moment.format("%b %-d").to_string();
    }

    moment.format("%b %-d, %Y").to_string()
}

pub fn format_date(moment: DateTime<Local>) -> String {
    moment.format("%B %-d, %Y").to_string()
}

pub fn format_short_date(moment: DateTime<Local>) -> String {
    moment.format("%b %-d, %H:%M").to_string()
}

pub fn greeting() -> &'static str {
    match Local::now().hour() {
        0..=4 => "Good night",
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}

pub fn format_number(value: usize) -> String {
    let digits = value.to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, c) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result
}

pub fn plural(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// Turns a note title into a safe file name.
pub fn sanitize_filename(title: &str) -> String {
    let mut result: String = title
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();

    result = collapse_whitespace(&result);
    result = result.trim_matches(['.', ' ']).to_string();

    if result.is_empty() {
        "Untitled note".into()
    } else {
        truncate_chars(&result, 80)
    }
}

/// `12` -> `"12"`, `1250` -> `"1.3k"`
pub fn compact_number(value: usize) -> String {
    if value < 1000 {
        value.to_string()
    } else if value < 1_000_000 {
        format!("{:.1}k", value as f32 / 1000.0)
    } else {
        format!("{:.1}m", value as f32 / 1_000_000.0)
    }
}

pub fn human_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let bytes = bytes as f64;

    if bytes < KB {
        format!("{} B", bytes as u64)
    } else if bytes < KB * KB {
        format!("{:.0} KB", bytes / KB)
    } else {
        format!("{:.1} MB", bytes / (KB * KB))
    }
}
