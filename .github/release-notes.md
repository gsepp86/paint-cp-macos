## PaintCP 1.0 per macOS

App di disegno semplice, in stile Paint di Windows, scritta in Rust.
Questo pacchetto è **universale**: funziona su Apple Silicon e su Intel, macOS 11 o successivo.

### Novità della 1.0

- **Livelli**: pannello laterale con aggiunta, duplicazione, eliminazione, spostamento,
  unione verso il basso, visibilità e opacità per ogni livello. Si disegna sul livello attivo.
- **Fluidifica**: nuovo strumento che trascina e deforma i pixel con una caduta morbida,
  con intensità regolabile (liquify).
- **Gomma trasparente**: cancella davvero i pixel del livello attivo invece di dipingere di bianco.
- **Salvataggio con trasparenza**: il PNG esportato contiene l'alfa dell'unione dei livelli.
- **Anteprime più morbide**: linee, ellissi e rettangoli hanno il bordo sfumato di un pixel.
- **Prestazioni**: riempimento a scansione (più veloce sulle aree grandi), tratti disegnati a
  passi regolari, ricomposizione dei livelli solo quando serve, build con LTO e codegen unico.
- **Sicurezza dei dati**: Nuovo foglio e Apri chiedono conferma se ci sono modifiche non salvate;
  la barra di stato segnala le modifiche da salvare.
- Correzioni: righe/colonne in più nelle forme piene, cronologia "Ripeti" senza limite di memoria,
  posizione del cursore fuori tela, gomma che sporcava i livelli superiori.

### Installazione

1. Scarica `PaintCP-1.0.0.dmg` (oppure lo `.zip`).
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

### Scorciatoie

⌘Z annulla, ⇧⌘Z ripeti, ⌘S salva, ⇧⌘S salva con nome, ⌘O apri, ⌘N nuovo foglio.
