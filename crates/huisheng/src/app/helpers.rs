// LYN: Widget ID

#[derive(Debug, Hash, Clone, Copy)]
pub enum WidgetId {
    MainAppTopToolBar,
    MainAppButtonStatusBar,
    MainAppLeftExplorerPanel,
    MainAppCentralSnarlCanvas,
    MainAppExplorerPatchesOrderingDnd,

    SnarlNodeOscillatorWaveformComboBox(usize),
}

impl From<WidgetId> for egui::Id {
    #[inline]
    fn from(val: WidgetId) -> Self {
        Self::new(val)
    }
}
