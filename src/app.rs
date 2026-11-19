//! Finestra principale di PaintCP: barra degli strumenti, tavolozza e tela.

use std::path::PathBuf;

use eframe::egui::{
    self, Align, Color32, ColorImage, CornerRadius, CursorIcon, Key, KeyboardShortcut, Layout,
    Modifiers, Pos2, Rect, Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, Vec2,
};

use crate::paint;

const CANVAS_W: usize = 1024;
const CANVAS_H: usize = 700;
const MAX_UNDO: usize = 12;
/// Limite di memoria per la cronologia (le tele grandi occupano molto).
const UNDO_BUDGET_BYTES: usize = 256 * 1024 * 1024;
const MAX_DIMENSION: usize = 4096;

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
}

impl Tool {
    const ALL: [Tool; 8] = [
        Tool::Pencil,
        Tool::Eraser,
        Tool::Line,
        Tool::Rect,
        Tool::RectFilled,
        Tool::Ellipse,
        Tool::EllipseFilled,
        Tool::Fill,
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
        }
    }

    fn is_shape(self) -> bool {
        matches!(
            self,
            Tool::Line | Tool::Rect | Tool::RectFilled | Tool::Ellipse | Tool::EllipseFilled
        )
    }
}

/// Tratto in corso: per le forme serve la copia della tela su cui ridisegnare
/// l'anteprima a ogni movimento del mouse.
struct DrawStroke {
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
    fn to_image(&self, p: Pos2, clamp: bool) -> Option<Pos2> {
        if !clamp && !self.rect.contains(p) {
            return None;
        }
        let [w, h] = [self.rect.width() / self.scale, self.rect.height() / self.scale];
        let x = ((p.x - self.rect.min.x) / self.scale).clamp(0.0, w - 0.01);
        let y = ((p.y - self.rect.min.y) / self.scale).clamp(0.0, h - 0.01);
        Some(Pos2::new(x, y))
    }
}

pub struct PaintApp {
    image: ColorImage,
    texture: Option<TextureHandle>,
    texture_dirty: bool,
    tool: Tool,
    color: Color32,
    brush: f32,
    undo: Vec<ColorImage>,
    redo: Vec<ColorImage>,
    stroke: Option<DrawStroke>,
    view: Option<View>,
    path: Option<PathBuf>,
    status: String,
}

