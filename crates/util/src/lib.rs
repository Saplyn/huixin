use egui::{
    FontData, FontFamily,
    epaint::text::{FontInsert, InsertFontFamily},
};
use epaint::text::FontPriority;

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
