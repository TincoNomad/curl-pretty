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

### Opción 1 — Compilar desde fuente (recomendado)

Requiere Rust 1.80+. Si no tienes Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Luego:
```bash
git clone <este-repo>
cd curlp
cargo build --release
```

Instalar globalmente:
```bash
# Linux / macOS
sudo cp target/release/curlp /usr/local/bin/
# o sin sudo, en tu PATH local:
cp target/release/curlp ~/.local/bin/
```

### Opción 2 — Script de instalación rápida

```bash
./install.sh
```

Compila el binario y lo copia a `~/.local/bin/curlp` (asegúrate de que esté en tu PATH).

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

## WebSocket (roadmap)

El soporte para WS está planificado. Por ahora puedes usar:
```bash
# websocat (instalar con: cargo install websocat)
websocat wss://echo.websocket.org
```

---

## Contribuir

PRs bienvenidos. El código está en dos módulos:
- `src/main.rs` — parser de respuesta HTTP y renderizado
- `src/curl_parser.rs` — tokenizador y reconstrucción del comando curl