impl PaintApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        cc.egui_ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = Vec2::new(6.0, 4.0);
        });

        Self {
            image: paint::blank(CANVAS_W, CANVAS_H),
            texture: None,
            texture_dirty: true,
            tool: Tool::Pencil,
            color: Color32::BLACK,
            brush: 4.0,
            undo: Vec::new(),
            redo: Vec::new(),
            stroke: None,
            view: None,
            path: None,
            status: "Pronto".to_owned(),
        }
    }

    // --- Modifica della tela ------------------------------------------------

    fn push_undo(&mut self) {
        self.undo.push(self.image.clone());
        self.redo.clear();

        let bytes = |snapshot: &ColorImage| snapshot.pixels.len() * std::mem::size_of::<Color32>();
        let mut total: usize = self.undo.iter().map(bytes).sum();
        while self.undo.len() > 1 && (self.undo.len() > MAX_UNDO || total > UNDO_BUDGET_BYTES) {
            total -= bytes(&self.undo.remove(0));
        }
    }

    fn begin_stroke(&mut self, at: Pos2) {
        if self.tool == Tool::Fill {
            self.push_undo();
            paint::flood_fill(&mut self.image, at, self.color);
            self.texture_dirty = true;
            self.status = "Riempimento".to_owned();
            return;
        }

        self.push_undo();
        let base = self.tool.is_shape().then(|| self.image.clone());
        self.stroke = Some(DrawStroke {
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
        let tool = self.tool;
        let color = if tool == Tool::Eraser {
            paint::PAPER
        } else {
            self.color
        };
        let radius = (self.brush * 0.5).max(0.5);
        let image = &mut self.image;

        let Some(stroke) = self.stroke.as_mut() else {
            return;
        };

        if let Some(base) = &stroke.base {
            image.pixels.copy_from_slice(&base.pixels);
        }

        let start = stroke.start;
        match tool {
            Tool::Pencil | Tool::Eraser => paint::line(image, stroke.last, at, radius, color),
            Tool::Line => paint::line(image, start, at, radius, color),
            Tool::Rect => paint::rect(image, start, at, false, radius, color),
            Tool::RectFilled => paint::rect(image, start, at, true, radius, color),
            Tool::Ellipse => paint::ellipse(image, start, at, false, radius, color),
            Tool::EllipseFilled => paint::ellipse(image, start, at, true, radius, color),
            Tool::Fill => {}
        }

        stroke.last = at;
        self.texture_dirty = true;
    }

    fn end_stroke(&mut self) {
        if self.stroke.take().is_some() {
            self.status = "Disegno completato".to_owned();
        }
    }

    fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.image, previous));
            if self.redo.len() > MAX_UNDO {
                self.redo.remove(0);
            }
            self.texture_dirty = true;
            self.stroke = None;
            self.status = "Annullato".to_owned();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.image, next));
            self.texture_dirty = true;
            self.stroke = None;
            self.status = "Ripetuto".to_owned();
        }
    }

    fn new_canvas(&mut self) {
        self.image = paint::blank(CANVAS_W, CANVAS_H);
        self.undo.clear();
        self.redo.clear();
        self.stroke = None;
        self.path = None;
        self.texture_dirty = true;
        self.status = "Nuovo foglio".to_owned();
    }

    // --- File ---------------------------------------------------------------

    fn open(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Immagini", &["png", "jpg", "jpeg", "bmp"])
            .pick_file()
        else {
            return;
        };

        match load_image(&path) {
            Ok(image) => {
                self.image = image;
                self.texture_dirty = true;
                self.undo.clear();
                self.redo.clear();
                self.stroke = None;
                self.path = Some(path.clone());
                self.status = format!("Aperto: {}", file_name(&path));
            }
            Err(message) => self.status = message,
        }
    }

    fn save(&mut self, save_as: bool) {
        let path = if save_as { None } else { self.path.clone() };
        let path = path.or_else(|| {
            rfd::FileDialog::new()
                .add_filter("Immagine PNG", &["png"])
                .set_file_name(self.suggested_name())
                .save_file()
        });
        let Some(path) = path else {
            return;
        };

        // Il salvataggio è sempre PNG: se manca o è sbagliata, si corregge l'estensione.
        let is_png = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
        let path = if is_png { path } else { path.with_extension("png") };

        match save_png(&self.image, &path) {
            Ok(()) => {
                self.status = format!("Salvato: {}", file_name(&path));
                self.path = Some(path);
            }
            Err(message) => self.status = message,
        }
    }

    fn suggested_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "disegno.png".to_owned())
    }

    // --- Interfaccia --------------------------------------------------------

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
            self.open();
        }
        if new {
            self.new_canvas();
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Nuovo").clicked() {
                self.new_canvas();
            }
            if ui.button("Apri…").clicked() {
                self.open();
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
        });

        ui.add_space(2.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Strumento:");
            for tool in Tool::ALL {
                if ui
                    .selectable_label(self.tool == tool, tool.label())
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
                painter.rect_stroke(rect.shrink(1.0), CornerRadius::same(3), border, StrokeKind::Inside);
                if response.on_hover_text("Usa questo colore").clicked() {
                    self.color = color;
                }
            }
        });
        ui.add_space(2.0);
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        let [w, h] = self.image.size;
        ui.horizontal(|ui| {
            ui.label(format!("{w} × {h} px"));
            ui.separator();
            ui.label(format!("Strumento: {}", self.tool.label()));

            if let (Some(view), Some(pointer)) = (
                self.view,
                ui.ctx().input(|i| i.pointer.hover_pos()),
            ) {
                if let Some(p) = view.to_image(pointer, false) {
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
            });
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, Sense::click_and_drag());

        if self.texture_dirty {
            let image = self.image.clone();
            match &mut self.texture {
                Some(texture) => texture.set(image, TextureOptions::NEAREST),
                None => {
                    self.texture =
                        Some(ui.ctx().load_texture("tela", image, TextureOptions::NEAREST))
                }
            }
            self.texture_dirty = false;
        }

        let [w, h] = self.image.size;
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
                .and_then(|p| view.to_image(p, false))
            {
                self.begin_stroke(point);
            }
        } else if pointer_down {
            if let Some(point) = response
                .interact_pointer_pos()
                .and_then(|p| view.to_image(p, true))
            {
                self.continue_stroke(point);
            }
        } else if self.stroke.is_some() {
            self.end_stroke();
        }

        if self.tool == Tool::Fill && response.clicked() {
            if let Some(point) = response
                .interact_pointer_pos()
                .and_then(|p| view.to_image(p, false))
            {
                self.begin_stroke(point);
            }
        }
    }
}

impl eframe::App for PaintApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.shortcuts(ui.ctx());

        egui::Panel::top("barra_strumenti").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("barra_stato").show(ui, |ui| self.status_bar(ui));
        egui::CentralPanel::default().show(ui, |ui| self.canvas(ui));
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
