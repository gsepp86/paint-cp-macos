#!/usr/bin/env bash
# Crea il bundle macOS PaintCP.app a partire da un binario già compilato.
#
# Uso: ./scripts/bundle-app.sh <percorso-binario> [cartella-output]
# Esempio: ./scripts/bundle-app.sh target/release/paintcp target
#
# Variabili opzionali:
#   PAINTCP_VERSION  versione scritta in Info.plist (default 1.0.0)
set -euo pipefail

BINARY="${1:?uso: bundle-app.sh <percorso-binario> [cartella-output]}"
OUT_DIR="${2:-target}"
VERSION="${PAINTCP_VERSION:-1.0.0}"

if [ ! -x "$BINARY" ]; then
    echo "Errore: binario non trovato o non eseguibile: $BINARY" >&2
    exit 1
fi

APP_DIR="$OUT_DIR/PaintCP.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "$BINARY" "$APP_DIR/Contents/MacOS/PaintCP"
chmod +x "$APP_DIR/Contents/MacOS/PaintCP"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>PaintCP</string>
    <key>CFBundleDisplayName</key>
    <string>PaintCP</string>
    <key>CFBundleIdentifier</key>
    <string>com.paintcp.app</string>
    <key>CFBundleExecutable</key>
    <string>PaintCP</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
PLIST

# Firma ad-hoc: non è una firma Developer ID, ma evita alcuni errori di avvio
# sui binari scaricati. La notarizzazione richiede un certificato Apple.
if command -v codesign >/dev/null 2>&1; then
    codesign --force --deep --sign - "$APP_DIR" >/dev/null 2>&1 \
        || echo "Avviso: firma ad-hoc non riuscita (l'app funziona comunque)" >&2
fi

echo "Creato: $APP_DIR"
