//! Operazioni di disegno su una bitmap RGBA (`ColorImage`).
//!
//! Tutte le coordinate sono in pixel dell'immagine, con l'origine in alto a
//! sinistra. I colori sono premoltiplicati (come in `Color32`), quindi la
//! fusione dei pixel è `src + dst * (1 - a)`.
//!
//! Le linee e i bordi delle forme sono disegnati con dei timbri circolari
//! sovrapposti e con un bordo morbido di un pixel, così il risultato è
//! fluido come in Paint.

use eframe::egui::{Color32, ColorImage, Pos2};

/// Colore del foglio appena creato.
pub const PAPER: Color32 = Color32::WHITE;

/// Superficie bianca (lo "Sfondo" del foglio).
pub fn blank(width: usize, height: usize) -> ColorImage {
    ColorImage::filled([width.max(1), height.max(1)], PAPER)
}

/// Superficie completamente trasparente (un livello nuovo).
pub fn transparent(width: usize, height: usize) -> ColorImage {
    ColorImage::filled([width.max(1), height.max(1)], Color32::TRANSPARENT)
}

#[inline]
fn index(img: &ColorImage, x: i32, y: i32) -> Option<usize> {
    let [w, h] = img.size;
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        None
    } else {
        Some(y as usize * w + x as usize)
    }
}

/// Rettangolo (estremi esclusi) che contiene il cerchio `(cx, cy, r)`.
fn circle_bounds(cx: f32, cy: f32, r: f32, size: [usize; 2]) -> Option<(i32, i32, i32, i32)> {
    let [w, h] = size;
    let x0 = ((cx - r).floor() as i64).max(0) as i32;
    let y0 = ((cy - r).floor() as i64).max(0) as i32;
    let x1 = ((cx + r).ceil() as i64).min(w as i64) as i32;
    let y1 = ((cy + r).ceil() as i64).min(h as i64) as i32;
    (x1 > x0 && y1 > y0).then_some((x0, y0, x1, y1))
}

/// Intervallo di pixel interi coperto da `v0..v1`, almeno un pixel.
fn span(v0: f32, v1: f32, limit: usize) -> (i32, i32) {
    let start = (v0.min(v1).floor() as i64).max(0);
    let end = (v0.max(v1).ceil() as i64).max(start + 1).min(limit as i64);
    (start as i32, end as i32)
}

#[inline]
fn blend_px(img: &mut ColorImage, x: i32, y: i32, color: Color32, coverage: f32) {
    if coverage <= 0.0 {
        return;
    }
    let Some(idx) = index(img, x, y) else {
        return;
    };
    if coverage >= 1.0 {
        img.pixels[idx] = color;
        return;
    }
    let dst = img.pixels[idx];
    let alpha = color.a() as f32 / 255.0 * coverage;
    let keep = 1.0 - alpha;
    img.pixels[idx] = Color32::from_rgba_premultiplied(
        (color.r() as f32 * coverage + dst.r() as f32 * keep).min(255.0) as u8,
        (color.g() as f32 * coverage + dst.g() as f32 * keep).min(255.0) as u8,
        (color.b() as f32 * coverage + dst.b() as f32 * keep).min(255.0) as u8,
        (color.a() as f32 * coverage + dst.a() as f32 * keep).min(255.0) as u8,
    );
}

#[inline]
fn erase_px(img: &mut ColorImage, x: i32, y: i32, coverage: f32) {
    if coverage <= 0.0 {
        return;
    }
    let Some(idx) = index(img, x, y) else {
        return;
    };
    if coverage >= 1.0 {
        img.pixels[idx] = Color32::TRANSPARENT;
        return;
    }
    let dst = img.pixels[idx];
    let keep = 1.0 - coverage;
    img.pixels[idx] = Color32::from_rgba_premultiplied(
        (dst.r() as f32 * keep) as u8,
        (dst.g() as f32 * keep) as u8,
        (dst.b() as f32 * keep) as u8,
        (dst.a() as f32 * keep) as u8,
    );
}

/// Disegna un cerchio pieno di raggio `radius` centrato in `center`.
pub fn stamp(img: &mut ColorImage, center: Pos2, radius: f32, color: Color32) {
    let r = radius.max(0.5);
    if r < 0.75 {
        blend_px(
            img,
            center.x.floor() as i32,
            center.y.floor() as i32,
            color,
            1.0,
        );
        return;
    }
    let Some((x0, y0, x1, y1)) = circle_bounds(center.x, center.y, r, img.size) else {
        return;
    };
    for y in y0..y1 {
        let dy = y as f32 + 0.5 - center.y;
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - center.x;
            let coverage = (r - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);
            blend_px(img, x, y, color, coverage);
        }
    }
}

