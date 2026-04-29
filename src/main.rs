use colored::*;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::Instant;

mod curl_parser;
mod display;
mod help;
mod version;
mod ws;
mod ws_client;

use curl_parser::CurlCommand;
use display::display_response;
use help::{print_doctor, print_help};
use version::{check_for_update_notification, check_latest_version, update_pcurl};
use ws::extract_ws_url;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let stdin_has_data = !atty::is(atty::Stream::Stdin);

    // ── Modo 1: curl -si ... | pcurl  ──────────────────────────────────
    if stdin_has_data {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .expect("Error leyendo stdin");
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
        "--help" | "-h" => {
            print_help();
            return;
        }
        "--version" | "-V" => {
            let current = env!("CARGO_PKG_VERSION");
            println!("{} {}", "pcurl".cyan().bold(), current.white());

            // Check for updates silently
            if let Ok(latest) = check_latest_version() {
                if latest != current {
                    println!(
                        "  {} New version available: {} → {}",
                        "⚠️".yellow(),
                        current,
                        latest.green()
                    );
                    println!(
                        "  {} Update with: {}",
                        "→".dimmed(),
                        "pcurl --update".cyan()
                    );
                }
            }
            return;
        }
        "--doctor" | "--check" => {
            print_doctor();
            return;
        }
        "--update" => {
            update_pcurl();
            return;
        }
        flag if flag.starts_with("--") => {
            eprintln!(
                "{} {}: Unknown option '{}'",
                "❌".red().bold(),
                "Error".red().bold(),
                flag
            );
            eprintln!("{} Use 'pcurl --help' for available options", "➡️".dimmed());
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

    // ── Modo 2: pcurl 'curl ...'  o  pcurl curl ... ───────────────────
    execute_curl_and_display(&command_str);
}

// ─────────────────────────────────────────────────────────────────────────────
// Execution
// ─────────────────────────────────────────────────────────────────────────────

fn execute_curl_and_display(command_str: &str) {
    let parsed = CurlCommand::parse(command_str);

    println!();
    let method_label = parsed
        .method
        .as_deref()
        .unwrap_or(if parsed.data.is_some() { "POST" } else { "GET" });
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
                let real_errors: Vec<&str> = err
                    .lines()
                    .filter(|l| {
                        let t = l.trim();
                        !t.is_empty()
                            && !t.contains("% Total")
                            && !t.contains("Dload")
                            && !t.starts_with(' ')
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

#[cfg(test)]
mod tests {
    use crate::display::parse_status_code;

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
        crate::display::display_response(json_response, 123);
    }

    #[test]
    fn test_display_response_xml() {
        let xml_response = "HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\n\r\n<?xml version=\"1.0\"?><root><item>test</item></root>";
        crate::display::display_response(xml_response, 456);
    }

    #[test]
    fn test_display_response_plain_text() {
        let text_response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
        crate::display::display_response(text_response, 789);
    }

    #[test]
    fn test_display_response_no_headers() {
        let body_only = "{\"message\": \"no headers\"}";
        crate::display::display_response(body_only, 0);
    }

    #[test]
    fn test_display_response_empty_body() {
        let empty_response = "HTTP/1.1 204 No Content\r\n\r\n";
        crate::display::display_response(empty_response, 100);
    }

    #[test]
    fn test_display_response_with_redirects() {
        let redirect_response = "HTTP/1.1 301 Moved Permanently\r\nLocation: /new-url\r\n\r\nHTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"success\": true}";
        crate::display::display_response(redirect_response, 250);
    }
}
