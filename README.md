# PaintCP 1.0

Un Paint semplice per macOS, scritto in Rust con [egui/eframe](https://github.com/emilk/egui).
Finestra unica, tela bianca, pochi strumenti essenziali ma completi: l'idea è quella del vecchio
Paint di Windows, con in più i **livelli** e lo strumento **Fluidifica**.

## Download e installazione

Le versioni pronte per l'uso sono nella pagina **Releases** della repository:
scarica il `.dmg` (o lo `.zip`), trascina **PaintCP** in **Applicazioni** e aprila con
clic destro → **Apri** la prima volta (l'app non è firmata con un certificato Apple).

## Requisiti

- macOS 11 o successivo (Apple Silicon o Intel)
- [Xcode Command Line Tools](https://developer.apple.com/xcode/resources/): `xcode-select --install`
- [Rust](https://rustup.rs): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Avvio

```bash
cargo run --release
```

## App "vera" con icona nel Dock

```bash
chmod +x scripts/build-app.sh
./scripts/build-app.sh
open target/PaintCP.app
```

Lo script compila in release e crea `target/PaintCP.app` con il suo `Info.plist`.

## Strumenti

| Strumento | Cosa fa |
| --- | --- |
| Matita | tratto a mano libera |
| Gomma | cancella verso il trasparente sul livello attivo |
| Linea | linea dritta dall'inizio del trascinamento |
| Rettangolo / Rettangolo pieno | cornice o rettangolo pieno |
| Ellisse / Ellisse piena | ellisse inscritta nel rettangolo trascinato |
| Secchio | riempie l'area contigua dello stesso colore |
| Fluidifica | trascina e deforma i pixel come se fossero bagnati (effetto "liquify") |

Le forme mostrano l'anteprima mentre trascini: la tela viene ridisegnata a ogni movimento e
confermata al rilascio del mouse.

Per lo strumento **Fluidifica** lo spessore regola il raggio del pennello (da 6 a 48 px) e il
cursore **Intensità** (0,1 – 1) decide quanto è deciso lo spostamento dei pixel. Il tratto si può
annullare come qualsiasi altro disegno.

Altri comandi: colore libero (selettore), tavolozza pronta, spessore del tratto da 1 a 48 px,
**Annulla/Ripeti**, **Nuovo** foglio, **Apri** e **Salva** in PNG (si possono aprire anche JPG e BMP).

## Livelli

Il pannello **Livelli** a destra gestisce la pila dei livelli, dal più alto (in cima all'elenco)
al più basso:

- **Nuovo** aggiunge un livello trasparente sopra quello attivo;
- **Duplica** copia il livello attivo;
- **Elimina** rimuove il livello attivo (ne resta sempre almeno uno, lo `Sfondo` bianco);
- **▲ / ▼** spostano il livello attivo sopra o sotto quello vicino;
- **Unisci** fonde il livello attivo in quello sottostante (rispettando la sua opacità);
- la casella accanto al nome mostra o nasconde il livello, un clic sul nome lo seleziona;
- il cursore in fondo regola l'**opacità** del livello attivo.

Si disegna sempre e solo sul livello attivo (quello evidenziato); l'unione dei livelli è quello
che si vede sulla tela e quello che viene salvato nel PNG, trasparenze comprese (sulla tela sono
mostrate con una scacchiera). Aggiunte, eliminazioni, spostamenti, unioni e cambi di opacità
entrano nella cronologia di Annulla/Ripeti.

## Scorciatoie

| Scorciatoia | Azione |
| --- | --- |
| ⌘Z | Annulla |
| ⇧⌘Z / ⌘Y | Ripeti |
| ⌘S | Salva |
| ⇧⌘S | Salva con nome… |
| ⌘O | Apri… |
| ⌘N | Nuovo foglio |

Nuovo foglio e Apri chiedono conferma se ci sono modifiche non salvate: si può salvare e
continuare, scartare o annullare. Lo stato "● modifiche non salvate" è nella barra in basso.

## Struttura

```
src/main.rs    finestra e avvio dell'app
src/app.rs     interfaccia, strumenti, livelli, gestione tratti e file
src/paint.rs   disegno sui pixel (linee, rettangoli, ellissi, riempimento, fluidifica)
scripts/build-app.sh   compila e crea PaintCP.app per il Mac corrente
scripts/bundle-app.sh  crea PaintCP.app da un binario (usato anche dalla CI)
.github/workflows/release.yml  build universale e release su tag
```

La tela predefinita è 1024 × 700 px; aprendo un'immagine la tela assume le sue dimensioni
(fino a 4096 px per lato). I file vengono salvati come PNG RGBA.

## Pubblicare una release

Il workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) costruisce su un
runner macOS un binario **universale** (Apple Silicon + Intel), crea `PaintCP.app`, lo
impacchetta in ZIP e DMG con i checksum e pubblica il tutto nella release del tag.

```bash
git tag v1.0.0
git push origin v1.0.0
```

Si può anche avviare a mano da GitHub → *Actions* → *Release macOS* → *Run workflow*,
indicando il tag da pubblicare.

## Verifiche

Il progetto è compilato con `cargo check --all-targets` (Linux) e `cargo clippy`.
L'app non viene avviata in questa macchina (è un ambiente Linux senza display), quindi
l'interfaccia va provata su macOS.