/// Cancella (porta a trasparente) un cerchio pieno.
pub fn erase_stamp(img: &mut ColorImage, center: Pos2, radius: f32) {
    let r = radius.max(0.5);
    if r < 0.75 {
        erase_px(img, center.x.floor() as i32, center.y.floor() as i32, 1.0);
        return;
    }
    let Some((x0, y0, x1, y1)) = circle_bounds(center.x, center.y, r, img.size) else {
        return;
    };
    for y in y0..y1 {
        let dy = y as f32 + 0.5 - center.y;
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - center.x;
            let coverage = (r - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);
            erase_px(img, x, y, coverage);
        }
    }
}

/// Percorre il segmento `a`-`b` chiamando `dab` a passi regolari.
fn walk(a: Pos2, b: Pos2, radius: f32, mut dab: impl FnMut(Pos2)) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length = (dx * dx + dy * dy).sqrt();
    let step = (radius.max(0.5) * 0.5).max(0.5);
    let steps = (length / step).ceil().max(1.0) as usize;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        dab(Pos2::new(a.x + dx * t, a.y + dy * t));
    }
}

/// Segmento tra `a` e `b` con spessore `radius`.
pub fn line(img: &mut ColorImage, a: Pos2, b: Pos2, radius: f32, color: Color32) {
    walk(a, b, radius, |p| stamp(img, p, radius, color));
}

/// Segmento cancellato (trasparente) tra `a` e `b`.
pub fn erase_line(img: &mut ColorImage, a: Pos2, b: Pos2, radius: f32) {
    walk(a, b, radius, |p| erase_stamp(img, p, radius));
}

/// Rettangolo con angoli opposti `a` e `b`.
pub fn rect(img: &mut ColorImage, a: Pos2, b: Pos2, filled: bool, radius: f32, color: Color32) {
    if filled {
        let [w, _] = img.size;
        let (sx, ex) = span(a.x, b.x, w);
        let (sy, ey) = span(a.y, b.y, img.size[1]);
        for y in sy..ey {
            let row = y as usize * w;
            for x in sx..ex {
                img.pixels[row + x as usize] = color;
            }
        }
        return;
    }

    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
    let corners = [
        Pos2::new(x0, y0),
        Pos2::new(x1, y0),
        Pos2::new(x1, y1),
        Pos2::new(x0, y1),
    ];
    for pair in corners.windows(2) {
        line(img, pair[0], pair[1], radius, color);
    }
    line(img, corners[3], corners[0], radius, color);
}

/// Ellisse inscritta nel rettangolo con angoli opposti `a` e `b`.
pub fn ellipse(img: &mut ColorImage, a: Pos2, b: Pos2, filled: bool, radius: f32, color: Color32) {
    let cx = (a.x + b.x) * 0.5;
    let cy = (a.y + b.y) * 0.5;
    let rx = ((a.x - b.x).abs() * 0.5).max(0.5);
    let ry = ((a.y - b.y).abs() * 0.5).max(0.5);
    let [w, h] = img.size;

    if filled {
        let (sy, ey) = span(cy - ry, cy + ry, h);
        for y in sy..ey {
            let dy = (y as f32 + 0.5 - cy) / ry;
            if dy.abs() > 1.0 {
                continue;
            }
            let half = rx * (1.0 - dy * dy).sqrt();
            let (sx, ex) = span(cx - half, cx + half, w);
            let row = y as usize * w;
            for x in sx..ex {
                img.pixels[row + x as usize] = color;
            }
        }
        return;
    }

    // Bordo: una polilinea con abbastanza segmenti da sembrare curva.
    let segments = (((rx + ry) * 2.0) as usize).max(32);
    let mut prev = Pos2::new(cx + rx, cy);
    for i in 1..=segments {
        let t = i as f32 / segments as f32 * std::f32::consts::TAU;
        let current = Pos2::new(cx + rx * t.cos(), cy + ry * t.sin());
        line(img, prev, current, radius, color);
        prev = current;
    }
}

