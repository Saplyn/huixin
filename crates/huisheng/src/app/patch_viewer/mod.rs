use std::sync::Arc;

use egui::ecolor;
use egui_snarl::{
    Snarl,
    ui::{SnarlPin, SnarlViewer},
};

use crate::model::{
    patch::{
        WireDataType,
        node::{
            PatchNode, PatchNodeTrait, PatchNodeType, adsr_curve::ADSRCurve,
            amp_multiplier::AmpMultiplier, bang::BangNode, midi_to_freq::MidiToFreq,
            number::NumberNode, oscillator::Oscillator, remote_data::RemoteData, speaker::Speaker,
        },
    },
    state::CentralState,
};

mod constants;
mod node;

#[derive(Debug)]
pub struct PatchViewer {
    state: Arc<CentralState>,

    pub output: PatchViewerOutput,
}

#[derive(Debug, Default)]
pub struct PatchViewerOutput {
    pub rebuild: bool,
}

impl PatchViewer {
    #[inline(always)]
    pub fn new(state: Arc<CentralState>) -> Self {
        Self {
            state,
            output: PatchViewerOutput::default(),
        }
    }

    #[inline(always)]
    fn insert_node(&mut self, snarl: &mut Snarl<NodeType>, pos: egui::Pos2, node: PatchNode) {
        snarl.insert_node(pos, node);
        self.output.rebuild = true;
    }

    #[inline(always)]
    fn remove_node(&mut self, snarl: &mut Snarl<NodeType>, node: egui_snarl::NodeId) {
        snarl.remove_node(node);
        self.output.rebuild = true;
    }

    #[inline(always)]
    fn connect_node(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<NodeType>,
    ) {
        // refuse self-connection
        if from.id.node == to.id.node {
            return;
        }

        // refuse invalid type connection
        let (from_type, to_type) = (
            snarl[from.id.node].output_type(from.id.output),
            snarl[to.id.node].input_type(to.id.input),
        );
        let same_type = from_type == to_type;
        let from_nonblock_valid =
            from_type == WireDataType::NonBlock && to_type != WireDataType::Block;
        let to_nonblock_valid =
            to_type == WireDataType::NonBlock && from_type != WireDataType::Block;
        let from_constant_valid = from_type == WireDataType::Constant
            && matches!(to_type, WireDataType::Number | WireDataType::Text);
        let to_constant_valid = to_type == WireDataType::Constant
            && matches!(from_type, WireDataType::Number | WireDataType::Text);
        if !same_type
            && (!from_nonblock_valid && !to_nonblock_valid)
            && (!from_constant_valid && !to_constant_valid)
        {
            return;
        }

        // allow only one connection per input pin unless specified
        if !snarl[to.id.node].pin_accept_multi(to.id.input) {
            for &remote in &to.remotes {
                snarl.disconnect(remote, to.id);
            }
        }

        snarl.connect(from.id, to.id);
        snarl[to.id.node].take_input(to.id.input, from.id);
        snarl[from.id.node].on_output_connect(from.id.output, to.id);

        self.output.rebuild = true;
    }

    #[inline(always)]
    fn disconnect_node(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<NodeType>,
    ) {
        snarl.disconnect(from.id, to.id);
        snarl[to.id.node].drop_input(to.id.input, from.id);
        snarl[from.id.node].on_output_disconnect(from.id.output, to.id);

        self.output.rebuild = true;
    }
}

type NodeType = PatchNode;

impl SnarlViewer<NodeType> for PatchViewer {
    // LYN: Node Basic Information

    fn title(&mut self, node: &NodeType) -> String {
        node.name().to_string()
    }
    fn inputs(&mut self, node: &NodeType) -> usize {
        node.inputs()
    }
    fn outputs(&mut self, node: &NodeType) -> usize {
        node.outputs()
    }

    // LYN: Node Header UI Impl

    fn show_header(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[egui_snarl::InPin],
        _outputs: &[egui_snarl::OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
    ) {
        ui.add_enabled(
            false,
            egui::Label::new(
                egui::RichText::new(self.title(&snarl[node])).color(ecolor::Color32::LIGHT_GRAY),
            ),
        );
    }

    // LYN: Input Pin UI Implementation

    fn show_input(
        &mut self,
        pin: &egui_snarl::InPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
    ) -> impl SnarlPin + 'static {
        match snarl[pin.id.node].get_type() {
            // Signal
            PatchNodeType::Oscillator => Oscillator::pin_input(pin, ui, snarl, pin.id.input),
            PatchNodeType::Speaker => Speaker::pin_input(pin, ui, snarl, pin.id.input),

            // Variable
            PatchNodeType::Number => NumberNode::pin_input(pin, ui, snarl, pin.id.input),
            PatchNodeType::Bang => BangNode::pin_input(pin, ui, snarl, pin.id.input),

            // Calculation
            PatchNodeType::ADSRCurve => ADSRCurve::pin_input(pin, ui, snarl, pin.id.input),
            PatchNodeType::MidiToFreq => MidiToFreq::pin_input(pin, ui, snarl, pin.id.input),

            // Processing
            PatchNodeType::AmpMultiplier => AmpMultiplier::pin_input(pin, ui, snarl, pin.id.input),

            // Communication
            PatchNodeType::RemoteData => RemoteData::pin_input(pin, ui, snarl, pin.id.input),
        }
    }

