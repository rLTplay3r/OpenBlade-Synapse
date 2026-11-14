//! Device artwork (port of images.py): download the official render, cut the cable off, cache it,
//! and report the mouse's bounding box for placing the callouts.

use std::io::Read;
use std::path::{Path, PathBuf};

pub struct Rgba {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

fn hash12(s: &str) -> String {
    // FNV-1a (only needs to be stable; cache names differ from the Python version on purpose)
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:012x}")[..12].to_string()
}

pub fn cached_paths(url: &str) -> (PathBuf, PathBuf) {
    let d = obcore::cache_dir();
    let h = hash12(url);
    (d.join(format!("rs-device-{h}.png")), d.join(format!("rs-device-{h}-v3.png")))
}

pub fn decode(bytes: &[u8]) -> Option<Rgba> {
    // some of Razer's PNGs have a bad CRC in the colour-profile chunk; the pixels are fine
    let mut opts = png::DecodeOptions::default();
    opts.set_ignore_crc(true);
    let mut dec = png::Decoder::new_with_options(std::io::Cursor::new(bytes), opts);
    dec.set_transformations(png::Transformations::normalize_to_color8());
    dec.set_ignore_iccp_chunk(true);
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width as usize, info.height as usize);
    let mut px = Vec::with_capacity(w * h * 4);
    let data = &buf[..info.buffer_size()];
    match info.color_type {
        png::ColorType::Rgba => px.extend_from_slice(data),
        png::ColorType::Rgb => data.chunks(3).for_each(|c| px.extend_from_slice(&[c[0], c[1], c[2], 255])),
        png::ColorType::GrayscaleAlpha => data.chunks(2).for_each(|c| px.extend_from_slice(&[c[0], c[0], c[0], c[1]])),
        png::ColorType::Grayscale => data.iter().for_each(|g| px.extend_from_slice(&[*g, *g, *g, 255])),
        _ => return None,
    }
    Some(Rgba { w, h, px })
}

fn encode(img: &Rgba, path: &Path) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp.png");
    {
        let f = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
        let mut enc = png::Encoder::new(f, img.w as u32, img.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header().map_err(std::io::Error::other)?;
        wr.write_image_data(&img.px).map_err(std::io::Error::other)?;
    }
    std::fs::rename(tmp, path) // atomic: a killed process never leaves half a PNG
}

fn alpha_bbox(img: &Rgba, thr: u8) -> Option<(usize, usize, usize, usize)> {
    let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
    for y in 0..img.h {
        for x in 0..img.w {
            if img.px[(y * img.w + x) * 4 + 3] > thr {
                x0 = x0.min(x); y0 = y0.min(y); x1 = x1.max(x + 1); y1 = y1.max(y + 1);
            }
        }
    }
    (x0 != usize::MAX).then_some((x0, y0, x1, y1))
}

fn crop(img: &Rgba, x0: usize, y0: usize, x1: usize, y1: usize) -> Rgba {
    let (w, h) = (x1 - x0, y1 - y0);
    let mut px = Vec::with_capacity(w * h * 4);
    for y in y0..y1 {
        px.extend_from_slice(&img.px[(y * img.w + x0) * 4..(y * img.w + x1) * 4]);
    }
    Rgba { w, h, px }
}

/// Crop away a cable hanging off the top (a long run of narrow rows), trim to the object.
pub fn cut_cable(img: &Rgba) -> Rgba {
    let Some((bx0, by0, bx1, by1)) = alpha_bbox(img, 24) else { return crop(img, 0, 0, img.w, img.h) };
    let widths: Vec<usize> = (0..img.h)
        .map(|y| (0..img.w).filter(|x| img.px[(y * img.w + x) * 4 + 3] > 128).count())
        .collect();
    let first = widths.iter().position(|w| *w > 0).unwrap_or(0);
    let mut run = first;
    while run < img.h && widths[run] > 0 && (widths[run] as f64) < img.w as f64 * 0.12 {
        run += 1;
    }
    let top = if (run - first) as f64 > img.h as f64 * 0.06 { run.saturating_sub(2) } else { by0 };
    let pad = 6;
    crop(img, bx0.saturating_sub(pad), top, (bx1 + pad).min(img.w), (by1 + pad).min(img.h))
}

/// Path to the prepared image (downloading + processing on first use), None on failure. Blocking.
pub fn prepare(url: &str) -> Option<PathBuf> {
    if url.is_empty() {
        return None;
    }
    let (raw, out) = cached_paths(url);
    if out.exists() {
        if let Ok(b) = std::fs::read(&out) {
            if decode(&b).is_some() {
                return Some(out);
            }
        }
        let _ = std::fs::remove_file(&out);
    }
    let _ = std::fs::create_dir_all(obcore::cache_dir());
    let mut bytes = std::fs::read(&raw).ok().filter(|b| decode(b).is_some());
    if bytes.is_none() {
        let mut body = vec![];
        ureq::get(url)
            .header("User-Agent", "openblade-synapse")
            .call()
            .ok()?
            .body_mut()
            .as_reader()
            .read_to_end(&mut body)
            .ok()?;
        decode(&body)?;
        let _ = std::fs::write(&raw, &body);
        bytes = Some(body);
    }
    let img = decode(&bytes?)?;
    encode(&cut_cable(&img), &out).ok()?;
    Some(out)
}

/// [x0, y0, x1, y1, w, h] of the visible object in a prepared image.
pub fn image_box(path: &Path) -> Option<[u32; 6]> {
    let img = decode(&std::fs::read(path).ok()?)?;
    let (x0, y0, x1, y1) = alpha_bbox(&img, 24).unwrap_or((0, 0, img.w, img.h));
    Some([x0 as u32, y0 as u32, x1 as u32, y1 as u32, img.w as u32, img.h as u32])
}
