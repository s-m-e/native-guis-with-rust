use clap::Parser;
use fltk::{
    app,
    draw,
    enums::{Color, ColorDepth, Event, Font, Key},
    prelude::*,
    widget::Widget,
    window::Window,
};
use image::imageops::FilterType;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

// ── constants ──────────────────────────────────────────────────────────────────

const THUMB_W: i32 = 160;
const THUMB_H: i32 = 120;
const CELL_PAD: i32 = 10;
const TEXT_H: i32 = 36;
const CELL_W: i32 = THUMB_W + CELL_PAD * 2;
const CELL_H: i32 = THUMB_H + TEXT_H + CELL_PAD * 2;
const LOADER_THREADS: usize = 4;
const BATCH_SIZE: usize = 8;

// ── CLI ────────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "imgv", about = "Minimal image viewer")]
struct Cli {
    /// Directory to scan for images (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

// ── data types ─────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct ThumbData {
    rgb: Arc<Vec<u8>>,
    w: i32,
    h: i32,
}

#[derive(Clone)]
struct ImageEntry {
    path: PathBuf,
    name: String,
    modified: Option<SystemTime>,
    thumb: Option<ThumbData>,
}

enum ThumbMsg {
    Batch,
}

struct FullMsg {
    req_idx: usize, // which index was selected when request was made
    data: ThumbData,
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Overview,
    Image,
}

struct AppState {
    images: Arc<Mutex<Vec<ImageEntry>>>,
    selected: usize,
    scroll_y: i32,
    cols: i32,
    win_w: i32,
    win_h: i32,
    mode: ViewMode,
    full_image: Option<ThumbData>,
}

impl AppState {
    fn new(images: Arc<Mutex<Vec<ImageEntry>>>, win_w: i32, win_h: i32) -> Self {
        AppState {
            images,
            selected: 0,
            scroll_y: 0,
            cols: (win_w / CELL_W).max(1),
            win_w,
            win_h,
            mode: ViewMode::Overview,
            full_image: None,
        }
    }

    fn recalc_cols(&mut self) {
        self.cols = (self.win_w / CELL_W).max(1);
    }

    fn len(&self) -> usize {
        self.images.lock().unwrap().len()
    }

    fn total_rows(&self) -> i32 {
        let n = self.len() as i32;
        (n + self.cols - 1) / self.cols
    }

    fn clamp_scroll(&mut self) {
        let max_scroll = ((self.total_rows() * CELL_H) - self.win_h).max(0);
        self.scroll_y = self.scroll_y.clamp(0, max_scroll);
    }

    fn ensure_selected_visible(&mut self) {
        let row = (self.selected as i32) / self.cols;
        let top = row * CELL_H;
        let bot = top + CELL_H;
        if top < self.scroll_y {
            self.scroll_y = top;
        } else if bot > self.scroll_y + self.win_h {
            self.scroll_y = bot - self.win_h;
        }
    }
}

// ── JPEG EXIF thumbnail extraction ────────────────────────────────────────────

fn jpeg_exif_thumb(path: &Path) -> Option<image::DynamicImage> {
    let data = fs::read(path).ok()?;
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut pos = 2usize;
    while pos + 3 < data.len() {
        if data[pos] != 0xFF {
            break;
        }
        let marker = data[pos + 1];
        let seg_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        if marker == 0xE1 && pos + 2 + seg_len <= data.len() {
            let seg = &data[pos + 2..pos + 2 + seg_len];
            if seg.len() > 8 && &seg[2..8] == b"Exif\0\0" {
                if let Some(img) = parse_exif_ifd1_thumb(&seg[8..]) {
                    return Some(img);
                }
            }
        }
        if marker == 0xDA {
            break;
        }
        pos += 2 + seg_len;
    }
    None
}

