// LYN: Widget ID

use egui_winit::clipboard::Clipboard;

#[derive(Debug, Hash, Clone, Copy)]
pub enum WidgetId {
    MainAppTopToolBar,
    MainAppButtonStatusBar,
    MainAppLeftExplorerPanel,

    ErrorModal,

    Tester,
    TesterTopUtilBar,
    TesterRightDetailPanel,

    PatternEditor,
    PatternEditorMidiNotificationBar,
    PatternEditorMidiDetailPanel,
    PatternEditorMidiDetailPanelGrid,
    PatternEditorMidiComboBoxCommTarget,

    ConnectionManager,

    TrackEditorTopPanel,
}

impl From<WidgetId> for egui::Id {
    #[inline]
    fn from(val: WidgetId) -> Self {
        Self::new(val)
    }
}

// LYN: Helper Functions

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
