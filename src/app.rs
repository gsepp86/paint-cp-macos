//! Finestra principale di PaintCP: barra degli strumenti, tavolozza, livelli e tela.

use std::path::PathBuf;

use eframe::egui::{
    self, Align, Color32, ColorImage, CornerRadius, CursorIcon, Key, KeyboardShortcut, Layout,
    Modifiers, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions,
    Vec2,
};

use crate::paint;

const CANVAS_W: usize = 1024;
const CANVAS_H: usize = 700;
const MAX_UNDO: usize = 20;
/// Limite di memoria per la cronologia (le tele grandi occupano molto).
const UNDO_BUDGET_BYTES: usize = 256 * 1024 * 1024;
const MAX_DIMENSION: usize = 4096;
const MAX_LAYERS: usize = 16;

const PALETTE: [Color32; 14] = [
    Color32::BLACK,
    Color32::from_rgb(127, 127, 127),
    Color32::from_rgb(136, 0, 21),
    Color32::from_rgb(237, 28, 36),
    Color32::from_rgb(255, 127, 39),
    Color32::from_rgb(255, 242, 0),
    Color32::from_rgb(34, 177, 76),
    Color32::from_rgb(0, 162, 232),
    Color32::from_rgb(63, 72, 204),
    Color32::from_rgb(163, 73, 164),
    Color32::from_rgb(185, 122, 87),
    Color32::from_rgb(255, 174, 201),
    Color32::from_rgb(195, 195, 195),
    Color32::WHITE,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Pencil,
    Eraser,
    Line,
    Rect,
    RectFilled,
    Ellipse,
    EllipseFilled,
    Fill,
    Liquify,
}

impl Tool {
    const ALL: [Tool; 9] = [
        Tool::Pencil,
        Tool::Eraser,
        Tool::Line,
        Tool::Rect,
        Tool::RectFilled,
        Tool::Ellipse,
        Tool::EllipseFilled,
        Tool::Fill,
        Tool::Liquify,
    ];

    fn label(self) -> &'static str {
        match self {
            Tool::Pencil => "Matita",
            Tool::Eraser => "Gomma",
            Tool::Line => "Linea",
            Tool::Rect => "Rettangolo",
            Tool::RectFilled => "Rettangolo pieno",
            Tool::Ellipse => "Ellisse",
            Tool::EllipseFilled => "Ellisse piena",
            Tool::Fill => "Secchio",
            Tool::Liquify => "Fluidifica",
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Tool::Pencil => "Tratto a mano libera",
            Tool::Eraser => "Cancella il livello attivo (trasparente)",
            Tool::Line => "Linea dritta, con anteprima",
            Tool::Rect => "Rettangolo vuoto, con anteprima",
            Tool::RectFilled => "Rettangolo pieno, con anteprima",
            Tool::Ellipse => "Ellisse vuota, con anteprima",
            Tool::EllipseFilled => "Ellisse piena, con anteprima",
            Tool::Fill => "Riempie l'area contigua dello stesso colore",
            Tool::Liquify => "Trascina e deforma i pixel del livello attivo",
        }
    }

    /// Le forme vengono ridisegnate da zero a ogni movimento (anteprima).
    fn is_shape(self) -> bool {
        matches!(
            self,
            Tool::Line | Tool::Rect | Tool::RectFilled | Tool::Ellipse | Tool::EllipseFilled
        )
    }
}

/// Un livello dell'immagine: pixel, visibilità e opacità.
#[derive(Clone)]
struct Layer {
    name: String,
    image: ColorImage,
    visible: bool,
    opacity: f32,
}

impl Layer {
    fn new(name: impl Into<String>, image: ColorImage) -> Self {
        Self {
            name: name.into(),
            image,
            visible: true,
            opacity: 1.0,
        }
    }
}

/// Attributi modificabili senza toccare i pixel (per la cronologia).
#[derive(Clone, Copy)]
struct LayerAttrs {
    opacity: f32,
    visible: bool,
}

/// Una modifica annullabile. Il `before` è lo stato precedente: annullando si
/// scambia con quello attuale, così la stessa voce serve anche per il "Ripeti".
enum Change {
    /// Pixel di un singolo livello prima di un tratto, di un riempimento o di
    /// una fluidificazione.
    Pixels { index: usize, before: ColorImage },
    /// Intero elenco dei livelli prima di una modifica strutturale (aggiunta,
    /// eliminazione, spostamento, unione).
    Stack { layers: Vec<Layer>, active: usize },
    /// Opacità e visibilità di un livello.
    Attrs { index: usize, before: LayerAttrs },
}