fn parse_exif_ifd1_thumb(tiff: &[u8]) -> Option<image::DynamicImage> {
    if tiff.len() < 8 {
        return None;
    }
    let le = match &tiff[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let ru16 = |off: usize| -> usize {
        if off + 2 > tiff.len() { return 0; }
        let b = [tiff[off], tiff[off + 1]];
        if le { u16::from_le_bytes(b) as usize } else { u16::from_be_bytes(b) as usize }
    };
    let ru32 = |off: usize| -> usize {
        if off + 4 > tiff.len() { return 0; }
        let b = [tiff[off], tiff[off + 1], tiff[off + 2], tiff[off + 3]];
        if le { u32::from_le_bytes(b) as usize } else { u32::from_be_bytes(b) as usize }
    };
    let ifd0 = ru32(4);
    if ifd0 + 2 > tiff.len() { return None; }
    let n0 = ru16(ifd0);
    let ifd1_ptr = ifd0 + 2 + n0 * 12;
    if ifd1_ptr + 4 > tiff.len() { return None; }
    let ifd1 = ru32(ifd1_ptr);
    if ifd1 == 0 || ifd1 + 2 > tiff.len() { return None; }
    let n1 = ru16(ifd1);
    let mut joff = None;
    let mut jlen = None;
    for i in 0..n1 {
        let e = ifd1 + 2 + i * 12;
        if e + 12 > tiff.len() { break; }
        match ru16(e) {
            0x0201 => joff = Some(ru32(e + 8)),
            0x0202 => jlen = Some(ru32(e + 8)),
            _ => {}
        }
    }
    let off = joff?;
    let len = jlen?;
    if len == 0 || off + len > tiff.len() { return None; }
    image::load_from_memory(&tiff[off..off + len]).ok()
}

// ── image loading ─────────────────────────────────────────────────────────────

fn load_thumb(path: &Path) -> Option<ThumbData> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    let dyn_img = if ext == "jpg" || ext == "jpeg" {
        jpeg_exif_thumb(path).or_else(|| image::open(path).ok())
    } else {
        image::open(path).ok()
    }?;
    let (iw, ih) = (dyn_img.width(), dyn_img.height());
    let scale = ((THUMB_W as f32) / iw as f32)
        .min((THUMB_H as f32) / ih as f32)
        .min(1.0);
    let tw = ((iw as f32 * scale).round() as u32).max(1);
    let th = ((ih as f32 * scale).round() as u32).max(1);
    let thumb = dyn_img.resize(tw, th, FilterType::Triangle);
    let rgb = thumb.to_rgb8().into_raw();
    Some(ThumbData { rgb: Arc::new(rgb), w: tw as i32, h: th as i32 })
}

fn load_full(path: &Path) -> Option<ThumbData> {
    let img = image::open(path).ok()?;
    let w = img.width() as i32;
    let h = img.height() as i32;
    let rgb = img.to_rgb8().into_raw();
    Some(ThumbData { rgb: Arc::new(rgb), w, h })
}

// ── background thumbnail loader ───────────────────────────────────────────────

fn spawn_loaders(images: Arc<Mutex<Vec<ImageEntry>>>, tx: mpsc::SyncSender<ThumbMsg>) {
    thread::spawn(move || {
        let paths: Vec<(usize, PathBuf)> = {
            let imgs = images.lock().unwrap();
            imgs.iter().enumerate().map(|(i, e)| (i, e.path.clone())).collect()
        };
        let _n = paths.len();
        let (job_tx, job_rx) = mpsc::channel::<(usize, PathBuf)>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let tx = Arc::new(tx);
        let images = Arc::clone(&images);

        let mut handles = vec![];
        for _ in 0..LOADER_THREADS {
            let job_rx = Arc::clone(&job_rx);
            let tx = Arc::clone(&tx);
            let images = Arc::clone(&images);
            handles.push(thread::spawn(move || {
                let mut batch: Vec<(usize, ThumbData)> = Vec::new();
                loop {
                    let job = { job_rx.lock().unwrap().recv() };
                    match job {
                        Err(_) => break,
                        Ok((idx, path)) => {
                            if let Some(td) = load_thumb(&path) {
                                {
                                    let mut imgs = images.lock().unwrap();
                                    if idx < imgs.len() {
                                        imgs[idx].thumb = Some(td.clone());
                                    }
                                }
                                batch.push((idx, td));
                                if batch.len() >= BATCH_SIZE {
                                    batch.clear();
                                    tx.send(ThumbMsg::Batch).ok();
                                    app::awake();
                                }
                            }
                        }
                    }
                }
                if !batch.is_empty() {
                    tx.send(ThumbMsg::Batch).ok();
                    app::awake();
                }
            }));
        }

        for job in paths {
            job_tx.send(job).ok();
        }
        drop(job_tx);
        for h in handles {
            h.join().ok();
        }
    });
}

// ── directory scan ────────────────────────────────────────────────────────────

