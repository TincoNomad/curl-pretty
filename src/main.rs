use colored::*;
use std::io::{self, IsTerminal, Read};
use std::process::{Command, Stdio};
use std::time::Instant;

mod cli;
mod curl_parser;
mod display;
mod help;
mod version;
mod ws_client;

use clap::Parser;
use cli::{Cli, Commands, OutputMode};
use curl_parser::CurlCommand;
use display::display_response;
use help::print_doctor;
use version::{check_for_update_notification, update_pcurl};

fn main() {
    let cli = Cli::parse();

    // Flags especiales (doctor / update) tienen prioridad
    if cli.doctor {
        print_doctor();
        return;
    }
    if cli.update {
        update_pcurl();
        return;
    }

    // Check NO_COLOR env var (clap doesn't auto-handle this in derive API)
    let no_color = cli.no_color || std::env::var("NO_COLOR").is_ok();

    let output_mode = OutputMode {
        body_only: cli.body_only,
        headers_only: cli.headers_only,
        no_color,
    };

    // Despacho por subcomando
    match cli.command {
        // pcurl ws wss://echo.websocket.org
        Some(Commands::Ws { url, verbose }) => {
            run_websocket(&url, verbose, &output_mode);
        }

        // pcurl wscat -c wss://echo.websocket.org
        Some(Commands::Wscat { connect, verbose }) => {
            run_websocket(&connect, verbose, &output_mode);
        }

        // Sin subcomando → modo HTTP
        None => {
            let stdin_has_data = !io::stdin().is_terminal();

            match cli.curl_command {
                // Modo argumento: pcurl 'curl https://...'
                Some(cmd) => {
                    // Detecta si es una URL directa (no empieza con "curl")
                    let curl_cmd = if cmd.trim_start().starts_with("curl") {
                        cmd
                    } else {
                        format!("curl -si {}", cmd)
                    };
                    run_http_argument_mode(&curl_cmd, &output_mode);
                }

                // Modo pipe: curl -si ... | pcurl
                None => {
                    if stdin_has_data {
                        run_pipe_mode(&output_mode);
                    } else {
                        // stdin es un terminal → no hay pipe → mostrar ayuda
                        eprintln!("pcurl: no se recibió input. Usa `pcurl --help` para ver los modos de uso.");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    // ── Check for updates (silent notification) ───────────────────────
    // This now happens only if no other command/flag took precedence
    check_for_update_notification();
}

// ─────────────────────────────────────────────────────────────────────────────
// Execution
// ─────────────────────────────────────────────────────────────────────────────

// Validar que la URL solo use protocolos seguros o explícitamente permitidos
fn is_safe_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("ws://")
        || url.starts_with("wss://")
        || url.starts_with("ftp://")
        || url.starts_with("file://")
}

// Renamed from execute_curl_and_display
fn run_http_argument_mode(command_str: &str, mode: &OutputMode) {
    let parsed = CurlCommand::parse(command_str);

    // Validar URL para prevenir inyección de comandos maliciosos
    if !parsed.url.is_empty() && !is_safe_url(&parsed.url) {
        eprintln!(
            "{} {}: URL protocol not allowed: {}",
            "❌".red().bold(),
            "Error".red().bold(),
            &parsed.url.split(':').next().unwrap_or("unknown")
        );
        eprintln!(
            "{} Allowed protocols: http, https, ws, wss, ftp, file",
            "➡️".dimmed()
        );
        std::process::exit(1);
    }

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
            display_response(&raw, elapsed, mode);
        }
    }
}

fn run_pipe_mode(mode: &OutputMode) {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Error leyendo stdin");
    println!();
    display_response(&input, 0, mode);
}

async fn run_websocket_async(url: &str, verbose: bool, mode: &OutputMode) {
    ws_client::connect_ws(url, verbose, mode).await;
}

fn run_websocket(url: &str, verbose: bool, mode: &OutputMode) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(run_websocket_async(url, verbose, mode));
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
        crate::display::display_response(
            json_response,
            123,
            &crate::cli::OutputMode {
                body_only: false,
                headers_only: false,
                no_color: false,
            },
        );
    }

    #[test]
    fn test_display_response_xml() {
        let xml_response = "HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\n\r\n<?xml version=\"1.0\"?><root><item>test</item></root>";
        crate::display::display_response(
            xml_response,
            456,
            &crate::cli::OutputMode {
                body_only: false,
                headers_only: false,
                no_color: false,
            },
        );
    }

    #[test]
    fn test_display_response_plain_text() {
        let text_response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!";
        crate::display::display_response(
            text_response,
            789,
            &crate::cli::OutputMode {
                body_only: false,
                headers_only: false,
                no_color: false,
            },
        );
    }

    #[test]
    fn test_display_response_no_headers() {
        let body_only = "{\"message\": \"no headers\"}";
        crate::display::display_response(
            body_only,
            0,
            &crate::cli::OutputMode {
                body_only: false,
                headers_only: false,
                no_color: false,
            },
        );
    }

    #[test]
    fn test_display_response_empty_body() {
        let empty_response = "HTTP/1.1 204 No Content\r\n\r\n";
        crate::display::display_response(
            empty_response,
            100,
            &crate::cli::OutputMode {
                body_only: false,
                headers_only: false,
                no_color: false,
            },
        );
    }

    #[test]
    fn test_display_response_with_redirects() {
        let redirect_response = "HTTP/1.1 301 Moved Permanently\r\nLocation: /new-url\r\n\r\nHTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"success\": true}";
        crate::display::display_response(
            redirect_response,
            250,
            &crate::cli::OutputMode {
                body_only: false,
                headers_only: false,
                no_color: false,
            },
        );
    }
}
