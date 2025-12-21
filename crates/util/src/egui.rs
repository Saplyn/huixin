use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    sync::atomic::{AtomicU64, Ordering},
};

use egui::{
    FontData, FontFamily,
    epaint::text::{FontInsert, InsertFontFamily},
};
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