fn scan_dir(dir: &Path) -> Vec<ImageEntry> {
    let exts = ["jpg", "jpeg", "png", "gif", "tiff", "tif"];
    let mut entries: Vec<ImageEntry> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() { return None; }
            let ext = path.extension()?.to_string_lossy().to_lowercase();
            if !exts.contains(&ext.as_str()) { return None; }
            let name = path.file_name()?.to_string_lossy().into_owned();
            let modified = e.metadata().ok().and_then(|m| m.modified().ok());
            Some(ImageEntry { path, name, modified, thumb: None })
        })
        .collect();
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

// ── drawing ───────────────────────────────────────────────────────────────────

fn fmt_modified(t: SystemTime) -> String {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let h = (secs / 3600) % 24;
            let m = (secs / 60) % 60;
            let s = secs % 60;
            let days = secs / 86400;
            let y = 1970 + days / 365;
            let rem = days % 365;
            let mo = rem / 30 + 1;
            let da = rem % 30 + 1;
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, da, h, m, s)
        }
        Err(_) => "unknown".to_string(),
    }
}

fn truncate_str(s: &str, max_px: i32) -> String {
    let max_chars = (max_px / 7).max(0) as usize;
    if s.len() <= max_chars {
        s.to_string()
    } else if max_chars > 1 {
        let boundary = s
            .char_indices()
            .map(|(i, _)| i)
            .nth(max_chars.saturating_sub(1))
            .unwrap_or(s.len());
        format!("{}…", &s[..boundary])
    } else {
        s.chars().take(max_chars).collect()
    }
}

fn draw_overview(s: &AppState) {
    let imgs = s.images.lock().unwrap();
    let n = imgs.len() as i32;
    let cols = s.cols;
    let scroll = s.scroll_y;
    let ww = s.win_w;
    let wh = s.win_h;

    draw::draw_rect_fill(0, 0, ww, wh, Color::from_rgb(28, 28, 28));

    let x_off = ((ww - cols * CELL_W) / 2).max(0);

    for i in 0..n {
        let col = i % cols;
        let row = i / cols;
        let cx = x_off + col * CELL_W;
        let cy = row * CELL_H - scroll;
        if cy + CELL_H < 0 || cy > wh { continue; }

        let selected = i == s.selected as i32;
        let bg = if selected {
            Color::from_rgb(55, 95, 155)
        } else {
            Color::from_rgb(42, 42, 42)
        };
        draw::draw_rect_fill(cx + 2, cy + 2, CELL_W - 4, CELL_H - 4, bg);

        let entry = &imgs[i as usize];
        if let Some(td) = &entry.thumb {
            let tx = cx + CELL_PAD + (THUMB_W - td.w) / 2;
            let ty = cy + CELL_PAD + (THUMB_H - td.h) / 2;
            draw::draw_image(&td.rgb, tx, ty, td.w, td.h, ColorDepth::Rgb8).ok();
        } else {
            draw::set_draw_color(Color::from_rgb(65, 65, 65));
            draw::draw_rect_fill(
                cx + CELL_PAD, cy + CELL_PAD, THUMB_W, THUMB_H,
                Color::from_rgb(50, 50, 50),
            );
            draw::set_draw_color(Color::from_rgb(90, 90, 90));
            draw::draw_rect(cx + CELL_PAD, cy + CELL_PAD, THUMB_W, THUMB_H);
        }

        let ty_text = cy + CELL_PAD + THUMB_H + 4;
        let max_w = CELL_W - CELL_PAD * 2;

        draw::set_draw_color(Color::from_rgb(215, 215, 215));
        draw::set_font(Font::Helvetica, 11);
        let name = truncate_str(&entry.name, max_w);
        draw::draw_text2(
            &name, cx + CELL_PAD, ty_text, max_w, 16,
            fltk::enums::Align::Left,
        );

        draw::set_draw_color(Color::from_rgb(150, 150, 150));
        draw::set_font(Font::Helvetica, 10);
        if let Some(mtime) = entry.modified {
            let date = fmt_modified(mtime);
            draw::draw_text2(
                &date, cx + CELL_PAD, ty_text + 17, max_w, 14,
                fltk::enums::Align::Left,
            );
        }
    }

    if n == 0 {
        draw::set_draw_color(Color::from_rgb(150, 150, 150));
        draw::set_font(Font::Helvetica, 14);
        draw::draw_text2(
            "No images found", 0, wh / 2 - 10, ww, 20,
            fltk::enums::Align::Center,
        );
    }
}