/// Riempimento a secchio a partire dal pixel sotto `seed`.
///
/// Usa l'algoritmo a scansione (scanline): ogni riga viene riempita in un
/// colpo solo, quindi anche le aree grandi restano veloci.
pub fn flood_fill(img: &mut ColorImage, seed: Pos2, color: Color32) {
    let [w, h] = img.size;
    let (sx, sy) = (seed.x.floor() as i32, seed.y.floor() as i32);
    if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
        return;
    }

    let target = img.pixels[sy as usize * w + sx as usize];
    if target == color {
        return;
    }

    let bounds = img.size;
    let mut stack: Vec<(i32, i32)> = Vec::new();
    stack.push((sx, sy));

    while let Some((x, y)) = stack.pop() {
        if y < 0 || y >= h as i32 {
            continue;
        }
        let mut x = x;
        if !covers(img, x, y, target) {
            continue;
        }
        // Torna all'inizio della macchia su questa riga.
        while x > 0 && covers(img, x - 1, y, target) {
            x -= 1;
        }

        let mut span_above = false;
        let mut span_below = false;
        while x < w as i32 && covers(img, x, y, target) {
            let idx = y as usize * w + x as usize;
            img.pixels[idx] = color;

            if y > 0 {
                let above = covers(img, x, y - 1, target);
                if above && !span_above {
                    stack.push((x, y - 1));
                    span_above = true;
                } else if !above {
                    span_above = false;
                }
            }
            if y + 1 < bounds[1] as i32 {
                let below = covers(img, x, y + 1, target);
                if below && !span_below {
                    stack.push((x, y + 1));
                    span_below = true;
                } else if !below {
                    span_below = false;
                }
            }
            x += 1;
        }
    }
}

#[inline]
fn covers(img: &ColorImage, x: i32, y: i32, target: Color32) -> bool {
    index(img, x, y).is_some_and(|idx| img.pixels[idx] == target)
}

/// Interpolazione lineare tra due colori premoltiplicati.
#[inline]
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let lerp = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgba_premultiplied(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        lerp(a.a(), b.a()),
    )
}

/// "Fluidifica": trascina i pixel del pennello da `from` verso `to`,
/// deformando l'immagine con una caduta morbida verso il bordo del cerchio.
pub fn liquify(img: &mut ColorImage, from: Pos2, to: Pos2, radius: f32, strength: f32) {
    let [w, h] = img.size;
    let r = radius.max(2.0);
    let s = strength.clamp(0.05, 1.0);
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx.abs() < 0.01 && dy.abs() < 0.01 {
        return;
    }

    // Regione da leggere: deve contenere sia il cerchio del pennello sia i
    // punti da cui vengono prelevati i pixel (spostati all'indietro).
    let back = Pos2::new(to.x - dx * s - r, to.y - dy * s - r);
    let front = Pos2::new(to.x + r, to.y + r);
    let x0 = (back.x.min(front.x).floor() as i64).max(0) as usize;
    let y0 = (back.y.min(front.y).floor() as i64).max(0) as usize;
    let x1 = (back.x.max(front.x).ceil() as i64).min(w as i64).max(0) as usize;
    let y1 = (back.y.max(front.y).ceil() as i64).min(h as i64).max(0) as usize;
    if x1 <= x0 || y1 <= y0 {
        return;
    }

    let (rw, rh) = (x1 - x0, y1 - y0);
    let mut source = Vec::with_capacity(rw * rh);
    for y in y0..y1 {
        let row = y * w;
        source.extend_from_slice(&img.pixels[row + x0..row + x1]);
    }

    // Campionamento bilineare nella regione copiata.
    let last_x = (rw as f32 - 1.0).max(0.0);
    let last_y = (rh as f32 - 1.0).max(0.0);
    let sample = |x: f32, y: f32| -> Color32 {
        let fx = (x - x0 as f32 - 0.5).clamp(0.0, last_x);
        let fy = (y - y0 as f32 - 0.5).clamp(0.0, last_y);
        let ix = (fx.floor() as usize).min(rw.saturating_sub(1));
        let iy = (fy.floor() as usize).min(rh.saturating_sub(1));
        let ix1 = (ix + 1).min(rw - 1);
        let iy1 = (iy + 1).min(rh - 1);
        let tx = fx - ix as f32;
        let ty = fy - iy as f32;

        let p00 = source[iy * rw + ix];
        let p10 = source[iy * rw + ix1];
        let p01 = source[iy1 * rw + ix];
        let p11 = source[iy1 * rw + ix1];
        mix(mix(p00, p10, tx), mix(p01, p11, tx), ty)
    };

    let Some((bx0, by0, bx1, by1)) = circle_bounds(to.x, to.y, r, img.size) else {
        return;
    };
    for y in by0..by1 {
        let py = y as f32 + 0.5;
        for x in bx0..bx1 {
            let px = x as f32 + 0.5;
            let distance = ((px - to.x).powi(2) + (py - to.y).powi(2)).sqrt();
            if distance >= r {
                continue;
            }
            let t = 1.0 - distance / r;
            let falloff = t * t * (3.0 - 2.0 * t);
            if falloff <= 0.0 {
                continue;
            }
            let warped = sample(px - dx * s * falloff, py - dy * s * falloff);
            let idx = y as usize * w + x as usize;
            img.pixels[idx] = mix(img.pixels[idx], warped, falloff);
        }
    }
}
