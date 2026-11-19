//! Operazioni di disegno su una bitmap RGBA (`ColorImage`).
//!
//! Tutte le coordinate sono in pixel dell'immagine, con l'origine in alto a
//! sinistra. Le forme vengono disegnate "a mano libera" tracciando dei timbri
//! circolari, così il risultato è morbido come in Paint.

use eframe::egui::{Color32, ColorImage, Pos2};

/// Colore del foglio appena creato.
pub const PAPER: Color32 = Color32::WHITE;

pub fn blank(width: usize, height: usize) -> ColorImage {
    ColorImage::filled([width.max(1), height.max(1)], PAPER)
}

#[inline]
fn set_px(img: &mut ColorImage, x: i32, y: i32, color: Color32) {
    let [w, h] = img.size;
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    if x < w && y < h {
        img.pixels[y * w + x] = color;
    }
}

/// Disegna un cerchio pieno di raggio `radius` centrato in `center`.
pub fn stamp(img: &mut ColorImage, center: Pos2, radius: f32, color: Color32) {
    let r = radius.max(0.5);
    let r2 = r * r;
    let (cx, cy) = (center.x, center.y);

    let x0 = (cx - r).floor() as i32;
    let x1 = (cx + r).ceil() as i32;
    let y0 = (cy - r).floor() as i32;
    let y1 = (cy + r).ceil() as i32;

    for y in y0..=y1 {
        let dy = y as f32 + 0.5 - cy;
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            if dx * dx + dy * dy <= r2 {
                set_px(img, x, y, color);
            }
        }
    }
}

/// Segmento tra `a` e `b` con spessore `radius`.
pub fn line(img: &mut ColorImage, a: Pos2, b: Pos2, radius: f32, color: Color32) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as usize;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        stamp(img, Pos2::new(a.x + dx * t, a.y + dy * t), radius, color);
    }
}

/// Rettangolo con angoli opposti `a` e `b`.
pub fn rect(img: &mut ColorImage, a: Pos2, b: Pos2, filled: bool, radius: f32, color: Color32) {
    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));

    if filled {
        let mut y = y0.floor() as i32;
        while (y as f32) <= y1.ceil() {
            let mut x = x0.floor() as i32;
            while (x as f32) <= x1.ceil() {
                set_px(img, x, y, color);
                x += 1;
            }
            y += 1;
        }
    } else {
        let p0 = Pos2::new(x0, y0);
        let p1 = Pos2::new(x1, y0);
        let p2 = Pos2::new(x1, y1);
        let p3 = Pos2::new(x0, y1);
        line(img, p0, p1, radius, color);
        line(img, p1, p2, radius, color);
        line(img, p2, p3, radius, color);
        line(img, p3, p0, radius, color);
    }
}

/// Ellisse inscritta nel rettangolo con angoli opposti `a` e `b`.
pub fn ellipse(img: &mut ColorImage, a: Pos2, b: Pos2, filled: bool, radius: f32, color: Color32) {
    let cx = (a.x + b.x) * 0.5;
    let cy = (a.y + b.y) * 0.5;
    let rx = ((a.x - b.x).abs() * 0.5).max(0.5);
    let ry = ((a.y - b.y).abs() * 0.5).max(0.5);

    if filled {
        let y0 = (cy - ry).floor() as i32;
        let y1 = (cy + ry).ceil() as i32;
        for y in y0..=y1 {
            let dy = (y as f32 + 0.5 - cy) / ry;
            if dy.abs() > 1.0 {
                continue;
            }
            let half = rx * (1.0 - dy * dy).sqrt();
            let x0 = (cx - half).floor() as i32;
            let x1 = (cx + half).ceil() as i32;
            for x in x0..=x1 {
                set_px(img, x, y, color);
            }
        }
    } else {
        let steps = (((rx + ry) * 4.0) as usize).max(32);
        let mut prev = Pos2::new(cx + rx, cy);
        for i in 1..=steps {
            let t = i as f32 / steps as f32 * std::f32::consts::TAU;
            let cur = Pos2::new(cx + rx * t.cos(), cy + ry * t.sin());
            line(img, prev, cur, radius, color);
            prev = cur;
        }
    }
}

/// Riempimento a secchio a partire dal pixel sotto `seed`.
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

    let mut stack = vec![(sx, sy)];
    while let Some((x, y)) = stack.pop() {
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            continue;
        }
        let idx = y as usize * w + x as usize;
        if img.pixels[idx] != target {
            continue;
        }
        img.pixels[idx] = color;

        stack.push((x + 1, y));
        stack.push((x - 1, y));
        stack.push((x, y + 1));
        stack.push((x, y - 1));
    }
}