impl Change {
    fn bytes(&self) -> usize {
        match self {
            Change::Pixels { before, .. } => before.pixels.len() * std::mem::size_of::<Color32>(),
            Change::Stack { layers, .. } => layers
                .iter()
                .map(|layer| layer.image.pixels.len() * std::mem::size_of::<Color32>())
                .sum(),
            Change::Attrs { .. } => 64,
        }
    }
}

/// Tratto in corso: per le forme serve la copia del livello su cui ridisegnare
/// l'anteprima a ogni movimento del mouse.
struct DrawStroke {
    index: usize,
    start: Pos2,
    last: Pos2,
    base: Option<ColorImage>,
}

/// Posizione e scala con cui la tela è stata disegnata nell'ultimo frame.
#[derive(Clone, Copy)]
struct View {
    rect: Rect,
    scale: f32,
}

impl View {
    fn image_pos(&self, p: Pos2, clamp: bool) -> Option<Pos2> {
        if !clamp && !self.rect.contains(p) {
            return None;
        }
        let w = (self.rect.width() / self.scale).max(1.0);
        let h = (self.rect.height() / self.scale).max(1.0);
        let x = ((p.x - self.rect.min.x) / self.scale).clamp(0.0, w - 0.01);
        let y = ((p.y - self.rect.min.y) / self.scale).clamp(0.0, h - 0.01);
        Some(Pos2::new(x, y))
    }
}

/// Azione che può far perdere le modifiche non salvate.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    New,
    Open,
}

