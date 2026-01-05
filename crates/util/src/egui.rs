use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

use egui::{
    FontData, FontFamily,
    epaint::text::{FontInsert, InsertFontFamily},
};
use egui_winit::clipboard::Clipboard;
use epaint::text::FontPriority;
use rand::{Rng, rngs::ThreadRng};

// LYN: Chinese Font Loader

pub trait EguiContextExt {
    fn load_chinese_fonts(&self);
}

impl EguiContextExt for egui::Context {
    fn load_chinese_fonts(&self) {
        self.add_font(FontInsert::new(
            "Noto Sans SC",
            FontData::from_static(include_bytes!(
                "../fonts/noto-sans-sc/NotoSansSC-VariableFont_wght.ttf"
            )),
            vec![InsertFontFamily {
                family: FontFamily::Proportional,
                priority: FontPriority::Lowest,
            }],
        ));

        self.add_font(FontInsert::new(
            "MapleMono NF CN Regular",
            FontData::from_static(include_bytes!(
                "../fonts/maplemono-nf-cn/MapleMono-NF-CN-Regular.ttf"
            )),
            vec![
                InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Lowest,
                },
                InsertFontFamily {
                    family: FontFamily::Monospace,
                    priority: FontPriority::Lowest,
                },
            ],
        ));
    }
}

// LYN: Custom Identifier

const NUM_HASH_PREFIX: &str = "LynNumId";
const STR_HASH_INFIX: &str = "LynStrId";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LynId(u64);

impl LynId {
    pub fn obtain() -> LynId {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        LynId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    pub fn obtain_string() -> String {
        thread_local! {
            pub static RNG: RefCell<ThreadRng> = RefCell::new(rand::rng());
        }
        let mut state = DefaultHasher::new();
        let num = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            (rng.random::<u64>(), rng.random::<u64>())
        });
        num.0.hash(&mut state);
        (STR_HASH_INFIX, num.1).hash(&mut state);
        format!("{:x}", state.finish())
    }
}

impl From<LynId> for egui::Id {
    fn from(value: LynId) -> Self {
        Self::new(value)
    }
}

impl std::hash::Hash for LynId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (NUM_HASH_PREFIX, self.0).hash(state);
    }
}

// LYN: Coloring

pub fn text_color(bg: ecolor::Color32) -> ecolor::Color32 {
    let [r, g, b, _] = bg.to_array();
    let luminance = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;

    if luminance < 128.0 {
        ecolor::Color32::WHITE
    } else {
        ecolor::Color32::BLACK
    }
}

pub fn copy_color(color: ecolor::Color32) {
    let (r, g, b, _) = color.to_tuple();
    let hex = format!("#{r:02X}{g:02X}{b:02X}");
    Clipboard::new(None).set_text(hex);
}

pub fn parse_color(text: String) -> Option<ecolor::Color32> {
    let s = text.trim().trim_start_matches('#');
    if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&s[0..2], 16),
            u8::from_str_radix(&s[2..4], 16),
            u8::from_str_radix(&s[4..6], 16),
        ) {
            Some(ecolor::Color32::from_rgb(r, g, b))
        } else {
            None
        }
    } else {
        let parts: Vec<&str> = s.split(',').map(|s| s.trim()).collect();
        if parts.len() == 3
            && let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            )
        {
            Some(ecolor::Color32::from_rgb(r, g, b))
        } else {
            None
        }
    }
}
