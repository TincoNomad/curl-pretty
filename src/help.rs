use colored::*;
use std::process::Command;

pub fn print_help() {
    let v = env!("CARGO_PKG_VERSION");
    println!();
    println!(
        "  {} {}  —  HTTP pretty-printer for your terminal",
        "pcurl".cyan().bold(),
        format!("v{}", v).dimmed()
    );
    println!("  {}", "─".repeat(54).dimmed());
    println!();
    println!("  {}", "USAGE MODES".white().bold());
    println!();
    println!("  {}  {}", "pcurl".cyan(), "[OPCIONES]".white());
    println!(
        "  {}  {}",
        "pcurl".cyan(),
        "'curl [opciones] <url>'".white()
    );
    println!("  {}  {}", "pcurl".cyan(), "<websocket-url>".white());
    println!("  {}  {}", "curl".cyan(), "-si <url> | pcurl".white());
    println!();
    println!("  {}", "OPTIONS".white().bold());
    println!();
    println!("  {}  {}", "-h, --help".yellow(), "Show this help".white());
    println!("  {}  {}", "-V, --version".yellow(), "Show version".white());
    println!(
        "  {}  {}",
        "--doctor".yellow(),
        "Diagnose installation and PATH".white()
    );
    println!(
        "  {}  {}",
        "--update".yellow(),
        "Update to latest version".white()
    );
    println!();
    println!("  {}", "HTTP MODE".white().bold());
    println!();
    println!(
        "  {}  {}",
        "1. Argument mode".yellow().bold(),
        "pcurl executes curl and prettifies the response".white()
    );
    println!("     {}", "pass curl raw output directly to pcurl".dimmed());
    println!();
    println!(
        "  {}  {}",
        "2. Pipe mode".yellow().bold(),
        "curl -si <url> | pcurl".white()
    );
    println!(
        "     {}",
        "(-s silences progress, -i includes headers)".dimmed()
    );
    println!();
    println!("  {}", "WEBSOCKET MODE".white().bold());
    println!();
    println!(
        "  {}",
        "Direct connection  pcurl wss://echo.websocket.org".white()
    );
    println!(
        "  {}  {}",
        "wscat command".white(),
        "pcurl 'wscat -c wss://echo.websocket.org'".white()
    );
    println!();
    println!("  {}", "HTTP EXAMPLES".white().bold());
    println!();
    println!("  {}", "# Simple GET".dimmed());
    println!(
        "  {}",
        "pcurl 'curl https://jsonplaceholder.typicode.com/todos/1'".dimmed()
    );
    println!();
    println!("  {}", "# POST with JSON".dimmed());
    println!(
        "  {}",
        "pcurl 'curl -X POST https://httpbin.org/post \\".dimmed()
    );
    println!(
        "  {}",
        "  -H \"Content-Type: application/json\" \\".dimmed()
    );
    println!(
        "  {}",
        "  -d \'{{\"user\":\"juan\",\"role\":\"admin\"}}\''".dimmed()
    );
    println!();
    println!("  {}", "# With authentication".dimmed());
    println!(
        "  {}",
        "pcurl 'curl -u user:password https://api.example.com/private'".dimmed()
    );
    println!();
    println!("  {}", "# Custom headers".dimmed());
    println!(
        "  {}",
        "pcurl 'curl -H \"Authorization: Bearer <token>\" https://api.example.com/me'".dimmed()
    );
    println!();
    println!("  {}", "# Follow redirects".dimmed());
    println!(
        "  {}",
        "pcurl 'curl -L https://httpbin.org/redirect/1'".dimmed()
    );
    println!();
    println!("  {}", "HTTP FEATURES".white().bold());
    println!();
    println!("  {}  JSON with colors and indentation", "◆".cyan());
    println!("  {}  XML formatted", "◆".cyan());
    println!("  {}  Colored headers", "◆".cyan());
    println!(
        "  {}  Status colors (2xx green · 3xx yellow · 4xx/5xx red)",
        "◆".cyan()
    );
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
    println!(
        "  {}  {}",
        "Universal script:".dimmed(),
        "curl -sSL https://raw.githubusercontent.com/tinconomad/pretty-curl/main/install.sh | bash"
            .white()
    );
    println!(
        "  {}  {}",
        "Manual:".dimmed(),
        "https://github.com/tinconomad/pretty-curl/releases".white()
    );
    println!();
}

