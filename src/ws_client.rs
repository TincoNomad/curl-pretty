use colored::*;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub async fn connect_ws(url_str: &str) {
    // Validar URL
    if let Err(e) = Url::parse(url_str) {
        eprintln!("{} URL inválida: {}", "✗".red().bold(), e);
        return;
    }

    println!();
    println!("{} {}", "↔".cyan().bold(), url_str.white().bold());
    println!("{}", "─".repeat(64).dimmed());

    let (ws_stream, response) = match connect_async(url_str).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("{} Error conectando: {}", "✗".red().bold(), e);
            return;
        }
    };

    println!(
        "{} {} ({})",
        "✓".green().bold(),
        "Conectado!".green(),
        format!("HTTP {}", response.status()).dimmed()
    );
    println!("{}", "─".repeat(64).dimmed());
    println!(
        "{}",
        "Escribe mensajes y presiona Enter. /quit para salir.".dimmed()
    );
    println!();

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    // Task para leer mensajes del WebSocket
    let ws_read_task = tokio::spawn(async move {
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    // Prettificar JSON si es JSON
                    if let Ok(json) = serde_json::from_str::<Value>(&text) {
                        println!("{} ", "←".green().bold());
                        print_json_pretty(&json, 1);
                    } else {
                        println!("{} {}", "←".green().bold(), text.green());
                    }
                }
                Ok(Message::Binary(data)) => {
                    println!("{} {} bytes", "←".green().bold(), data.len());
                }
                Ok(Message::Close(close_frame)) => {
                    if let Some(frame) = close_frame {
                        println!("{} Conexión cerrada: {}", "←".yellow().bold(), frame.reason);
                    } else {
                        println!("{} Conexión cerrada", "←".yellow().bold());
                    }
                    break;
                }
                Ok(Message::Ping(data)) => {
                    println!("{} Ping {} bytes", "←".blue().bold(), data.len());
                }
                Ok(Message::Pong(data)) => {
                    println!("{} Pong {} bytes", "←".blue().bold(), data.len());
                }
                Ok(Message::Frame(_)) => {
                    // Raw frame, ignore for now
                }
                Err(e) => {
                    eprintln!("{} Error WebSocket: {}", "✗".red().bold(), e);
                    break;
                }
            }
        }
    });

    // Task para leer stdin y enviar mensajes
    let stdin_task = tokio::spawn(async move {
        let mut stdin_reader = BufReader::new(tokio::io::stdin());
        let mut line = String::new();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            line.clear();
            match stdin_reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim();

                    if trimmed == "/quit" {
                        println!("{} Cerrando conexión...", "←".yellow().bold());
                        let _ = tx.send("/quit".to_string()).await;
                        break;
                    }

                    if !trimmed.is_empty() {
                        if let Err(e) = tx.send(trimmed.to_string()).await {
                            eprintln!("{} Error enviando mensaje: {}", "✗".red().bold(), e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} Error leyendo stdin: {}", "✗".red().bold(), e);
                    break;
                }
            }
        }
    });

    // Task para enviar mensajes al WebSocket
    let ws_write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg_clone = msg.clone();
            if msg == "/quit" {
                let _ = write.send(Message::Close(None)).await;
                break;
            }

            if let Err(e) = write.send(Message::Text(msg)).await {
                eprintln!("{} Error enviando a WebSocket: {}", "✗".red().bold(), e);
                break;
            } else {
                // Mostrar mensaje saliente
                let msg_text = msg_clone.trim();
                if let Ok(json) = serde_json::from_str::<Value>(msg_text) {
                    println!("{} ", "→".cyan().bold());
                    print_json_pretty(&json, 1);
                } else {
                    println!("{} {}", "→".cyan().bold(), msg_text.cyan());
                }
            }
        }
    });

    // Esperar a que alguna tarea termine
    tokio::select! {
        _ = ws_read_task => {},
        _ = stdin_task => {},
        _ = ws_write_task => {},
    }

    println!();
    println!("{} Sesión WebSocket finalizada", "✓".green().bold());
}

fn print_json_pretty(value: &Value, depth: usize) {
    let pad = "  ".repeat(depth);
    let pad_c = "  ".repeat(depth.saturating_sub(1));

    match value {
        Value::Object(map) => {
            if map.is_empty() {
                println!("{}{}", "{}".dimmed(), "");
                return;
            }
            println!("{}", "{".dimmed());
            let entries: Vec<_> = map.iter().collect();
            for (k, v) in entries.iter() {
                print!("{}{} ", pad, format!("\"{}\":", k).cyan());
                print_json_pretty(v, depth + 1);
            }
            print!("{}}}", pad_c);
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                println!("{}{}", "[]".dimmed(), "");
                return;
            }
            println!("{}", "[".dimmed());
            for item in arr.iter() {
                print!("{}", pad);
                print_json_pretty(item, depth + 1);
            }
            print!("{}]", pad_c);
        }
        Value::String(s) => println!("{}{}", format!("\"{}\"", s).green(), ""),
        Value::Number(n) => println!("{}{}", n.to_string().yellow(), ""),
        Value::Bool(b) => println!("{}{}", b.to_string().magenta(), ""),
        Value::Null => println!("{}{}", "null".red().dimmed(), ""),
    }

    // Always end with newline
    println!();
}
