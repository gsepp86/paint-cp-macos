## PaintCP per macOS

App di disegno semplice, in stile Paint di Windows, scritta in Rust.
Questo pacchetto è **universale**: funziona su Apple Silicon e su Intel, macOS 11 o successivo.

### Installazione

1. Scarica `PaintCP-*.dmg` (oppure lo `.zip`).
2. Apri il DMG e trascina **PaintCP** nella cartella **Applicazioni**.
3. Al primo avvio: clic destro sull'app → **Apri** → **Apri**.

L'app non è firmata con un certificato Apple Developer, quindi macOS mostra un avviso di
sicurezza la prima volta. Se l'apertura viene bloccata del tutto, esegui:

```bash
xattr -dr com.apple.quarantine /Applications/PaintCP.app
```

### Verifica del download

I file `SHA256SUMS.txt` contengono gli hash degli allegati:

```bash
shasum -a 256 -c SHA256SUMS.txt
```

### Strumenti

Matita, Gomma, Linea, Rettangolo (vuoto/pieno), Ellisse (vuota/piena), Secchio,
colore libero e tavolozza, spessore 1–48 px, Annulla/Ripeti, apertura PNG/JPG/BMP e
salvataggio PNG.

Scorciatoie: ⌘Z annulla, ⇧⌘Z ripeti, ⌘S salva, ⇧⌘S salva con nome, ⌘O apri, ⌘N nuovo foglio.
