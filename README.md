# curlp

HTTP pretty-printer para tu terminal. Toma la salida cruda de `curl` y la convierte en algo legible — como Postman o Bruno, pero sin salir de la terminal.

```
  ✓  HTTP/2 200 OK    142ms

  HEADERS
  ──────────────────────────────────────────────────────
  Content-Type                   application/json
  X-Request-Id                   abc-def-123

  BODY
  ──────────────────────────────────────────────────────
{
  "id": 42,
  "nombre": "Juan Pérez",
  "permisos": [
    "read",
    "write",
    "delete"
  ]
}
```

## Instalación

### Opción 1 — Instalador Universal (recomendado)

**Sin necesidad de Rust. Descarga binario precompilado para tu sistema:**

```bash
curl -sSL https://raw.githubusercontent.com/tinconomad/curl-pretty/main/install.sh | bash
```

El instalador detecta automáticamente:
- Linux (x64, ARM64) 
- macOS (Intel, Apple Silicon)
- Descarga el binario correcto desde GitHub Releases
- Fallback a compilación si no hay binario disponible

### Opción 2 — Descarga Manual

Ve a [GitHub Releases](https://github.com/tinconomad/curl-pretty/releases) y descarga:

- `curlp-linux-x64` - Linux 64-bit
- `curlp-linux-arm64` - Linux ARM64  
- `curlp-macos-x64` - macOS Intel
- `curlp-macos-arm64` - macOS Apple Silicon
- `curlp-windows-x64.exe` - Windows 64-bit

Luego:
```bash
# Linux/macOS
chmod +x curlp-*
sudo cp curlp-* /usr/local/bin/curlp

# Windows
# Mueve curlp.exe a un directorio en tu PATH
```

### Opción 3 — Compilar desde Fuente

Si tienes Rust instalado:

```bash
git clone https://github.com/tinconomad/curl-pretty
cd curl-pretty
cargo build --release
sudo cp target/release/curlp /usr/local/bin/
```

---

## Uso

### Modo 1 — Argumento (recomendado)

`curlp` ejecuta el `curl` por ti y prettifica la respuesta:

```bash
# GET simple
curlp 'curl https://api.ejemplo.com/users/1'

# POST con JSON
curlp 'curl -X POST https://api.ejemplo.com/users \
  -H "Authorization: Bearer <token>" \
  -d '"'"'{"nombre":"Juan","rol":"admin"}'"'"''

# Con flags extra
curlp 'curl -L -k https://api.interna.com/health'
curlp 'curl -u usuario:contraseña https://api.ejemplo.com/private'
```

### Modo 2 — Pipe

Si prefieres ejecutar `curl` tú mismo, usa `-si` y pipea:

```bash
curl -si https://api.ejemplo.com/users/1 | curlp
curl -si -X DELETE https://api.ejemplo.com/users/42 | curlp
```

> `-s` silencia la barra de progreso, `-i` incluye los headers en stdout.

### Alias recomendados

Agrega esto a tu `.bashrc` / `.zshrc`:

```bash
# Prettifica cualquier curl automáticamente
curlp() { command curlp "$@"; }

# O un alias más corto
alias cget='curlp curl'
alias cpost='curlp curl -X POST'
```

---

## Qué muestra

| Elemento | Descripción |
|---|---|
| **Status** | Código + texto, coloreado: 🟢 2xx · 🟡 3xx · 🔴 4xx/5xx |
| **Tiempo** | Milisegundos de la petición (modo argumento) |
| **Headers** | Clave alineada + valor, con clave en cyan |
| **Body JSON** | Indentado con colores: strings verde, números amarillo, booleans magenta, null rojo |
| **Body XML** | Árbol indentado con tags en cyan |
| **Body texto** | Plano, sin modificar |

---

## WebSocket

**¡Nuevo! Soporte WebSocket integrado:**

```bash
# Conectar a WebSocket URL
curlp wss://echo.websocket.org
curlp ws://localhost:8080/chat

# Comandos estilo wscat también funcionan
curlp wscat -c wss://echo.websocket.org
```

Características:
- **JSON prettifier** automático para mensajes
- **Prefijos coloreados**: `←` entrante (verde), `→` saliente (cyan)
- **Interactivo**: Escribe mensajes y presiona Enter
- **Comando `/quit`** para cerrar conexión
- **Status de conexión** al iniciar

Ejemplo de sesión:
```
↔ wss://echo.websocket.org
────────────────────────────────────────────────────────────
✓ Conectado! (HTTP 101)
────────────────────────────────────────────────────────────
Escribe mensajes y presiona Enter. /quit para salir.

> hola mundo
← "hola mundo"

> {"type":"ping","timestamp":123456}
← {
     "type": "ping",
     "timestamp": 123456
   }
```

---

## Contribuir

PRs bienvenidos. El código está en dos módulos:
- `src/main.rs` — parser de respuesta HTTP y renderizado
- `src/curl_parser.rs` — tokenizador y reconstrucción del comando curl
