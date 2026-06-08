use colored::*;
use std::process::Command;

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
