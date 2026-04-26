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
            let current = env!("CARGO_PKG_VERSION");
            println!("{} {}", "curlp".cyan().bold(), current.white());
            
            // Check for updates silently
            if let Ok(latest) = check_latest_version() {
                if latest != current {
                    println!("  {} New version available: {} → {}", "⚠️".yellow(), current, latest.green());
                    println!("  {} Update with: {}", "→".dimmed(), "curlp --update".cyan());
                }
            }
            return;
        }
        "--doctor" | "--check" => { print_doctor(); return; }
        "--update" => { update_curlp(); return; }
        flag if flag.starts_with("--") => {
            eprintln!("{} {}: Unknown option '{}'", "❌", "Error".red().bold(), flag);
            eprintln!("{} Use 'curlp --help' for available options", "➡️".dimmed());
            std::process::exit(1);
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

    // ── Check for updates (silent notification) ───────────────────────
    check_for_update_notification();

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
    println!("  {} {}  —  HTTP pretty-printer for your terminal",
        "curlp".cyan().bold(), format!("v{}", v).dimmed());
    println!("  {}", "─".repeat(54).dimmed());
    println!();
    println!("  {}", "USAGE MODES".white().bold());
    println!();
    println!("  {}  {}", "curlp".cyan(), "[OPCIONES]".white());
    println!("  {}  {}", "curlp".cyan(), "'curl [opciones] <url>'".white());
    println!("  {}  {}", "curlp".cyan(), "<websocket-url>".white());
    println!("  {}  {}", "curl".cyan(), "-si <url> | curlp".white());
    println!();
    println!("  {}", "OPTIONS".white().bold());
    println!();
    println!("  {}  {}", "-h, --help".yellow(), "Show this help".white());
    println!("  {}  {}", "-V, --version".yellow(), "Show version".white());
    println!("  {}  {}", "--doctor".yellow(), "Diagnose installation and PATH".white());
    println!("  {}  {}", "--update".yellow(), "Update to latest version".white());
    println!();
    println!("  {}", "HTTP MODE".white().bold());
    println!();
    println!("  {}  {}", "1. Argument mode".yellow().bold(),
        "curlp executes curl and prettifies the response".white());
    println!("     {}", "pass curl raw output directly to curlp".dimmed());
    println!();
    println!("  {}  {}", "2. Pipe mode".yellow().bold(),
        "curl -si <url> | curlp".white());
    println!("     {}", "(-s silences progress, -i includes headers)".dimmed());
    println!();
    println!("  {}", "WEBSOCKET MODE".white().bold());
    println!();
    println!("  {}", "Direct connection  curlp wss://echo.websocket.org".white());
    println!("  {}  {}", "wscat command".white(), "curlp 'wscat -c wss://echo.websocket.org'".white());
    println!();
    println!("  {}", "HTTP EXAMPLES".white().bold());
    println!();
    println!("  {}", "# Simple GET".dimmed());
    println!("  {}", "curlp 'curl https://jsonplaceholder.typicode.com/todos/1'".dimmed());
    println!();
    println!("  {}", "# POST with JSON".dimmed());
    println!("  {}", "curlp 'curl -X POST https://httpbin.org/post \\".dimmed());
    println!("  {}", "  -H \"Content-Type: application/json\" \\".dimmed());
    println!("  {}", "  -d \'{{\"user\":\"juan\",\"role\":\"admin\"}}\''".dimmed());
    println!();
    println!("  {}", "# With authentication".dimmed());
    println!("  {}", "curlp 'curl -u user:password https://api.example.com/private'".dimmed());
    println!();
    println!("  {}", "# Custom headers".dimmed());
    println!("  {}", "curlp 'curl -H \"Authorization: Bearer <token>\" https://api.example.com/me'".dimmed());
    println!();
    println!("  {}", "# Follow redirects".dimmed());
    println!("  {}", "curlp 'curl -L https://httpbin.org/redirect/1'".dimmed());
    println!();
    println!("  {}", "HTTP FEATURES".white().bold());
    println!();
    println!("  {}  JSON with colors and indentation", "◆".cyan());
    println!("  {}  XML formatted", "◆".cyan());
    println!("  {}  Colored headers", "◆".cyan());
    println!("  {}  Status colors (2xx green · 3xx yellow · 4xx/5xx red)", "◆".cyan());
    println!("  {}  Response time", "◆".cyan());
    println!("  {}  Automatic redirects (-L)", "◆".cyan());
    println!("  {}  All curl flags (-k, -u, --proxy, etc.)", "◆".cyan());
    println!();
    println!("  {}", "WEBSOCKET FEATURES".white().bold());
    println!();
    println!("  {}  Automatic JSON prettifier", "◆".cyan());
    println!("  {}  Colored prefixes: ← incoming, → outgoing", "◆".cyan());
    println!("  {}  Interactive mode with stdin/stdout", "◆".cyan());
    println!("  {}  /quit command to close connection", "◆".cyan());
    println!("  {}  Connection status on startup", "◆".cyan());
    println!();
    println!("  {}", "INSTALLATION".white().bold());
    println!();
    println!("  {}  {}", "Universal script:".dimmed(), "curl -sSL https://raw.githubusercontent.com/tinconomad/curl-pretty/main/install.sh | bash".white());
    println!("  {}  {}", "Manual:".dimmed(), "https://github.com/tinconomad/curl-pretty/releases".white());
    println!();
}

