use colored::*;
use serde_json::Value;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::Instant;

mod curl_parser;
mod ws_client;
use curl_parser::CurlCommand;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let stdin_has_data = !atty::is(atty::Stream::Stdin);

    // ── Modo 1: curl -si ... | curlp  ──────────────────────────────────
    if stdin_has_data {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).expect("Error leyendo stdin");
        println!();
        display_response(&input, 0);
        return;
    }

    // ── Sin argumentos: mostrar ayuda ──────────────────────────────────
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "--help" | "-h" => { print_help(); return; }
        "--version" | "-V" => {
            println!("{} {}", "curlp".cyan().bold(), env!("CARGO_PKG_VERSION").white());
            return;
        }
        _ => {}
    }

    // ── Modo WebSocket: URLs directas ──────────────────────────────────
    let command_str = if args.len() == 2 {
        args[1].clone()
    } else {
        args[1..].join(" ")
    };

    let trimmed = command_str.trim();
    
    // URL directa de WebSocket
    if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(ws_client::connect_ws(trimmed));
        return;
    }

    // Comandos tipo wscat -c wss://...
    if let Some(url) = extract_ws_url(trimmed) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(ws_client::connect_ws(&url));
        return;
    }

    // ── Modo 2: curlp 'curl ...'  o  curlp curl ... ───────────────────
    execute_curl_and_display(&command_str);
}

// ─────────────────────────────────────────────────────────────────────────────
// Execution
// ─────────────────────────────────────────────────────────────────────────────

