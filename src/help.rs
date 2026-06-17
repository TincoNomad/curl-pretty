use colored::*;
use std::process::Command;

pub fn print_doctor() {
    use std::env;

    println!();
    println!("  {}  —  Installation Diagnostic", "pcurl".cyan().bold());
    println!("  {}", "─".repeat(54).dimmed());
    println!();

    // Check binary location
    let current_exe = env::current_exe().ok();
    let install_paths = vec![
        format!("{}/.local/bin/pcurl", env::var("HOME").unwrap_or_default()),
        "/usr/local/bin/pcurl".to_string(),
        "/usr/bin/pcurl".to_string(),
    ];

    println!("  {}", "BINARY LOCATION".white().bold());
    println!();

    if let Some(exe_path) = current_exe {
        println!(
            "  {}  {}",
            "✅".green(),
            format!("Running from: {}", exe_path.display()).white()
        );
    }

    let mut found_in_path = false;
    for path in &install_paths {
        if std::path::Path::new(path).exists() {
            println!(
                "  {}  {}",
                "✅".green(),
                format!("Found at: {}", path).white()
            );
            found_in_path = true;
        }
    }

    if !found_in_path {
        println!(
            "  {}  {}",
            "❌".red(),
            "Not found in standard locations".white()
        );
    }
    println!();

    // Check PATH
    println!("  {}", "PATH CONFIGURATION".white().bold());
    println!();

    let path = env::var("PATH").unwrap_or_default();
    let home = env::var("HOME").unwrap_or_default();
    let local_bin = format!("{}/.local/bin", home);

    if path.contains(&local_bin) {
        println!(
            "  {}  {}",
            "✅".green(),
            format!("{} is in PATH", local_bin).white()
        );
    } else {
        println!(
            "  {}  {}",
            "⚠️".yellow(),
            format!("{} is NOT in PATH", local_bin).white()
        );
        println!();
        println!("  {}", "FIX:".yellow().bold());
        println!("  {}", "Add this to your ~/.bashrc or ~/.zshrc:".white());
        println!(
            "  {}",
            "export PATH=\"$HOME/.local/bin:$PATH\"".to_string().cyan()
        );
        println!();
        println!("  {}", "Then reload your config:".white());
        println!("  {}", "source ~/.zshrc  # or source ~/.bashrc".cyan());
    }
    println!();

    // Check curl
    println!("  {}", "DEPENDENCIES".white().bold());
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
            "curl: Not found (required for HTTP mode)".white()
        ),
    }
    println!();

    // Connectivity test
    println!("  {}", "CONNECTIVITY TEST".white().bold());
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
                println!("  {}  {}", "✅".green(), "Internet connection: OK".white());
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
            "Could not connect to the internet".white()
        ),
    }
    println!();

    // Summary
    println!("  {}", "SUMMARY".white().bold());
    println!();
    if found_in_path && path.contains(&local_bin) {
        println!(
            "  {}  {}",
            "✅".green(),
            "Everything is properly configured!".white()
        );
        println!();
        println!("  {}", "Try running:".dimmed());
        println!("  {}", "pcurl 'curl https://httpbin.org/get'".cyan());
    } else {
        println!(
            "  {}  {}",
            "⚠️".yellow(),
            "Found some issues. See the fixes above.".white()
        );
    }
    println!();
}