fn print_doctor() {
    use std::env;
    
    println!();
    println!("  {}  —  Diagnóstico de instalación", "curlp".cyan().bold());
    println!("  {}", "─".repeat(54).dimmed());
    println!();
    
    // Verificar ubicación del binario
    let current_exe = env::current_exe().ok();
    let install_paths = vec![
        format!("{}/.local/bin/curlp", env::var("HOME").unwrap_or_default()),
        "/usr/local/bin/curlp".to_string(),
        "/usr/bin/curlp".to_string(),
    ];
    
    println!("  {}", "UBICACIÓN DEL BINARIO".white().bold());
    println!();
    
    if let Some(exe_path) = current_exe {
        println!("  {}  {}", "✅".green(), format!("Ejecutando desde: {}", exe_path.display()).white());
    }
    
    let mut found_in_path = false;
    for path in &install_paths {
        if std::path::Path::new(path).exists() {
            println!("  {}  {}", "✅".green(), format!("Encontrado en: {}", path).white());
            found_in_path = true;
        }
    }
    
    if !found_in_path {
        println!("  {}  {}", "❌".red(), "No encontrado en ubicaciones estándar".white());
    }
    println!();
    
    // Verificar PATH
    println!("  {}", "CONFIGURACIÓN DE PATH".white().bold());
    println!();
    
    let path = env::var("PATH").unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();
    let local_bin = format!("{}/.local/bin", home);
    
    if path.contains(&local_bin) {
        println!("  {}  {}", "✅".green(), format!("{} está en PATH", local_bin).white());
    } else {
        println!("  {}  {}", "⚠️".yellow(), format!("{} NO está en PATH", local_bin).white());
        println!();
        println!("  {}", "SOLUCIÓN:".yellow().bold());
        println!("  {}", "Agrega esto a tu ~/.bashrc o ~/.zshrc:".white());
        println!("  {}", format!("export PATH=\"$HOME/.local/bin:$PATH\"").cyan());
        println!();
        println!("  {}", "Luego recarga la configuración:".white());
        println!("  {}", "source ~/.zshrc  # o source ~/.bashrc".cyan());
    }
    println!();
    
    // Verificar curl
    println!("  {}", "DEPENDENCIAS".white().bold());
    println!();
    
    match Command::new("curl").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let first_line = version.lines().next().unwrap_or("curl found");
            println!("  {}  {}", "✅".green(), format!("curl: {}", first_line).white());
        }
        _ => println!("  {}  {}", "❌".red(), "curl: No encontrado (requerido para HTTP mode)".white()),
    }
    println!();
    
    // Test de conectividad
    println!("  {}", "PRUEBA DE CONECTIVIDAD".white().bold());
    println!();
    
    match Command::new("curl").args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", "https://httpbin.org/get"]).output() {
        Ok(output) if output.status.success() => {
            let code = String::from_utf8_lossy(&output.stdout);
            if code.trim() == "200" {
                println!("  {}  {}", "✅".green(), "Conexión a internet: OK".white());
            } else {
                println!("  {}  {}", "⚠️".yellow(), format!("HTTP status: {}", code.trim()).white());
            }
        }
        _ => println!("  {}  {}", "❌".red(), "No se pudo conectar a internet".white()),
    }
    println!();
    
    // Resumen
    println!("  {}", "RESUMEN".white().bold());
    println!();
    if found_in_path && path.contains(&local_bin) {
        println!("  {}  {}", "✅".green(), "Todo está correctamente configurado!".white());
        println!();
        println!("  {}", "Prueba con:".dimmed());
        println!("  {}", "curlp 'curl https://httpbin.org/get'".cyan());
    } else {
        println!("  {}  {}", "⚠️".yellow(), "Se encontraron problemas. Revisa las soluciones arriba.".white());
    }
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

// ─────────────────────────────────────────────────────────────────────────────
// Version checking and update
// ─────────────────────────────────────────────────────────────────────────────

fn check_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let response = ureq::get("https://api.github.com/repos/tinconomad/curl-pretty/releases/latest")
        .call()?;
    
    let text = response.into_string()?;
    if let Some(tag_line) = text.lines().find(|line| line.contains("\"tag_name\"")) {
        if let Some(tag) = tag_line.split(':').nth(1) {
            let tag = tag.trim().trim_matches('"');
            return Ok(tag.to_string());
        }
    }
    
    Err("Failed to parse version".into())
}

fn update_curlp() {
    println!("{} Updating curlp...", "🔄".cyan());
    
    match std::process::Command::new("sh")
        .arg("-c")
        .arg("curl -sSL https://raw.githubusercontent.com/tinconomad/curl-pretty/main/install.sh | bash")
        .status()
    {
        Ok(status) if status.success() => {
            println!("{} Update successful!", "✅".green());
            println!("{} Please restart your terminal or run: {}", "→".dimmed(), "source ~/.bashrc".cyan());
        }
        Ok(_) => {
            println!("{} Update failed. Please try manually:", "❌".red());
            println!("  {}", "curl -sSL https://raw.githubusercontent.com/tinconomad/curl-pretty/main/install.sh | bash".cyan());
        }
        Err(e) => {
            println!("{} Failed to run update: {}", "❌".red(), e);
        }
    }
}

fn check_for_update_notification() {
    let current = env!("CARGO_PKG_VERSION");
    if let Ok(latest) = check_latest_version() {
        if latest != current {
            eprintln!("{} New version available: {} → {} (run {} to update)", 
                "⚠️".yellow(), current, latest.green(), "curlp --update".cyan());
            eprintln!(); // Add spacing
        }
    }
}
