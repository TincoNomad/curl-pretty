pub fn extract_ws_url(command: &str) -> Option<String> {
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
