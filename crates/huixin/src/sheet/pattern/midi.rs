use super::SheetPatternTrait;

#[derive(Debug)]
pub struct MidiPattern {
    pub name: String,
    pub icon: String,
    pub beats: u64,

    pub notes: Vec<MidiNote>,
}

#[derive(Debug)]
pub struct MidiNote {
    pub midicode: u8,
    pub strength: u16,
    pub start: u64,
    pub length: u64,
}

impl MidiPattern {
    pub fn new(name: String) -> Self {
        Self {
            name,
            icon: String::from("󰄛 "),
            beats: 1,
            notes: vec![],
        }
    }
}

impl Default for MidiPattern {
    fn default() -> Self {
        Self {
            name: String::new(),
            icon: String::from("󰄛 "),
            beats: 1,
            notes: vec![],
        }
    }
}

impl SheetPatternTrait for MidiPattern {
    fn name_ref(&self) -> &String {
        &self.name
    }
    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn icon_ref(&self) -> &String {
        &self.icon
    }
    fn icon_mut(&mut self) -> &mut String {
        &mut self.icon
    }
    fn set_icon(&mut self, icon: String) {
        self.icon = icon;
    }

    fn beats(&self) -> u64 {
        self.beats
    }
}

impl MidiPattern {
    pub fn notes_ref(&self) -> &Vec<MidiNote> {
        &self.notes
    }

    pub fn notes_mut(&mut self) -> &mut Vec<MidiNote> {
        &mut self.notes
    }
}