    // LYN: Output Pin UI Implementation

    fn show_output(
        &mut self,
        pin: &egui_snarl::OutPin,
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
    ) -> impl SnarlPin + 'static {
        match snarl[pin.id.node].get_type() {
            // Signal
            PatchNodeType::Oscillator => Oscillator::pin_output(pin, ui, snarl, pin.id.output),
            PatchNodeType::Speaker => Speaker::pin_output(pin, ui, snarl, pin.id.output),

            // Variable
            PatchNodeType::Number => NumberNode::pin_output(pin, ui, snarl, pin.id.output),
            PatchNodeType::Bang => BangNode::pin_output(pin, ui, snarl, pin.id.output),

            // Calculation
            PatchNodeType::ADSRCurve => ADSRCurve::pin_output(pin, ui, snarl, pin.id.output),
            PatchNodeType::MidiToFreq => MidiToFreq::pin_output(pin, ui, snarl, pin.id.output),

            // Processing
            PatchNodeType::AmpMultiplier => {
                AmpMultiplier::pin_output(pin, ui, snarl, pin.id.output)
            }

            // Communication
            PatchNodeType::RemoteData => RemoteData::pin_output(pin, ui, snarl, pin.id.output),
        }
    }

    // LYN: Graph Right Click Menu

    fn has_graph_menu(&mut self, _pos: egui::Pos2, _snarl: &mut Snarl<NodeType>) -> bool {
        true
    }
    fn show_graph_menu(&mut self, pos: egui::Pos2, ui: &mut egui::Ui, snarl: &mut Snarl<NodeType>) {
        ui.menu_button("信号", |ui| {
            if ui.button(Oscillator::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::Oscillator(Oscillator::new().into()));
                ui.close();
            }
            if ui.button(Speaker::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::Speaker(Speaker::new()));
                ui.close();
            }
        });
        ui.menu_button("变量", |ui| {
            if ui.button(NumberNode::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::Number(NumberNode::new()));
                ui.close();
            }
            if ui.button("文字").clicked() {
                ui.close();
            }
            if ui.button(BangNode::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::Bang(BangNode::new()));
                ui.close();
            }
        });
        ui.menu_button("算数", |ui| {
            if ui.button("表达式").clicked() {
                ui.close();
            }
            if ui.button(ADSRCurve::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::ADSRCurve(ADSRCurve::new().into()));
                ui.close();
            }
            if ui.button(MidiToFreq::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::MidiToFreq(MidiToFreq::new()));
                ui.close();
            }
        });
        ui.menu_button("整波", |ui| {
            if ui.button("加波器").clicked() {
                ui.close();
            }
            if ui.button(AmpMultiplier::NAME).clicked() {
                self.insert_node(
                    snarl,
                    pos,
                    PatchNode::AmpMultiplier(AmpMultiplier::new().into()),
                );
                ui.close();
            }
            if ui.button("移幅器").clicked() {
                ui.close();
            }
            if ui.button("倍幅器").clicked() {
                ui.close();
            }
            if ui.button("限幅器").clicked() {
                ui.close();
            }
        });
        ui.menu_button("通讯", |ui| {
            if ui.button(RemoteData::NAME).clicked() {
                self.insert_node(snarl, pos, PatchNode::RemoteData(RemoteData::new()));
                ui.close();
            }
        });
        ui.menu_button("逻辑", |ui| {});
    }

    // LYN: Connect Action Impl

    fn connect(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<NodeType>,
    ) {
        self.connect_node(from, to, snarl);
    }

    fn disconnect(
        &mut self,
        from: &egui_snarl::OutPin,
        to: &egui_snarl::InPin,
        snarl: &mut Snarl<NodeType>,
    ) {
        self.disconnect_node(from, to, snarl);
    }

    // LYN: Connection Suggestion Menu

    fn has_dropped_wire_menu(
        &mut self,
        src_pins: egui_snarl::ui::AnyPins,
        snarl: &mut Snarl<NodeType>,
    ) -> bool {
        false
    }
    fn show_dropped_wire_menu(
        &mut self,
        pos: egui::Pos2,
        ui: &mut egui::Ui,
        src_pins: egui_snarl::ui::AnyPins,
        snarl: &mut Snarl<NodeType>,
    ) {
        todo!("called when dragging one pin's wire to empty canvas")
    }

    // LYN: Right-Click Node Menu

    fn has_node_menu(&mut self, _node: &NodeType) -> bool {
        true
    }
    fn show_node_menu(
        &mut self,
        node: egui_snarl::NodeId,
        _inputs: &[egui_snarl::InPin],
        _outputs: &[egui_snarl::OutPin],
        ui: &mut egui::Ui,
        snarl: &mut Snarl<NodeType>,
    ) {
        if ui.button("删除").clicked() {
            self.remove_node(snarl, node);
            ui.close();
        }
    }
}
