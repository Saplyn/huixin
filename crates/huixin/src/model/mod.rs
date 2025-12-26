pub mod comm;
pub mod pattern;
pub mod persistence;
pub mod state;
pub mod track;

pub const DEFAULT_COMM_TARGET_ADDR: &str = "127.0.0.1:3000";
pub const DEFAULT_SELECTABLE_COLOR: ecolor::Color32 = ecolor::Color32::from_rgb(100, 149, 237);
pub const DEFAULT_TRACK_NAME: &str = "未命名轨道";
pub const DEFAULT_PATTERN_NAME: &str = "未命名片段";
pub const DEFAULT_ICON: &str = "󰝚 ";