pub fn print_doctor() {
    use std::env;

    println!();
    println!("  {}  —  Diagnóstico de instalación", "pcurl".cyan().bold());
    println!("  {}", "─".repeat(54).dimmed());
    println!();

    // Verificar ubicación del binario
    let current_exe = env::current_exe().ok();
    let install_paths = vec![
        format!("{}/.local/bin/pcurl", env::var("HOME").unwrap_or_default()),
        "/usr/local/bin/pcurl".to_string(),
        "/usr/bin/pcurl".to_string(),
    ];

    println!("  {}", "UBICACIÓN DEL BINARIO".white().bold());
    println!();

    if let Some(exe_path) = current_exe {
        println!(
            "  {}  {}",
            "✅".green(),
            format!("Ejecutando desde: {}", exe_path.display()).white()
        );
    }

    let mut found_in_path = false;
    for path in &install_paths {
        if std::path::Path::new(path).exists() {
            println!(
                "  {}  {}",
                "✅".green(),
                format!("Encontrado en: {}", path).white()
            );
            found_in_path = true;
        }
    }

    if !found_in_path {
        println!(
            "  {}  {}",
            "❌".red(),
            "No encontrado en ubicaciones estándar".white()
        );
    }
    println!();

    // Verificar PATH
    println!("  {}", "CONFIGURACIÓN DE PATH".white().bold());
    println!();

    let path = env::var("PATH").unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();
    let local_bin = format!("{}/.local/bin", home);

    if path.contains(&local_bin) {
        println!(
            "  {}  {}",
            "✅".green(),
            format!("{} está en PATH", local_bin).white()
        );
    } else {
        println!(
            "  {}  {}",
            "⚠️".yellow(),
            format!("{} NO está en PATH", local_bin).white()
        );
        println!();
        println!("  {}", "SOLUCIÓN:".yellow().bold());
        println!("  {}", "Agrega esto a tu ~/.bashrc o ~/.zshrc:".white());
        println!(
            "  {}",
            "export PATH=\"$HOME/.local/bin:$PATH\"".to_string().cyan()
        );
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
            println!(
                "  {}  {}",
                "✅".green(),
                format!("curl: {}", first_line).white()
            );
        }
        _ => println!(
            "  {}  {}",
            "❌".red(),
            "curl: No encontrado (requerido para HTTP mode)".white()
        ),
    }
    println!();

    // Test de conectividad
    println!("  {}", "PRUEBA DE CONECTIVIDAD".white().bold());
    println!();

    match Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "https://httpbin.org/get",
        ])
        .output()
    {
        Ok(output) if output.status.success() => {
            let code = String::from_utf8_lossy(&output.stdout);
            if code.trim() == "200" {
                println!("  {}  {}", "✅".green(), "Conexión a internet: OK".white());
            } else {
                println!(
                    "  {}  {}",
                    "⚠️".yellow(),
                    format!("HTTP status: {}", code.trim()).white()
                );
            }
        }
        _ => println!(
            "  {}  {}",
            "❌".red(),
            "No se pudo conectar a internet".white()
        ),
    }
    println!();

    // Resumen
    println!("  {}", "RESUMEN".white().bold());
    println!();
    if found_in_path && path.contains(&local_bin) {
        println!(
            "  {}  {}",
            "✅".green(),
            "Todo está correctamente configurado!".white()
        );
        println!();
        println!("  {}", "Prueba con:".dimmed());
        println!("  {}", "pcurl 'curl https://httpbin.org/get'".cyan());
    } else {
        println!(
            "  {}  {}",
            "⚠️".yellow(),
            "Se encontraron problemas. Revisa las soluciones arriba.".white()
        );
    }
    println!();
}
