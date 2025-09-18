use std::fmt;

// LYN: Widget ID

#[derive(Debug, Hash, Clone, Copy)]
pub enum WidgetId {
    MainAppTopToolBar,
    MainAppButtonStatusBar,
    ErrorModal,
}

impl From<WidgetId> for egui::Id {
    #[inline]
    fn from(val: WidgetId) -> Self {
        Self::new(val)
    }
}

// LYN: Pages

pub trait AppPage: eframe::App + fmt::Debug + Send + Sync {
    fn page_id(&self) -> PageId;
}
// impl<T> AppPage for T where T: eframe::App + fmt::Debug {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    #[default]
    Tester,
    Composer,
    Programmer,
    Networker,
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            PageId::Tester => "Tester",
            PageId::Composer => "Composer",
            PageId::Programmer => "Programmer",
            PageId::Networker => "Networker",
        };
        write!(f, "{text}")
    }
}
