//! Testi dell'interfaccia: italiano (predefinito) e inglese.
//!
//! Tutte le stringhe mostrate all'utente stanno qui, così l'interfaccia non
//! contiene testo fisso e la lingua si può cambiare a runtime dalla finestra
//! *Impostazioni*.

/// Lingua dell'interfaccia.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Italian,
    English,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Italian, Lang::English];

    /// Nome della lingua, scritto nella lingua stessa.
    pub fn name(self) -> &'static str {
        self.texts().language_name
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::Italian => &ITALIAN,
            Lang::English => &ENGLISH,
        }
    }
}

/// Insieme dei testi di una lingua.
#[derive(Clone, Copy)]
pub struct Texts {
    lang: Lang,
    /// Nome della lingua nel selettore.
    pub language_name: &'static str,

    // Barra degli strumenti.
    pub new_canvas: &'static str,
    pub open: &'static str,
    pub save: &'static str,
    pub save_as: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub color: &'static str,
    pub width: &'static str,
    pub strength: &'static str,
    pub tool: &'static str,
    pub palette: &'static str,
    pub pick_color: &'static str,
    /// Etichette degli strumenti, nell'ordine di `Tool::ALL`.
    pub tools: [&'static str; 9],
    /// Descrizioni mostrate al passaggio del mouse sugli strumenti.
    pub tool_hints: [&'static str; 9],

    // Pannello dei livelli.
    pub layers: &'static str,
    pub layer_new: &'static str,
    pub layer_duplicate: &'static str,
    pub layer_delete: &'static str,
    pub layer_merge: &'static str,
    pub hint_add_layer: &'static str,
    pub hint_duplicate_layer: &'static str,
    pub hint_delete_layer: &'static str,
    pub hint_move_up: &'static str,
    pub hint_move_down: &'static str,
    pub hint_merge_layer: &'static str,
    pub hint_visible: &'static str,
    pub hint_select: &'static str,
    pub layer_opacity: &'static str,
    pub background_layer: &'static str,
    pub layer_word: &'static str,
    pub copy_suffix: &'static str,

    // Barra di stato.
    pub unsaved: &'static str,
    pub unsaved_changes: &'static str,
    pub status_ready: &'static str,
    pub status_undo: &'static str,
    pub status_redo: &'static str,
    pub status_filled: &'static str,
    pub status_stroke_done: &'static str,
    pub status_new_layer: &'static str,
    pub status_layer_duplicated: &'static str,
    pub status_need_layer: &'static str,
    pub status_layer_deleted: &'static str,
    pub status_layer_up: &'static str,
    pub status_layer_down: &'static str,
    pub status_nothing_to_merge: &'static str,
    pub status_layers_merged: &'static str,
    pub status_layer_shown: &'static str,
    pub status_layer_hidden: &'static str,
    pub status_new_canvas: &'static str,
    pub opened_label: &'static str,
    pub saved_label: &'static str,

    // Finestra di conferma.
    pub confirm_title: &'static str,
    pub confirm_new: &'static str,
    pub confirm_open: &'static str,
    pub confirm_save: &'static str,
    pub confirm_discard: &'static str,
    pub confirm_cancel: &'static str,

    // Impostazioni.
    pub settings: &'static str,
    pub language: &'static str,
    pub language_hint: &'static str,
    pub close: &'static str,

    // File e messaggi di errore.
    pub images_filter: &'static str,
    pub png_filter: &'static str,
    pub default_file: &'static str,
    pub err_open: &'static str,
    pub err_empty: &'static str,
    pub err_save: &'static str,
}

impl Texts {
    pub fn opened(&self, name: &str) -> String {
        format!("{}: {name}", self.opened_label)
    }

    pub fn saved(&self, name: &str) -> String {
        format!("{}: {name}", self.saved_label)
    }

    pub fn max_layers(&self, max: usize) -> String {
        match self.lang {
            Lang::Italian => format!("Massimo {max} livelli"),
            Lang::English => format!("At most {max} layers"),
        }
    }

    pub fn opacity(&self, percent: i32) -> String {
        match self.lang {
            Lang::Italian => format!("Opacità {percent}%"),
            Lang::English => format!("Opacity {percent}%"),
        }
    }

    pub fn err_opening(&self, error: impl std::fmt::Display) -> String {
        format!("{}: {error}", self.err_open)
    }

    pub fn err_saving(&self, error: impl std::fmt::Display) -> String {
        format!("{}: {error}", self.err_save)
    }

    /// Messaggio per un'immagine oltre il limite di lato.
    pub fn err_too_large(&self, width: usize, height: usize, max: usize) -> String {
        match self.lang {
            Lang::Italian => format!(
                "Immagine troppo grande: {width} × {height} px, massimo {max} px per lato"
            ),
            Lang::English => {
                format!("Image too large: {width} × {height} px, maximum {max} px per side")
            }
        }
    }
}

const ITALIAN: Texts = Texts {
    lang: Lang::Italian,
    language_name: "Italiano",

    new_canvas: "Nuovo",
    open: "Apri…",
    save: "Salva",
    save_as: "Salva con nome…",
    undo: "↶ Annulla",
    redo: "↷ Ripeti",
    color: "Colore:",
    width: "Spessore:",
    strength: "Intensità:",
    tool: "Strumento:",
    palette: "Tavolozza:",
    pick_color: "Usa questo colore",
    tools: [
        "Matita",
        "Gomma",
        "Linea",
        "Rettangolo",
        "Rettangolo pieno",
        "Ellisse",
        "Ellisse piena",
        "Secchio",
        "Fluidifica",
    ],
    tool_hints: [
        "Tratto a mano libera",
        "Cancella il livello attivo (trasparente)",
        "Linea dritta, con anteprima",
        "Rettangolo vuoto, con anteprima",
        "Rettangolo pieno, con anteprima",
        "Ellisse vuota, con anteprima",
        "Ellisse piena, con anteprima",
        "Riempie l'area contigua dello stesso colore",
        "Trascina e deforma i pixel del livello attivo",
    ],

    layers: "Livelli",
    layer_new: "Nuovo",
    layer_duplicate: "Duplica",
    layer_delete: "Elimina",
    layer_merge: "Unisci",
    hint_add_layer: "Aggiunge un livello trasparente sopra quello attivo",
    hint_duplicate_layer: "Copia il livello attivo",
    hint_delete_layer: "Elimina il livello attivo",
    hint_move_up: "Sposta sopra",
    hint_move_down: "Sposta sotto",
    hint_merge_layer: "Unisce il livello attivo a quello sotto",
    hint_visible: "Mostra o nascondi il livello",
    hint_select: "Seleziona il livello",
    layer_opacity: "Opacità del livello",
    background_layer: "Sfondo",
    layer_word: "Livello",
    copy_suffix: "copia",

    unsaved: "non salvato",
    unsaved_changes: "● modifiche non salvate",
    status_ready: "Pronto",
    status_undo: "Annullato",
    status_redo: "Ripetuto",
    status_filled: "Riempimento",
    status_stroke_done: "Disegno completato",
    status_new_layer: "Nuovo livello",
    status_layer_duplicated: "Livello duplicato",
    status_need_layer: "Serve almeno un livello",
    status_layer_deleted: "Livello eliminato",
    status_layer_up: "Livello spostato sopra",
    status_layer_down: "Livello spostato sotto",
    status_nothing_to_merge: "Nessun livello sotto da unire",
    status_layers_merged: "Livelli uniti",
    status_layer_shown: "Livello visibile",
    status_layer_hidden: "Livello nascosto",
    status_new_canvas: "Nuovo foglio",
    opened_label: "Aperto",
    saved_label: "Salvato",

    confirm_title: "Modifiche non salvate",
    confirm_new: "Creare un nuovo foglio farà perdere le modifiche non salvate del foglio attuale.",
    confirm_open: "Aprire un'altra immagine farà perdere le modifiche non salvate del foglio attuale.",
    confirm_save: "Salva e continua",
    confirm_discard: "Scarta",
    confirm_cancel: "Annulla",

    settings: "Impostazioni",
    language: "Lingua",
    language_hint: "La scelta vale subito per tutta l'interfaccia.",
    close: "Chiudi",

    images_filter: "Immagini",
    png_filter: "Immagine PNG",
    default_file: "disegno.png",
    err_open: "Impossibile aprire l'immagine",
    err_empty: "L'immagine è vuota",
    err_save: "Salvataggio non riuscito",
};

const ENGLISH: Texts = Texts {
    lang: Lang::English,
    language_name: "English",

    new_canvas: "New",
    open: "Open…",
    save: "Save",
    save_as: "Save as…",
    undo: "↶ Undo",
    redo: "↷ Redo",
    color: "Color:",
    width: "Width:",
    strength: "Strength:",
    tool: "Tool:",
    palette: "Palette:",
    pick_color: "Use this color",
    tools: [
        "Pencil",
        "Eraser",
        "Line",
        "Rectangle",
        "Filled rectangle",
        "Ellipse",
        "Filled ellipse",
        "Paint bucket",
        "Liquify",
    ],
    tool_hints: [
        "Freehand stroke",
        "Erases the active layer (transparent)",
        "Straight line, with preview",
        "Empty rectangle, with preview",
        "Filled rectangle, with preview",
        "Empty ellipse, with preview",
        "Filled ellipse, with preview",
        "Fills the contiguous area of the same color",
        "Drag and warp the pixels of the active layer",
    ],

    layers: "Layers",
    layer_new: "New",
    layer_duplicate: "Duplicate",
    layer_delete: "Delete",
    layer_merge: "Merge",
    hint_add_layer: "Adds a transparent layer above the active one",
    hint_duplicate_layer: "Copies the active layer",
    hint_delete_layer: "Deletes the active layer",
    hint_move_up: "Move up",
    hint_move_down: "Move down",
    hint_merge_layer: "Merges the active layer into the one below",
    hint_visible: "Show or hide the layer",
    hint_select: "Select the layer",
    layer_opacity: "Layer opacity",
    background_layer: "Background",
    layer_word: "Layer",
    copy_suffix: "copy",

    unsaved: "unsaved",
    unsaved_changes: "● unsaved changes",
    status_ready: "Ready",
    status_undo: "Undone",
    status_redo: "Redone",
    status_filled: "Filled",
    status_stroke_done: "Stroke finished",
    status_new_layer: "New layer",
    status_layer_duplicated: "Layer duplicated",
    status_need_layer: "At least one layer is required",
    status_layer_deleted: "Layer deleted",
    status_layer_up: "Layer moved up",
    status_layer_down: "Layer moved down",
    status_nothing_to_merge: "No layer below to merge into",
    status_layers_merged: "Layers merged",
    status_layer_shown: "Layer shown",
    status_layer_hidden: "Layer hidden",
    status_new_canvas: "New canvas",
    opened_label: "Opened",
    saved_label: "Saved",

    confirm_title: "Unsaved changes",
    confirm_new: "Creating a new canvas will discard the unsaved changes in the current canvas.",
    confirm_open: "Opening another image will discard the unsaved changes in the current canvas.",
    confirm_save: "Save and continue",
    confirm_discard: "Discard",
    confirm_cancel: "Cancel",

    settings: "Settings",
    language: "Language",
    language_hint: "The choice applies to the whole interface right away.",
    close: "Close",

    images_filter: "Images",
    png_filter: "PNG image",
    default_file: "drawing.png",
    err_open: "Could not open the image",
    err_empty: "The image is empty",
    err_save: "Saving failed",
};
