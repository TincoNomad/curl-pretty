use colored::*;
use serde_json::Value;

pub fn check_latest_version() -> Result<String, Box<dyn std::error::Error>> {
    let response =
        ureq::get("https://api.github.com/repos/TincoNomad/pretty-curl/releases/latest").call()?;

    let text = response.into_string()?;
    let json: Value = serde_json::from_str(&text)?;

    if let Some(tag) = json.get("tag_name").and_then(|v| v.as_str()) {
        return Ok(tag.to_string());
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