pub struct PaintApp {
    layers: Vec<Layer>,
    active: usize,
    layer_counter: usize,
    /// Unione dei livelli, usata per disegnare e per salvare.
    composite: ColorImage,
    has_transparency: bool,
    dirty: bool,
    texture_dirty: bool,
    texture: Option<TextureHandle>,
    tool: Tool,
    color: Color32,
    brush: f32,
    liquify_strength: f32,
    undo: Vec<Change>,
    redo: Vec<Change>,
    stroke: Option<DrawStroke>,
    view: Option<View>,
    path: Option<PathBuf>,
    modified: bool,
    pending: Option<PendingAction>,
    status: String,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        cc.egui_ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = Vec2::new(6.0, 4.0);
        });

        Self {
            layers: vec![Layer::new("Sfondo", paint::blank(CANVAS_W, CANVAS_H))],
            active: 0,
            layer_counter: 1,
            composite: paint::blank(CANVAS_W, CANVAS_H),
            has_transparency: false,
            dirty: true,
            texture_dirty: true,
            texture: None,
            tool: Tool::Pencil,
            color: Color32::BLACK,
            brush: 4.0,
            liquify_strength: 0.6,
            undo: Vec::new(),
            redo: Vec::new(),
            stroke: None,
            view: None,
            path: None,
            modified: false,
            pending: None,
            status: "Pronto".to_owned(),
        }
    }

    // --- Livelli e composizione ---------------------------------------------

    fn canvas_size(&self) -> [usize; 2] {
        self.layers
            .first()
            .map(|layer| layer.image.size)
            .unwrap_or([CANVAS_W, CANVAS_H])
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.texture_dirty = true;
        self.modified = true;
    }

    /// Ricostruisce l'immagine visibile unendo i livelli dal basso in alto.
    fn recomposite(&mut self) {
        let [w, h] = self.canvas_size();
        let count = w * h;
        if self.composite.size != [w, h] || self.composite.pixels.len() != count {
            self.composite = paint::transparent(w, h);
        }

        let mut visible = 0;
        let mut only = None;
        for (i, layer) in self.layers.iter().enumerate() {
            if layer.visible && layer.opacity > 0.0 {
                visible += 1;
                only = Some(i);
                if visible > 1 {
                    break;
                }
            }
        }

        // Percorso veloce: un solo livello visibile e completamente opaco.
        if visible <= 1 {
            match only {
                Some(i)
                    if self.layers[i].opacity >= 0.999 && self.layers[i].image.size == [w, h] =>
                {
                    self.composite
                        .pixels
                        .copy_from_slice(&self.layers[i].image.pixels);
                }
                _ => self.composite.pixels.fill(Color32::TRANSPARENT),
            }
        } else {
            self.composite.pixels.fill(Color32::TRANSPARENT);
            for layer in &self.layers {
                if !layer.visible || layer.opacity <= 0.0 || layer.image.size != [w, h] {
                    continue;
                }
                let opacity = layer.opacity;
                if opacity >= 0.999 {
                    for (dst, src) in self.composite.pixels.iter_mut().zip(&layer.image.pixels) {
                        *dst = over(*src, 1.0, *dst);
                    }
                } else {
                    for (dst, src) in self.composite.pixels.iter_mut().zip(&layer.image.pixels) {
                        *dst = over(*src, opacity, *dst);
                    }
                }
            }
        }

        self.has_transparency = self.composite.pixels.iter().any(|p| p.a() < 255);
    }

    fn ensure_composite(&mut self) {
        if self.dirty {
            self.recomposite();
            self.dirty = false;
        }
    }

    // --- Cronologia ---------------------------------------------------------

    fn record(&mut self, change: Change) {
        self.undo.push(change);
        self.redo.clear();
        Self::trim(&mut self.undo);
    }

    fn trim(history: &mut Vec<Change>) {
        let mut total: usize = history.iter().map(Change::bytes).sum();
        while history.len() > 1 && (history.len() > MAX_UNDO || total > UNDO_BUDGET_BYTES) {
            total -= history.remove(0).bytes();
        }
    }

    /// Applica una modifica e restituisce quella inversa (per l'altra pila).
    fn apply(&mut self, change: Change) -> Option<Change> {
        match change {
            Change::Pixels { index, before } => {
                let layer = self.layers.get_mut(index)?;
                let current = std::mem::replace(&mut layer.image, before);
                Some(Change::Pixels {
                    index,
                    before: current,
                })
            }
            Change::Stack { layers, active } => {
                let current = std::mem::replace(&mut self.layers, layers);
                let previous_active = std::mem::replace(&mut self.active, active);
                self.active = self.active.min(self.layers.len().saturating_sub(1));
                Some(Change::Stack {
                    layers: current,
                    active: previous_active,
                })
            }
            Change::Attrs { index, before } => {
                let layer = self.layers.get_mut(index)?;
                let current = LayerAttrs {
                    opacity: layer.opacity,
                    visible: layer.visible,
                };
                layer.opacity = before.opacity;
                layer.visible = before.visible;
                Some(Change::Attrs {
                    index,
                    before: current,
                })
            }
        }
    }

    fn undo(&mut self) {
        let Some(change) = self.undo.pop() else {
            return;
        };
        if let Some(inverse) = self.apply(change) {
            self.redo.push(inverse);
            Self::trim(&mut self.redo);
        }
        self.stroke = None;
        self.mark_dirty();
        self.status = "Annullato".to_owned();
    }

    fn redo(&mut self) {
        let Some(change) = self.redo.pop() else {
            return;
        };
        if let Some(inverse) = self.apply(change) {
            self.undo.push(inverse);
            Self::trim(&mut self.undo);
        }
        self.stroke = None;
        self.mark_dirty();
        self.status = "Ripetuto".to_owned();
    }

    /// Copia dello stato dei livelli, per le modifiche strutturali.
    fn stack_snapshot(&self) -> Change {
        Change::Stack {
            layers: self.layers.clone(),
            active: self.active,
        }
    }

    // --- Disegno ------------------------------------------------------------

    fn begin_stroke(&mut self, at: Pos2) {
        let index = self.active;
        let Some(layer) = self.layers.get(index) else {
            return;
        };
        let before = layer.image.clone();
        self.record(Change::Pixels { index, before });

        if self.tool == Tool::Fill {
            if let Some(layer) = self.layers.get_mut(index) {
                paint::flood_fill(&mut layer.image, at, self.color);
            }
            self.mark_dirty();
            self.status = "Riempimento".to_owned();
            return;
        }

        let base = if self.tool.is_shape() {
            self.layers.get(index).map(|layer| layer.image.clone())
        } else {
            None
        };

        self.stroke = Some(DrawStroke {
            index,
            start: at,
            last: at,
            base,
        });
        self.paint_stroke(at);
    }

    fn continue_stroke(&mut self, at: Pos2) {
        if self.stroke.is_some() {
            self.paint_stroke(at);
        }
    }

    fn paint_stroke(&mut self, at: Pos2) {
        let Some(mut stroke) = self.stroke.take() else {
            return;
        };

        let tool = self.tool;
        let color = self.color;
        let radius = (self.brush * 0.5).max(0.5);
        let liquify_radius = self.brush.max(6.0);
        let strength = self.liquify_strength;

        if let Some(layer) = self.layers.get_mut(stroke.index) {
            let image = &mut layer.image;
            if let Some(base) = &stroke.base {
                image.pixels.copy_from_slice(&base.pixels);
            }

            let (start, last) = (stroke.start, stroke.last);
            match tool {
                Tool::Pencil => paint::line(image, last, at, radius, color),
                Tool::Eraser => paint::erase_line(image, last, at, radius),
                Tool::Line => paint::line(image, start, at, radius, color),
                Tool::Rect => paint::rect(image, start, at, false, radius, color),
                Tool::RectFilled => paint::rect(image, start, at, true, radius, color),
                Tool::Ellipse => paint::ellipse(image, start, at, false, radius, color),
                Tool::EllipseFilled => paint::ellipse(image, start, at, true, radius, color),
                Tool::Liquify => paint::liquify(image, last, at, liquify_radius, strength),
                Tool::Fill => {}
            }

            stroke.last = at;
            self.mark_dirty();
        }

        self.stroke = Some(stroke);
    }

    fn end_stroke(&mut self) {
        if self.stroke.take().is_some() {
            self.status = "Disegno completato".to_owned();
        }
    }

    // --- Operazioni sui livelli --------------------------------------------

    fn next_layer_name(&mut self) -> String {
        self.layer_counter += 1;
        format!("Livello {}", self.layer_counter)
    }

    fn add_layer(&mut self) {
        if self.layers.len() >= MAX_LAYERS {
            self.status = format!("Massimo {MAX_LAYERS} livelli");
            return;
        }
        self.record(self.stack_snapshot());
        let [w, h] = self.canvas_size();
        let name = self.next_layer_name();
        let index = self.active + 1;
        self.layers
            .insert(index, Layer::new(name, paint::transparent(w, h)));
        self.active = index;
        self.mark_dirty();
        self.status = "Nuovo livello".to_owned();
    }

    fn duplicate_layer(&mut self) {
        if self.layers.len() >= MAX_LAYERS {
            self.status = format!("Massimo {MAX_LAYERS} livelli");
            return;
        }
        let Some(source) = self.layers.get(self.active) else {
            return;
        };
        let copy = Layer {
            name: format!("{} copia", source.name),
            image: source.image.clone(),
            visible: source.visible,
            opacity: source.opacity,
        };
        self.record(self.stack_snapshot());
        let index = self.active + 1;
        self.layers.insert(index, copy);
        self.active = index;
        self.mark_dirty();
        self.status = "Livello duplicato".to_owned();
    }

    fn delete_layer(&mut self) {
        if self.layers.len() <= 1 {
            self.status = "Serve almeno un livello".to_owned();
            return;
        }
        self.record(self.stack_snapshot());
        self.layers.remove(self.active);
        self.active = self.active.min(self.layers.len() - 1);
        self.mark_dirty();
        self.status = "Livello eliminato".to_owned();
    }

    fn move_layer(&mut self, offset: isize) {
        let target = self.active as isize + offset;
        if target < 0 || target as usize >= self.layers.len() {
            return;
        }
        self.record(self.stack_snapshot());
        self.layers.swap(self.active, target as usize);
        self.active = target as usize;
        self.mark_dirty();
        self.status = if offset > 0 {
            "Livello spostato sopra".to_owned()
        } else {
            "Livello spostato sotto".to_owned()
        };
    }

    /// Unisce il livello attivo in quello sottostante.
    fn merge_down(&mut self) {
        if self.active == 0 {
            self.status = "Nessun livello sotto da unire".to_owned();
            return;
        }
        self.record(self.stack_snapshot());
        let index = self.active;
        let upper = self.layers[index].clone();
        let lower = &mut self.layers[index - 1];
        for (dst, src) in lower.image.pixels.iter_mut().zip(&upper.image.pixels) {
            *dst = over(*src, upper.opacity, *dst);
        }
        lower.visible = true;
        lower.opacity = 1.0;
        self.layers.remove(index);
        self.active = index - 1;
        self.mark_dirty();
        self.status = "Livelli uniti".to_owned();
    }

    fn set_layer_visible(&mut self, index: usize, visible: bool) {
        let Some(layer) = self.layers.get_mut(index) else {
            return;
        };
        let before = LayerAttrs {
            opacity: layer.opacity,
            visible: layer.visible,
        };
        if before.visible == visible {
            return;
        }
        self.record(Change::Attrs { index, before });
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = visible;
        }
        self.mark_dirty();
        self.status = if visible {
            "Livello visibile".to_owned()
        } else {
            "Livello nascosto".to_owned()
        };
    }

    // --- Foglio e file -----------------------------------------------------

    fn request(&mut self, action: PendingAction) {
        if self.modified {
            self.pending = Some(action);
        } else {
            self.perform(action);
        }
    }

    fn perform(&mut self, action: PendingAction) {
        match action {
            PendingAction::New => self.new_canvas(),
            PendingAction::Open => self.open(),
        }
    }

    fn new_canvas(&mut self) {
        self.layers = vec![Layer::new("Sfondo", paint::blank(CANVAS_W, CANVAS_H))];
        self.active = 0;
        self.layer_counter = 1;
        self.undo.clear();
        self.redo.clear();
        self.stroke = None;
        self.path = None;
        self.mark_dirty();
        self.modified = false;
        self.status = "Nuovo foglio".to_owned();
    }

    fn open(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Immagini", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
        else {
            return;
        };

        match load_image(&path) {
            Ok(image) => {
                self.layers = vec![Layer::new("Sfondo", image)];
                self.active = 0;
                self.layer_counter = 1;
                self.undo.clear();
                self.redo.clear();
                self.stroke = None;
                self.path = Some(path.clone());
                self.mark_dirty();
                self.modified = false;
                self.status = format!("Aperto: {}", file_name(&path));
            }
            Err(message) => self.status = message,
        }
    }

    /// Salva l'unione dei livelli in PNG. `true` se il file è stato scritto.
    fn save(&mut self, save_as: bool) -> bool {
        self.ensure_composite();

        let path = if save_as { None } else { self.path.clone() };
        let path = path.or_else(|| {
            rfd::FileDialog::new()
                .add_filter("Immagine PNG", &["png"])
                .set_file_name(self.suggested_name())
                .save_file()
        });
        let Some(path) = path else {
            return false;
        };

        // Il salvataggio è sempre PNG: se manca o è sbagliata, si corregge l'estensione.
        let is_png = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
        let path = if is_png {
            path
        } else {
            path.with_extension("png")
        };

        match save_png(&self.composite, &path) {
            Ok(()) => {
                self.status = format!("Salvato: {}", file_name(&path));
                self.path = Some(path);
                self.modified = false;
                true
            }
            Err(message) => {
                self.status = message;
                false
            }
        }
    }

    fn suggested_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "disegno.png".to_owned())
    }

    // --- Interfaccia -------------------------------------------------------

    fn shortcuts(&mut self, ctx: &egui::Context) {
        let cmd = Modifiers::COMMAND;
        let shift_cmd = cmd | Modifiers::SHIFT;

        // Le scorciatoie più specifiche vanno controllate per prime: egui
        // ignora Shift in più quando confronta i modificatori.
        let (redo, save_as, save, open, undo, new) = ctx.input_mut(|i| {
            (
                i.consume_shortcut(&KeyboardShortcut::new(shift_cmd, Key::Z))
                    || i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::Y)),
                i.consume_shortcut(&KeyboardShortcut::new(shift_cmd, Key::S)),
                i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::S)),
                i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::O)),
                i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::Z)),
                i.consume_shortcut(&KeyboardShortcut::new(cmd, Key::N)),
            )
        });

        if redo {
            self.redo();
        } else if undo {
            self.undo();
        }
        if save_as {
            self.save(true);
        } else if save {
            self.save(false);
        }
        if open {
            self.request(PendingAction::Open);
        }
        if new {
            self.request(PendingAction::New);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Nuovo").clicked() {
                self.request(PendingAction::New);
            }
            if ui.button("Apri…").clicked() {
                self.request(PendingAction::Open);
            }
            if ui.button("Salva").clicked() {
                self.save(false);
            }
            if ui.button("Salva con nome…").clicked() {
                self.save(true);
            }

            ui.separator();

            if ui
                .add_enabled(!self.undo.is_empty(), egui::Button::new("↶ Annulla"))
                .clicked()
            {
                self.undo();
            }
            if ui
                .add_enabled(!self.redo.is_empty(), egui::Button::new("↷ Ripeti"))
                .clicked()
            {
                self.redo();
            }

            ui.separator();

            ui.label("Colore:");
            ui.color_edit_button_srgba(&mut self.color);

            ui.add_space(4.0);
            ui.label("Spessore:");
            ui.add(
                egui::Slider::new(&mut self.brush, 1.0..=48.0)
                    .integer()
                    .suffix(" px"),
            );

            if self.tool == Tool::Liquify {
                ui.add_space(4.0);
                ui.label("Intensità:");
                ui.add(egui::Slider::new(&mut self.liquify_strength, 0.1..=1.0).max_decimals(2));
            }
        });

        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Strumento:");
            for tool in Tool::ALL {
                if ui
                    .selectable_label(self.tool == tool, tool.label())
                    .on_hover_text(tool.hint())
                    .clicked()
                {
                    self.tool = tool;
                }
            }
        });

        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Tavolozza:");
            for color in PALETTE {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
                let painter = ui.painter();
                painter.rect_filled(rect.shrink(1.0), CornerRadius::same(3), color);
                let border = if self.color == color {
                    Stroke::new(2.0, Color32::from_rgb(0, 120, 215))
                } else {
                    Stroke::new(1.0, Color32::from_gray(120))
                };
                painter.rect_stroke(
                    rect.shrink(1.0),
                    CornerRadius::same(3),
                    border,
                    StrokeKind::Inside,
                );
                if response.on_hover_text("Usa questo colore").clicked() {
                    self.color = color;
                }
            }
        });
        ui.add_space(2.0);
    }

    fn layers_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(RichText::new("Livelli").strong());
        ui.add_space(2.0);

        ui.horizontal_wrapped(|ui| {
            let can_add = self.layers.len() < MAX_LAYERS;
            if ui
                .add_enabled(can_add, egui::Button::new("Nuovo"))
                .on_hover_text("Aggiunge un livello trasparente sopra quello attivo")
                .clicked()
            {
                self.add_layer();
            }
            if ui
                .add_enabled(can_add, egui::Button::new("Duplica"))
                .on_hover_text("Copia il livello attivo")
                .clicked()
            {
                self.duplicate_layer();
            }
            if ui
                .add_enabled(self.layers.len() > 1, egui::Button::new("Elimina"))
                .on_hover_text("Elimina il livello attivo")
                .clicked()
            {
                self.delete_layer();
            }
            if ui
                .add_enabled(
                    self.active + 1 < self.layers.len(),
                    egui::Button::new("▲").small(),
                )
                .on_hover_text("Sposta sopra")
                .clicked()
            {
                self.move_layer(1);
            }
            if ui
                .add_enabled(self.active > 0, egui::Button::new("▼").small())
                .on_hover_text("Sposta sotto")
                .clicked()
            {
                self.move_layer(-1);
            }
            if ui
                .add_enabled(self.active > 0, egui::Button::new("Unisci"))
                .on_hover_text("Unisce il livello attivo a quello sotto")
                .clicked()
            {
                self.merge_down();
            }
        });

        ui.separator();

        let mut select: Option<usize> = None;
        let mut toggle: Option<(usize, bool)> = None;
        let list_height = (ui.available_height() - 62.0).max(80.0);

        egui::ScrollArea::vertical()
            .max_height(list_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Il livello in cima all'elenco è quello più in alto nella pila.
                for index in (0..self.layers.len()).rev() {
                    let layer = &self.layers[index];
                    ui.horizontal(|ui| {
                        let mut visible = layer.visible;
                        if ui
                            .checkbox(&mut visible, "")
                            .on_hover_text("Mostra o nascondi il livello")
                            .changed()
                        {
                            toggle = Some((index, visible));
                        }
                        if ui
                            .selectable_label(index == self.active, &layer.name)
                            .on_hover_text("Seleziona il livello")
                            .clicked()
                        {
                            select = Some(index);
                        }
                        if layer.opacity < 0.999 {
                            let _ = ui.weak(format!("{}%", (layer.opacity * 100.0).round() as i32));
                        }
                    });
                }
            });

        if let Some(index) = select {
            self.active = index;
        }
        if let Some((index, visible)) = toggle {
            self.set_layer_visible(index, visible);
        }

        ui.separator();
        let index = self.active;
        if let Some(layer) = self.layers.get(index) {
            let before = LayerAttrs {
                opacity: layer.opacity,
                visible: layer.visible,
            };
            let mut percent = before.opacity * 100.0;
            let response = ui.add(
                egui::Slider::new(&mut percent, 0.0..=100.0)
                    .integer()
                    .suffix(" %"),
            );
            let _ = ui.weak("Opacità del livello");
            if response.changed() {
                if response.drag_started() || !response.dragged() {
                    self.record(Change::Attrs { index, before });
                }
                if let Some(layer) = self.layers.get_mut(index) {
                    layer.opacity = percent / 100.0;
                }
                self.mark_dirty();
                self.status = format!("Opacità {}%", percent.round() as i32);
            }
        }
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let [w, h] = self.canvas_size();
        ui.horizontal(|ui| {
            ui.label(format!("{w} × {h} px"));
            ui.separator();
            let layer_name = self
                .layers
                .get(self.active)
                .map(|layer| layer.name.as_str())
                .unwrap_or("-");
            ui.label(format!("Livello: {layer_name}"));
            ui.separator();
            ui.label(format!("Strumento: {}", self.tool.label()));

            if let (Some(view), Some(pointer)) =
                (self.view, ui.ctx().input(|i| i.pointer.hover_pos()))
            {
                if let Some(p) = view.image_pos(pointer, false) {
                    ui.separator();
                    ui.label(format!("x: {}, y: {}", p.x as i32, p.y as i32));
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(path) = &self.path {
                    ui.label(file_name(path));
                } else {
                    ui.weak("non salvato");
                }
                ui.separator();
                ui.label(&self.status);
                if self.modified {
                    ui.separator();
                    ui.colored_label(Color32::from_rgb(190, 90, 0), "● modifiche non salvate");
                }
            });
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        if self.dirty {
            self.recomposite();
            self.dirty = false;
        }

        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());

        if self.texture_dirty || self.texture.is_none() {
            let image = self.composite.clone();
            match &mut self.texture {
                Some(texture) => texture.set(image, TextureOptions::NEAREST),
                None => {
                    self.texture = Some(ui.ctx().load_texture(
                        "tela",
                        image,
                        TextureOptions::NEAREST,
                    ))
                }
            }
            self.texture_dirty = false;
        }

        let [w, h] = self.canvas_size();
        let image_size = Vec2::new(w as f32, h as f32);
        let scale = (response.rect.width() / image_size.x)
            .min(response.rect.height() / image_size.y)
            .clamp(0.02, 1.0);
        let image_rect = Rect::from_center_size(response.rect.center(), image_size * scale);
        let view = View {
            rect: image_rect,
            scale,
        };
        self.view = Some(view);

        painter.rect_filled(response.rect, CornerRadius::ZERO, Color32::from_gray(60));

        // Scacchiera dietro la tela: si vede solo dove i livelli sono trasparenti.
        if self.has_transparency {
            let cell = (16.0 * scale).max(6.0);
            let light = Color32::from_gray(235);
            let dark = Color32::from_gray(205);
            painter.rect_filled(image_rect, CornerRadius::ZERO, light);
            let columns = (image_rect.width() / cell).ceil() as usize;
            let rows = (image_rect.height() / cell).ceil() as usize;
            for row in 0..rows {
                for column in (row % 2..columns).step_by(2) {
                    let min = Pos2::new(
                        image_rect.min.x + column as f32 * cell,
                        image_rect.min.y + row as f32 * cell,
                    );
                    let rect = Rect::from_min_size(min, Vec2::splat(cell)).intersect(image_rect);
                    painter.rect_filled(rect, CornerRadius::ZERO, dark);
                }
            }
        }

        if let Some(texture) = &self.texture {
            painter.image(
                texture.id(),
                image_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        painter.rect_stroke(
            image_rect,
            CornerRadius::ZERO,
            Stroke::new(1.0, Color32::from_gray(20)),
            StrokeKind::Outside,
        );

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
        }

        let pointer_down = response.is_pointer_button_down_on();

        if pointer_down && self.stroke.is_none() && self.tool != Tool::Fill {
            if let Some(point) = response
                .interact_pointer_pos()
                .and_then(|p| view.image_pos(p, false))
            {
                self.begin_stroke(point);
            }
        } else if pointer_down {
            if let Some(point) = response
                .interact_pointer_pos()
                .and_then(|p| view.image_pos(p, true))
            {
                self.continue_stroke(point);
            }
        } else if self.stroke.is_some() {
            self.end_stroke();
        }

        if self.tool == Tool::Fill && response.clicked() {
            if let Some(point) = response
                .interact_pointer_pos()
                .and_then(|p| view.image_pos(p, false))
            {
                self.begin_stroke(point);
            }
        }
    }

    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = self.pending else {
            return;
        };

        let mut choice: Option<u8> = None;
        let response =
            egui::Modal::new(egui::Id::unique("modifiche_non_salvate")).show(ctx, |ui| {
                ui.set_width(340.0);
                ui.label(RichText::new("Modifiche non salvate").strong());
                ui.add_space(4.0);
                let action = match pending {
                    PendingAction::New => "Creare un nuovo foglio",
                    PendingAction::Open => "Aprire un'altra immagine",
                };
                ui.label(format!(
                    "{action} farà perdere le modifiche non salvate del foglio attuale."
                ));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Salva e continua").clicked() {
                        choice = Some(0);
                    }
                    if ui.button("Scarta").clicked() {
                        choice = Some(1);
                    }
                    if ui.button("Annulla").clicked() {
                        choice = Some(2);
                    }
                });
            });

        if choice.is_none() && response.should_close() {
            choice = Some(2);
        }

        match choice {
            Some(0) => {
                if self.save(false) {
                    self.perform(pending);
                    self.pending = None;
                }
            }
            Some(1) => {
                self.perform(pending);
                self.pending = None;
            }
            Some(_) => self.pending = None,
            None => {}
        }
    }
}

