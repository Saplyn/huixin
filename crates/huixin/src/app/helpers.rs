// LYN: Widget ID

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
