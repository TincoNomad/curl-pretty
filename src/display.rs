use colored::*;
use serde_json::Value;

pub fn display_response(raw: &str, elapsed_ms: u128) {
    // curl -i puede devolver múltiples bloques header (por redirects).
    // Dividimos en todos los bloques y tomamos el último par headers/body.
    let separator = if raw.contains("\r\n\r\n") {
        "\r\n\r\n"
    } else {
        "\n\n"
    };

    // Puede haber múltiples bloques si hay redirects; quedarnos con el último
    let blocks: Vec<&str> = raw.split(separator).collect();

    // El último bloque es el body; el penúltimo son los headers finales
    let (headers_raw, body) = if blocks.len() >= 2 {
        // Buscar el último bloque de headers (el que tiene un status HTTP)
        let mut last_header_idx = 0;
        for (i, block) in blocks.iter().enumerate() {
            if block.trim_start().starts_with("HTTP/") {
                last_header_idx = i;
            }
        }
        let headers = blocks[last_header_idx];
        let body = blocks[last_header_idx + 1..].join(separator);
        (headers, body)
    } else {
        ("", raw.to_string())
    };

    let header_lines: Vec<&str> = headers_raw.lines().collect();
    let status_line = header_lines.first().copied().unwrap_or("").trim();
    let status_code = parse_status_code(status_line);

    // ── Status ──────────────────────────────────────────────────────────
    println!();
    display_status(status_line, status_code, elapsed_ms);
    println!();

    // ── Headers ─────────────────────────────────────────────────────────
    let real_headers: Vec<&str> = header_lines
        .iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .copied()
        .collect();

    if !real_headers.is_empty() {
        println!("{}", "  HEADERS".dimmed());
        println!(
            "{}",
            "  ──────────────────────────────────────────────────────".dimmed()
        );
        for line in &real_headers {
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let val = line[pos + 1..].trim();
                println!("  {:30} {}", key.cyan(), val.white());
            }
        }
        println!();
    }

    // ── Body ────────────────────────────────────────────────────────────
    let body_trimmed = body.trim();
    if body_trimmed.is_empty() {
        println!("  {}", "(respuesta sin cuerpo)".dimmed().italic());
    } else {
        println!("{}", "  BODY".dimmed());
        println!(
            "{}",
            "  ──────────────────────────────────────────────────────".dimmed()
        );
        display_body(body_trimmed);
    }

    println!();
}

pub fn parse_status_code(line: &str) -> u16 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn display_status(line: &str, code: u16, elapsed_ms: u128) {
    let (icon, label) = match code {
        200..=299 => (
            "✓".green().bold(),
            format!(" {} ", line).on_green().black().bold(),
        ),
        300..=399 => (
            "↝".yellow().bold(),
            format!(" {} ", line).on_yellow().black().bold(),
        ),
        400..=499 => (
            "✗".red().bold(),
            format!(" {} ", line).on_red().white().bold(),
        ),
        500..=599 => (
            "✗".bright_red().bold(),
            format!(" {} ", line).on_bright_red().white().bold(),
        ),
        _ => (
            "?".white().bold(),
            format!(" {} ", line).on_white().black().bold(),
        ),
    };

    if elapsed_ms > 0 {
        println!(
            "  {} {}  {}",
            icon,
            label,
            format!("{} ms", elapsed_ms).dimmed()
        );
    } else {
        println!("  {} {}", icon, label);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Body renderers
// ─────────────────────────────────────────────────────────────────────────────

pub fn display_body(body: &str) {
    match serde_json::from_str::<Value>(body) {
        Ok(json) => {
            print_json(&json, 1, true);
            println!();
        }
        Err(_) => {
            if body.trim_start().starts_with('<') {
                print_xml(body);
            } else {
                // Plain text — wrap at 80 cols
                for line in body.lines() {
                    println!("  {}", line.white());
                }
            }
        }
    }
}

fn print_json(value: &Value, depth: usize, is_last: bool) {
    let pad = "  ".repeat(depth);
    let pad_c = "  ".repeat(depth.saturating_sub(1));
    let comma = if is_last { "" } else { "," };

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                print!("{}{}", "{}".dimmed(), comma.dimmed());
                return;
            }
            println!("{}", "{".dimmed());
            let entries: Vec<_> = map.iter().collect();
            let len = entries.len();
            for (i, (k, v)) in entries.iter().enumerate() {
                let last = i == len - 1;
                print!("{}{} ", pad, format!("\"{}\":", k).cyan());
                print_json(v, depth + 1, last);
            }
            print!("{}}}{}", pad_c, comma.dimmed());
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                print!("{}{}", "[]".dimmed(), comma.dimmed());
                return;
            }
            println!("{}", "[".dimmed());
            let len = arr.len();
            for (i, item) in arr.iter().enumerate() {
                let last = i == len - 1;
                print!("{}", pad);
                print_json(item, depth + 1, last);
            }
            print!("{}]{}", pad_c, comma.dimmed());
        }
        Value::String(s) => print!("{}{}", format!("\"{}\"", s).green(), comma.dimmed()),
        Value::Number(n) => print!("{}{}", n.to_string().yellow(), comma.dimmed()),
        Value::Bool(b) => print!("{}{}", b.to_string().magenta(), comma.dimmed()),
        Value::Null => print!("{}{}", "null".red().dimmed(), comma.dimmed()),
    }

    // Always end with newline after top-level value
    println!();
}

fn print_xml(xml: &str) {
    let mut indent = 1usize;
    let mut chars = xml.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '<' {
            chars.next();
            let mut tag = String::new();
            for ch in chars.by_ref() {
                if ch == '>' {
                    break;
                }
                tag.push(ch);
            }
            let closing = tag.starts_with('/');
            let self_cls = tag.ends_with('/');
            let decl = tag.starts_with('?') || tag.starts_with('!');
            if closing {
                indent = indent.saturating_sub(1);
            }
            let pad = "  ".repeat(indent);
            if decl {
                println!("{}{}", pad, format!("<{}>", tag).dimmed());
            } else if closing {
                println!("{}{}", pad, format!("<{}>", tag).cyan());
            } else {
                println!("{}{}", pad, format!("<{}>", tag).cyan().bold());
            }
            if !closing && !self_cls && !decl {
                indent += 1;
            }
        } else {
            let mut text = String::new();
            while chars.peek().map(|&c| c != '<').unwrap_or(false) {
                text.push(chars.next().unwrap());
            }
            let t = text.trim();
            if !t.is_empty() {
                println!("{}{}", "  ".repeat(indent), t.green());
            }
        }
    }
}
