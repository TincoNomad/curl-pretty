use colored::*;

pub fn check_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let response =
        ureq::get("https://api.github.com/repos/TincoNomad/pretty-curl/releases/latest").call()?;

    let text = response.into_string()?;
    if let Some(tag_line) = text.lines().find(|line| line.contains("\"tag_name\"")) {
        if let Some(tag) = tag_line.split(':').nth(1) {
            let tag = tag.trim().trim_matches('"');
            return Ok(tag.to_string());
        }
    }

    Err("Failed to parse version".into())
}

pub fn update_pcurl() {
    println!("{} Updating pcurl...", "🔄".cyan());

    match std::process::Command::new("sh")
        .arg("-c")
        .arg("curl -sSL https://raw.githubusercontent.com/TincoNomad/pretty-curl/main/install.sh | bash")
        .status()
    {
        Ok(status) if status.success() => {
            println!("{} Update successful!", "✅".green());
            println!("{} Please restart your terminal or run: {}", "→".dimmed(), "source ~/.bashrc".cyan());
        }
        Ok(_) => {
            println!("{} Update failed. Please try manually:", "❌".red());
            println!("  {}", "curl -sSL https://raw.githubusercontent.com/TincoNomad/pretty-curl/main/install.sh | bash".cyan());
        }
        Err(e) => {
            println!("{} Failed to run update: {}", "❌".red(), e);
        }
    }
}

pub fn check_for_update_notification() {
    let current = env!("CARGO_PKG_VERSION");
    if let Ok(latest) = check_latest_version() {
        // Normalizar: quitar prefijo 'v' si existe para comparación correcta
        let latest_clean = latest.trim_start_matches('v');
        if latest_clean != current {
            eprintln!(
                "{} New version available: {} → {} (run {} to update)",
                "⚠️".yellow(),
                current,
                latest.green(),
                "pcurl --update".cyan()
            );
            eprintln!(); // Add spacing
        }
    }
}
