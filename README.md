# PaintCP

Un Paint semplice per macOS, scritto in Rust con [egui/eframe](https://github.com/emilk/egui).
Finestra unica, tela bianca, pochi strumenti essenziali: l'idea è quella del vecchio Paint di Windows.

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

## Pubblicare una release

Il workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) costruisce su un
runner macOS un binario **universale** (Apple Silicon + Intel), crea `PaintCP.app`, lo
impacchetta in ZIP e DMG con i checksum e pubblica il tutto nella release del tag.

```bash
git tag v0.1.0
git push origin v0.1.0
```

Si può anche avviare a mano da GitHub → *Actions* → *Release macOS* → *Run workflow*,
indicando il tag da pubblicare.

## Strumenti

| Strumento | Cosa fa |
| --- | --- |
| Matita | tratto a mano libera |
| Gomma | cancella verso il bianco |
| Linea | linea dritta dall'inizio del trascinamento |
| Rettangolo / Rettangolo pieno | cornice o rettangolo pieno |
| Ellisse / Ellisse piena | ellisse inscritta nel rettangolo trascinato |
| Secchio | riempie l'area contigua dello stesso colore |

Le forme mostrano l'anteprima mentre trascini: la tela viene ridisegnata a ogni movimento e confermata al rilascio del mouse.

Altri comandi: colore libero (selettore), tavolozza pronta, spessore del tratto da 1 a 48 px, **Annulla/Ripeti** (ultimi 12 passaggi), **Nuovo** foglio, **Apri** e **Salva** in PNG (si possono aprire anche JPG e BMP).

## Scorciatoie

| Scorciatoia | Azione |
| --- | --- |
| ⌘Z | Annulla |
| ⇧⌘Z / ⌘Y | Ripeti |
| ⌘S | Salva |
| ⇧⌘S | Salva con nome… |
| ⌘O | Apri… |
| ⌘N | Nuovo foglio |

## Struttura

```
src/main.rs    finestra e avvio dell'app
src/app.rs     interfaccia, strumenti, gestione tratti e file
src/paint.rs   disegno sui pixel (linee, rettangoli, ellissi, riempimento)
scripts/build-app.sh   compila e crea PaintCP.app per il Mac corrente
scripts/bundle-app.sh  crea PaintCP.app da un binario (usato anche dalla CI)
.github/workflows/release.yml  build universale e release su tag
```

La tela predefinita è 1024 × 700 px; aprendo un'immagine la tela assume le sue dimensioni
(fino a 4096 px per lato). I file vengono salvati come PNG RGBA.

## Verifiche

Il progetto è stato compilato con `cargo check` per il target `aarch64-apple-darwin` (oltre che per Linux).
L'app non è stata avviata in questa macchina (è un ambiente Linux senza display), quindi l'interfaccia va provata su macOS.
