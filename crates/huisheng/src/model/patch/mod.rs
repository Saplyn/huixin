use egui_snarl::Snarl;

use crate::model::{DEFAULT_PATCH_NAME, DEFAULT_SELECTABLE_COLOR, patch::node::PatchNode};

pub mod node;

// LYN: Patch

#[derive(Debug)]
pub struct Patch {
    pub icon: String,
    pub name: String,
    pub color: ecolor::Color32,

    pub snarl: Snarl<PatchNode>,
}

impl Patch {
    pub fn new() -> Self {
        Self {
            icon: "󰄛 ".to_string(),
            name: DEFAULT_PATCH_NAME.to_string(),
            color: DEFAULT_SELECTABLE_COLOR,

            snarl: Snarl::new(),
        }
    }
}

// LYN: Patch Output

pub const BLOCK_SIZE: usize = 1024;

pub type Number = f64;
pub type Text = String;
pub type Block = [Number; BLOCK_SIZE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchOutputType {
    Number,
    Text,
    Block,
    Bang,
}

pub struct PatchOutput;
impl PatchOutput {
    pub const fn empty_block() -> Block {
        [0.; BLOCK_SIZE]
    }
}
