// LYN: Widget ID

#[derive(Debug, Hash, Clone, Copy)]
pub enum WidgetId {
    MainAppTopToolBar,
    MainAppButtonStatusBar,
    MainAppLeftExplorerPanel,
    MainAppExplorerPatternsOrderingDnd,

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
    ConnectionManagerTargetsOrderingDnd,

    TrackEditorTopPanel,
    TrackEditorHeaderOrderingDnd,
}

impl From<WidgetId> for egui::Id {
    #[inline]
    fn from(val: WidgetId) -> Self {
        Self::new(val)
    }
}
