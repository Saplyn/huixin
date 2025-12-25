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
    PatternEditorMidiUtilBar,
    PatternEditorMidiDetailPanel,
    PatternEditorMidiDetailPanelGrid,
    PatternEditorMidiComboBoxCommTarget,

    ConnectionManager,
}

impl From<WidgetId> for egui::Id {
    #[inline]
    fn from(val: WidgetId) -> Self {
        Self::new(val)
    }
}
