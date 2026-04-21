use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use chrono::{DateTime, Local};
use image::imageops::FilterType;
use image::GenericImageView;
use slint::{Image, ModelRc, SharedPixelBuffer, VecModel};

slint::include_modules!();

const THUMBNAIL_SIZE: u32 = 300;
const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif"];

struct ImageInfo {
    path: PathBuf,
    filename: String,
    modified: String,
    thumbnail: Image,
}

fn is_image_file(path: &PathBuf) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn format_modified_time(path: &PathBuf) -> String {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .map(|time| {
            let datetime: DateTime<Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn load_thumbnail(path: &PathBuf) -> Option<Image> {
    let img = image::open(path).ok()?;
    let thumbnail = img.resize(THUMBNAIL_SIZE, THUMBNAIL_SIZE, FilterType::Triangle);
    let rgba = thumbnail.to_rgba8();
    let buffer = SharedPixelBuffer::clone_from_slice(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
    );
    Some(Image::from_rgba8(buffer))
}

fn load_full_image(path: &PathBuf) -> Option<Image> {
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    let buffer = SharedPixelBuffer::clone_from_slice(
        rgba.as_raw(),
        width,
        height,
    );
    Some(Image::from_rgba8(buffer))
}

fn load_images_from_directory(dir: &PathBuf) -> Vec<ImageInfo> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .ok()
        .map(|dir| {
            dir.filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && is_image_file(path))
                .collect()
        })
        .unwrap_or_default();

    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    entries
        .into_iter()
        .filter_map(|path| {
            let thumbnail = load_thumbnail(&path)?;
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            let modified = format_modified_time(&path);

            Some(ImageInfo {
                path,
                filename,
                modified,
                thumbnail,
            })
        })
        .collect()
}

fn main() -> Result<(), slint::PlatformError> {
    let args: Vec<String> = std::env::args().collect();
    let image_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    let window = MainWindow::new()?;

    let images = load_images_from_directory(&image_dir);
    let image_paths: Rc<RefCell<Vec<PathBuf>>> = Rc::new(RefCell::new(
        images.iter().map(|img| img.path.clone()).collect(),
    ));

    let image_data: Vec<ImageData> = images
        .into_iter()
        .enumerate()
        .map(|(index, img)| ImageData {
            thumbnail: img.thumbnail,
            full_image: Image::default(),
            filename: img.filename.into(),
            modified: img.modified.into(),
            index: index as i32,
        })
        .collect();

    let model = Rc::new(VecModel::from(image_data));
    window.set_images(ModelRc::from(model.clone()));

    if !image_paths.borrow().is_empty() {
        window.set_selected_index(0);
    }

    // Handle request for full image
    let paths = image_paths.clone();
    let window_weak = window.as_weak();
    window.on_request_full_image(move |index| {
        let paths = paths.borrow();
        if let Some(path) = paths.get(index as usize) {
            if let Some(full_img) = load_full_image(path) {
                if let Some(w) = window_weak.upgrade() {
                    w.set_current_full_image(full_img);
                }
            }
        }
    });

    // Handle clearing full image
    let window_weak = window.as_weak();
    window.on_clear_full_image(move || {
        if let Some(w) = window_weak.upgrade() {
            w.set_current_full_image(Image::default());
        }
    });

    // Handle fullscreen toggle
    let window_weak = window.as_weak();
    window.on_toggle_fullscreen(move || {
        if let Some(w) = window_weak.upgrade() {
            w.window().set_fullscreen(!w.window().is_fullscreen());
        }
    });

    // Handle navigation
    let window_weak = window.as_weak();
    let paths_len = image_paths.borrow().len() as i32;
    window.on_navigate(move |direction| {
        if let Some(w) = window_weak.upgrade() {
            let current = w.get_selected_index();
            let columns = w.get_columns();

            if paths_len == 0 {
                return;
            }

            let new_index = match direction {
                0 => {
                    // Left
                    if current > 0 {
                        current - 1
                    } else {
                        current
                    }
                }
                1 => {
                    // Right
                    if current < paths_len - 1 {
                        current + 1
                    } else {
                        current
                    }
                }
                2 => {
                    // Up
                    if current >= columns {
                        current - columns
                    } else {
                        current
                    }
                }
                3 => {
                    // Down
                    if current + columns < paths_len {
                        current + columns
                    } else {
                        current
                    }
                }
                _ => current,
            };

            w.set_selected_index(new_index);
        }
    });

    // Handle image view navigation (left/right only)
    let paths = image_paths.clone();
    let window_weak = window.as_weak();
    window.on_navigate_image(move |direction| {
        if let Some(w) = window_weak.upgrade() {
            let paths = paths.borrow();
            let paths_len = paths.len() as i32;
            if paths_len == 0 {
                return;
            }

            let current = w.get_selected_index();
            let new_index = match direction {
                0 => {
                    // Left (previous)
                    if current > 0 {
                        current - 1
                    } else {
                        current
                    }
                }
                1 => {
                    // Right (next)
                    if current < paths_len - 1 {
                        current + 1
                    } else {
                        current
                    }
                }
                _ => current,
            };

            if new_index != current {
                w.set_selected_index(new_index);
                if let Some(path) = paths.get(new_index as usize) {
                    if let Some(full_img) = load_full_image(path) {
                        w.set_current_full_image(full_img);
                    }
                }
            }
        }
    });

    window.run()
}