fn execute_curl_and_display(command_str: &str) {
    let parsed = CurlCommand::parse(command_str);

    println!();
    let method_label = parsed.method.as_deref().unwrap_or(if parsed.data.is_some() { "POST" } else { "GET" });
    println!(
        "{} {} {}",
        method_label.cyan().bold(),
        "→".dimmed(),
        parsed.url.white().bold()
    );
    println!("{}", "─".repeat(64).dimmed());

    let curl_args = parsed.to_args_with_headers();
    let start = Instant::now();

    let output = Command::new("curl")
        .args(&curl_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    let elapsed = start.elapsed().as_millis();

    match output {
        Err(e) => {
            eprintln!("{} curl no encontrado o error: {}", "✗".red().bold(), e);
            std::process::exit(1);
        }
        Ok(out) => {
            // Mostrar stderr filtrado (errores reales, no progreso)
            if !out.stderr.is_empty() {
                let err = String::from_utf8_lossy(&out.stderr);
                let real_errors: Vec<&str> = err.lines()
                    .filter(|l| {
                        let t = l.trim();
                        !t.is_empty() && !t.contains("% Total") && !t.contains("Dload") && !t.starts_with(' ')
                    })
                    .collect();
                if !real_errors.is_empty() {
                    eprintln!("{}", real_errors.join("\n").yellow());
                }
            }

            if out.stdout.is_empty() {
                eprintln!("{} No hubo respuesta. Verifica la URL.", "✗".red().bold());
                return;
            }

            let raw = String::from_utf8_lossy(&out.stdout).to_string();
            display_response(&raw, elapsed);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Response parser & display
// ─────────────────────────────────────────────────────────────────────────────

fn display_response(raw: &str, elapsed_ms: u128) {
    // curl -i puede devolver múltiples bloques header (por redirects).
    // Dividimos en todos los bloques y tomamos el último par headers/body.
    let separator = if raw.contains("\r\n\r\n") { "\r\n\r\n" } else { "\n\n" };

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
    let real_headers: Vec<&str> = header_lines.iter()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .copied()
        .collect();

    if !real_headers.is_empty() {
        println!("{}", "  HEADERS".dimmed());
        println!("{}", "  ──────────────────────────────────────────────────────".dimmed());
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
        println!("{}", "  ──────────────────────────────────────────────────────".dimmed());
        display_body(body_trimmed);
    }

    println!();
}

fn parse_status_code(line: &str) -> u16 {
    line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn display_status(line: &str, code: u16, elapsed_ms: u128) {
    let (icon, label) = match code {
        200..=299 => ("✓".green().bold(), format!(" {} ", line).on_green().black().bold()),
        300..=399 => ("↝".yellow().bold(), format!(" {} ", line).on_yellow().black().bold()),
        400..=499 => ("✗".red().bold(),   format!(" {} ", line).on_red().white().bold()),
        500..=599 => ("✗".bright_red().bold(), format!(" {} ", line).on_bright_red().white().bold()),
        _         => ("?".white().bold(), format!(" {} ", line).on_white().black().bold()),
    };

    if elapsed_ms > 0 {
        println!("  {} {}  {}", icon, label, format!("{} ms", elapsed_ms).dimmed());
    } else {
        println!("  {} {}", icon, label);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Body renderers
// ─────────────────────────────────────────────────────────────────────────────

fn display_body(body: &str) {
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
    let pad   = "  ".repeat(depth);
    let pad_c = "  ".repeat(depth.saturating_sub(1));
    let comma = if is_last { "" } else { "," };

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                print!("{}{}{}", "{}".dimmed(), comma.dimmed(), "");
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
        Value::String(s)  => print!("{}{}", format!("\"{}\"", s).green(),  comma.dimmed()),
        Value::Number(n)  => print!("{}{}", n.to_string().yellow(),         comma.dimmed()),
        Value::Bool(b)    => print!("{}{}", b.to_string().magenta(),        comma.dimmed()),
        Value::Null       => print!("{}{}", "null".red().dimmed(),          comma.dimmed()),
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
                if ch == '>' { break; }
                tag.push(ch);
            }
            let closing  = tag.starts_with('/');
            let self_cls = tag.ends_with('/');
            let decl     = tag.starts_with('?') || tag.starts_with('!');
            if closing { indent = indent.saturating_sub(1); }
            let pad = "  ".repeat(indent);
            if decl {
                println!("{}{}", pad, format!("<{}>", tag).dimmed());
            } else if closing {
                println!("{}{}", pad, format!("<{}>", tag).cyan());
            } else {
                println!("{}{}", pad, format!("<{}>", tag).cyan().bold());
            }
            if !closing && !self_cls && !decl { indent += 1; }
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

// ─────────────────────────────────────────────────────────────────────────────
// Help
// ─────────────────────────────────────────────────────────────────────────────

fn print_help() {
    let v = env!("CARGO_PKG_VERSION");
    println!();
    println!("  {} {}  —  HTTP pretty-printer para tu terminal",
        "curlp".cyan().bold(), format!("v{}", v).dimmed());
    println!("  {}", "─".repeat(54).dimmed());
    println!();
    println!("  {}", "MODOS DE USO".white().bold());
    println!();
    println!("  {}  {}", "1. Argumento".yellow().bold(),
        "curlp 'curl [opciones] <url>'".white());
    println!("     {}", "curlp ejecuta el comando y prettifica la respuesta".dimmed());
    println!();
    println!("  {}     {}", "2. Pipe".yellow().bold(),
        "curl -si <url> | curlp".white());
    println!("     {}", "pasa la salida raw de curl directamente a curlp".dimmed());
    println!("     {}", "(-s silencia progreso, -i incluye headers)".dimmed());
    println!();
    println!("  {}", "EJEMPLOS".white().bold());
    println!();
    println!("  {}", "curlp 'curl https://jsonplaceholder.typicode.com/todos/1'".dimmed());
    println!();
    println!("  {}", "curlp 'curl -X POST https://httpbin.org/post \\".dimmed());
    println!("  {}", "  -H \"Content-Type: application/json\" \\".dimmed());
    println!("  {}", "  -d \\'{{\"usuario\":\"juan\",\"rol\":\"admin\"}}\\''".dimmed());
    println!();
    println!("  {}", "curl -si https://httpbin.org/get | curlp".dimmed());
    println!();
    println!("  {}", "curlp 'curl -L https://httpbin.org/redirect/1'".dimmed());
    println!();
    println!("  {}", "SOPORTA".white().bold());
    println!();
    println!("  {}  JSON con colores y sangría", "◆".cyan());
    println!("  {}  XML formateado", "◆".cyan());
    println!("  {}  Headers coloreados", "◆".cyan());
    println!("  {}  Status con color (2xx verde · 3xx amarillo · 4xx/5xx rojo)", "◆".cyan());
    println!("  {}  Tiempo de respuesta", "◆".cyan());
    println!("  {}  Redirects automáticos (-L)", "◆".cyan());
    println!("  {}  Todos los flags de curl (-k, -u, --proxy, etc.)", "◆".cyan());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_code() {
        assert_eq!(parse_status_code("HTTP/1.1 200 OK"), 200);
        assert_eq!(parse_status_code("HTTP/2 404 Not Found"), 404);
        assert_eq!(parse_status_code("HTTP/1.1 500 Internal Server Error"), 500);
        assert_eq!(parse_status_code(""), 0);
        assert_eq!(parse_status_code("invalid"), 0);
    }

    #[test]
    fn test_display_response_json() {
        let json_response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"id\": 1, \"name\": \"test\"}";
        
        // This should not panic and should handle JSON properly
        display_response(json_response, 123);
    }

    #[test]
    fn test_display_response_xml() {
        let xml_response = "HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\n\r\n<?xml version=\"1.0\"?><root><item>test</item></root>";
        
        // This should not panic and should handle XML properly
        display_response(xml_response, 456);
    }

    #[test]
    fn test_display_response_plain_text() {
        let text_response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
        
        // This should not panic and should handle plain text properly
        display_response(text_response, 789);
    }

    #[test]
    fn test_display_response_no_headers() {
        let body_only = "{\"message\": \"no headers\"}";
        
        // This should handle body-only responses
        display_response(body_only, 0);
    }

    #[test]
    fn test_display_response_empty_body() {
        let empty_response = "HTTP/1.1 204 No Content\r\n\r\n";
        
        // This should handle empty body responses
        display_response(empty_response, 100);
    }

    #[test]
    fn test_display_response_with_redirects() {
        let redirect_response = "HTTP/1.1 301 Moved Permanently\r\nLocation: /new-url\r\n\r\nHTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"success\": true}";
        
        // This should handle responses with redirects
        display_response(redirect_response, 250);
    }
}

// Extrae URL WebSocket de comandos tipo wscat
fn extract_ws_url(command: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    
    for (i, token) in tokens.iter().enumerate() {
        match *token {
            "-c" | "--connect" => {
                if i + 1 < tokens.len() {
                    let next = tokens[i + 1];
                    if next.starts_with("ws://") || next.starts_with("wss://") {
                        return Some(next.to_string());
                    }
                }
            }
            _ => {
                // Buscar URLs WebSocket directamente
                if token.starts_with("ws://") || token.starts_with("wss://") {
                    return Some(token.to_string());
                }
            }
        }
    }
    
    None
}