fn draw_image_view(s: &AppState) {
    let ww = s.win_w;
    let wh = s.win_h;
    draw::draw_rect_fill(0, 0, ww, wh, Color::Black);
    if let Some(td) = &s.full_image {
        let scale = (ww as f32 / td.w as f32).min(wh as f32 / td.h as f32);
        let dw = (td.w as f32 * scale).round() as i32;
        let dh = (td.h as f32 * scale).round() as i32;
        let dx = (ww - dw) / 2;
        let dy = (wh - dh) / 2;
        if let Ok(mut img) =
            fltk::image::RgbImage::new(&td.rgb, td.w, td.h, ColorDepth::Rgb8)
        {
            img.draw(dx, dy, dw, dh);
        }
    } else {
        draw::set_draw_color(Color::from_rgb(100, 100, 100));
        draw::set_font(Font::Helvetica, 14);
        draw::draw_text2("Loading…", 0, wh / 2 - 10, ww, 20, fltk::enums::Align::Center);
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let dir = cli.path.canonicalize().unwrap_or(cli.path);

    let entries = scan_dir(&dir);
    let images: Arc<Mutex<Vec<ImageEntry>>> = Arc::new(Mutex::new(entries));

    // channel: background → main (thumbnails)
    let (thumb_tx, thumb_rx) = mpsc::sync_channel::<ThumbMsg>(32);
    spawn_loaders(Arc::clone(&images), thumb_tx);

    // channel: main → worker → main (full-size images)
    let (full_tx, full_rx) = mpsc::sync_channel::<FullMsg>(2);

    let app = app::App::default();

    let win_w = 1024i32;
    let win_h = 768i32;
    let mut win = Window::new(100, 100, win_w, win_h, "imgv");
    win.make_resizable(true);
    let mut canvas = Widget::default().with_size(win_w, win_h);
    win.resizable(&canvas);
    win.end();
    win.show();

    use std::cell::RefCell;
    use std::rc::Rc;

    let state = Rc::new(RefCell::new(AppState::new(Arc::clone(&images), win_w, win_h)));

    // ── idle: drain incoming messages ────────────────────────────────────────
    {
        let state = Rc::clone(&state);
        let mut canvas2 = canvas.clone();
        app::add_idle3(move |_| {
            let mut dirty = false;

            loop {
                match thumb_rx.try_recv() {
                    Ok(ThumbMsg::Batch) => dirty = true,
                    Err(_) => break,
                }
            }

            loop {
                match full_rx.try_recv() {
                    Ok(msg) => {
                        let mut s = state.borrow_mut();
                        if s.mode == ViewMode::Image && s.selected == msg.req_idx {
                            s.full_image = Some(msg.data);
                            dirty = true;
                        }
                    }
                    Err(_) => break,
                }
            }

            if dirty {
                canvas2.redraw();
            }
        });
    }

    // ── draw ─────────────────────────────────────────────────────────────────
    {
        let state = Rc::clone(&state);
        canvas.draw(move |w| {
            let mut s = state.borrow_mut();
            s.win_w = w.w();
            s.win_h = w.h();
            s.recalc_cols();
            drop(s);
            let s = state.borrow();
            match s.mode {
                ViewMode::Overview => draw_overview(&s),
                ViewMode::Image => draw_image_view(&s),
            }
        });
    }

    // ── events ────────────────────────────────────────────────────────────────
    {
        let state = Rc::clone(&state);
        let mut win2 = win.clone();
        let mut canvas3 = canvas.clone();
        let full_tx = full_tx.clone();

        canvas.handle(move |w, ev| match ev {
            Event::Push => {
                if state.borrow().mode != ViewMode::Overview {
                    return false;
                }
                let (mx, my) = (app::event_x(), app::event_y());
                let (cols, scroll, ww) = {
                    let s = state.borrow();
                    (s.cols, s.scroll_y, s.win_w)
                };
                let x_off = ((ww - cols * CELL_W) / 2).max(0);
                let col = (mx - x_off) / CELL_W;
                let row = (my + scroll) / CELL_H;
                if col >= 0 && col < cols {
                    let idx = (row * cols + col) as usize;
                    let n = state.borrow().len();
                    if idx < n {
                        state.borrow_mut().selected = idx;
                        if app::event_clicks() {
                            enter_image_view(&state, idx, &full_tx, &mut canvas3);
                        } else {
                            w.redraw();
                        }
                    }
                }
                true
            }

            Event::MouseWheel => {
                if state.borrow().mode != ViewMode::Overview {
                    return false;
                }
                let dy = match app::event_dy() {
                    app::MouseWheel::Up => -CELL_H,
                    app::MouseWheel::Down => CELL_H,
                    _ => 0,
                };
                {
                    let mut s = state.borrow_mut();
                    s.scroll_y += dy;
                    s.clamp_scroll();
                }
                w.redraw();
                true
            }

            Event::KeyDown => {
                let key = app::event_key();
                let mode = state.borrow().mode;

                // fullscreen toggle works in both modes
                if let Some(c) = key.to_char() {
                    if c == 'f' || c == 'F' {
                        if win2.fullscreen_active() {
                            win2.fullscreen(false);
                        } else {
                            win2.fullscreen(true);
                        }
                        return true;
                    }
                }

                match mode {
                    ViewMode::Overview => {
                        let n = state.borrow().len();
                        if n == 0 { return false; }
                        match key {
                            Key::Right => {
                                let sel = state.borrow().selected;
                                if sel + 1 < n {
                                    state.borrow_mut().selected = sel + 1;
                                    let mut s = state.borrow_mut();
                                    s.ensure_selected_visible();
                                    drop(s);
                                    w.redraw();
                                }
                                true
                            }
                            Key::Left => {
                                let sel = state.borrow().selected;
                                if sel > 0 {
                                    state.borrow_mut().selected = sel - 1;
                                    let mut s = state.borrow_mut();
                                    s.ensure_selected_visible();
                                    drop(s);
                                    w.redraw();
                                }
                                true
                            }
                            Key::Down => {
                                let (sel, cols) = {
                                    let s = state.borrow();
                                    (s.selected, s.cols as usize)
                                };
                                if sel + cols < n {
                                    state.borrow_mut().selected = sel + cols;
                                    let mut s = state.borrow_mut();
                                    s.ensure_selected_visible();
                                    drop(s);
                                    w.redraw();
                                }
                                true
                            }
                            Key::Up => {
                                let (sel, cols) = {
                                    let s = state.borrow();
                                    (s.selected, s.cols as usize)
                                };
                                if sel >= cols {
                                    state.borrow_mut().selected = sel - cols;
                                    let mut s = state.borrow_mut();
                                    s.ensure_selected_visible();
                                    drop(s);
                                    w.redraw();
                                }
                                true
                            }
                            Key::Enter => {
                                let idx = state.borrow().selected;
                                enter_image_view(&state, idx, &full_tx, &mut canvas3);
                                true
                            }
                            _ => false,
                        }
                    }

                    ViewMode::Image => match key {
                        Key::Escape => {
                            let mut s = state.borrow_mut();
                            s.mode = ViewMode::Overview;
                            s.full_image = None;
                            drop(s);
                            w.redraw();
                            true
                        }
                        Key::Right => {
                            let (sel, n) = {
                                let s = state.borrow();
                                (s.selected, s.len())
                            };
                            if sel + 1 < n {
                                state.borrow_mut().selected = sel + 1;
                                enter_image_view(&state, sel + 1, &full_tx, &mut canvas3);
                            }
                            true
                        }
                        Key::Left => {
                            let sel = state.borrow().selected;
                            if sel > 0 {
                                state.borrow_mut().selected = sel - 1;
                                enter_image_view(&state, sel - 1, &full_tx, &mut canvas3);
                            }
                            true
                        }
                        _ => false,
                    },
                }
            }

            Event::Resize => {
                // win_w/win_h updated in the draw callback from the widget's actual size
                true
            }

            _ => false,
        });
    }

    canvas.take_focus().ok();
    app.run().unwrap();
}

fn enter_image_view(
    state: &std::rc::Rc<std::cell::RefCell<AppState>>,
    idx: usize,
    full_tx: &mpsc::SyncSender<FullMsg>,
    canvas: &mut Widget,
) {
    let path = state.borrow().images.lock().unwrap()[idx].path.clone();
    {
        let mut s = state.borrow_mut();
        s.mode = ViewMode::Image;
        s.selected = idx;
        s.full_image = None;
    }
    canvas.redraw();

    let tx = full_tx.clone();
    thread::spawn(move || {
        if let Some(data) = load_full(&path) {
            tx.send(FullMsg { req_idx: idx, data }).ok();
            app::awake();
        }
    });
}