/// Fusione premoltiplicata di `src` (con opacità `opacity`) sopra `dst`.
#[inline]
fn over(src: Color32, opacity: f32, dst: Color32) -> Color32 {
    if src.a() == 0 || opacity <= 0.0 {
        return dst;
    }
    let opacity = opacity.clamp(0.0, 1.0);
    if opacity >= 1.0 && src.a() == 255 {
        return src;
    }

    let alpha = src.a() as f32 / 255.0 * opacity;
    let keep = 1.0 - alpha;
    #[inline]
    fn channel(src: u8, dst: u8, opacity: f32, keep: f32) -> u8 {
        (src as f32 * opacity + dst as f32 * keep)
            .round()
            .clamp(0.0, 255.0) as u8
    }
    Color32::from_rgba_premultiplied(
        channel(src.r(), dst.r(), opacity, keep),
        channel(src.g(), dst.g(), opacity, keep),
        channel(src.b(), dst.b(), opacity, keep),
        channel(src.a(), dst.a(), opacity, keep),
    )
}

impl eframe::App for PaintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shortcuts(ui.ctx());

        egui::Panel::top("barra_strumenti").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("barra_stato").show(ui, |ui| self.status_bar(ui));
        egui::Panel::right("livelli")
            .default_size(215.0)
            .min_size(170.0)
            .resizable(true)
            .show(ui, |ui| self.layers_panel(ui));
        egui::CentralPanel::default().show(ui, |ui| self.canvas(ui));

        self.confirm_dialog(ui.ctx());
    }
}

// --- Supporto file ----------------------------------------------------------

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn load_image(path: &std::path::Path) -> Result<ColorImage, String> {
    let decoded = image::open(path).map_err(|e| format!("Impossibile aprire l'immagine: {e}"))?;
    let rgba = decoded.to_rgba8();
    let (width, height) = (rgba.width() as usize, rgba.height() as usize);

    if width == 0 || height == 0 {
        return Err("L'immagine è vuota".to_owned());
    }
    if width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err(format!(
            "Immagine troppo grande ({width} × {height} px, massimo {MAX_DIMENSION} px per lato)"
        ));
    }

    let pixels = rgba
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(ColorImage::new([width, height], pixels))
}

fn save_png(image: &ColorImage, path: &std::path::Path) -> Result<(), String> {
    let [w, h] = image.size;
    let mut buffer = Vec::with_capacity(w * h * 4);
    for pixel in &image.pixels {
        buffer.extend_from_slice(&pixel.to_srgba_unmultiplied());
    }

    image::save_buffer(
        path,
        &buffer,
        w as u32,
        h as u32,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| format!("Salvataggio non riuscito: {e}"))
}
