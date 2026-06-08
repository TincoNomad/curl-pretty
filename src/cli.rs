use clap::{Parser, Subcommand};

// ------------------------------------------------------------
// CLI principal
// ------------------------------------------------------------

/// HTTP pretty-printer para tu terminal.
/// Ejecuta curl y embellece la respuesta — como Postman, sin salir de la terminal.
#[derive(Parser, Debug)]
#[command(
    name = "pcurl",
    version,                          // toma la versión de Cargo.toml automáticamente
    author,
    about,
    long_about = None,
    after_help = "EJEMPLOS:\n  pcurl 'curl https://api.example.com/users/1'\n  pcurl 'curl -X POST https://api.example.com/users -H \"Content-Type: application/json\" -d \\'{}\\'\n  curl -si https://api.example.com/users | pcurl\n  pcurl ws ws://localhost:8080/chat",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Comando curl completo entre comillas, o URL directa.
    /// Si se omite, pcurl lee desde stdin (modo pipe).
    pub curl_command: Option<String>,

    /// Muestra solo el body, sin headers ni status
    #[arg(long, global = true)]
    pub body_only: bool,

    /// Muestra solo los headers, sin body
    #[arg(long, global = true)]
    pub headers_only: bool,

    /// Fuerza salida sin colores (útil para pipes o logs)
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Verifica la instalación y dependencias
    #[arg(long)]
    pub doctor: bool,

    /// Actualiza pcurl a la última versión disponible
    #[arg(long)]
    pub update: bool,
}

// ------------------------------------------------------------
// Subcomandos
// ------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Conecta a un servidor WebSocket de forma interactiva
    Ws {
        /// URL del WebSocket (ws:// o wss://)
        url: String,

        /// Muestra mensajes de ping/pong
        #[arg(long)]
        verbose: bool,
    },

    /// Alias wscat-compatible: pcurl wscat -c <url>
    Wscat {
        /// URL del WebSocket
        #[arg(short = 'c', long)]
        connect: String,

        /// Muestra mensajes de ping/pong
        #[arg(long)]
        verbose: bool,
    },
}

// ------------------------------------------------------------
// Struct auxiliar para opciones de output
// ------------------------------------------------------------

pub struct OutputMode {
    pub body_only: bool,
    pub headers_only: bool,
    pub no_color: bool,
}
