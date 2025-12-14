use std::collections::{BTreeMap, HashMap};

use either::Either;
use lyn_util::{
    comm::{DataMap, Instruction},
    egui::LynId,
};

use crate::{model::SheetMessage, routines::metronome::TICK_PER_BEAT};

use super::SheetPatternTrait;

// LYN: Midi Pattern

#[derive(Debug)]
pub struct MidiPattern {
    // pattern
    pub name: String,
    pub icon: String,
    /// total ticks = beats / TICK_PER_BEAT
    pub beats: u64,

    // pattern internal
    end_tick_map: BTreeMap<u64, u32>,
    notes: HashMap<u64, Vec<MidiNote>>,

    // communication
    pub tag: String,
    pub target_id: Option<LynId>,
}

impl MidiPattern {
    pub fn new(name: String, icon: Option<String>) -> Self {
        Self {
            name,
            icon: icon.unwrap_or(String::from("󰄛 ")),
            beats: 1,
            end_tick_map: BTreeMap::new(),
            notes: HashMap::new(),
            tag: String::new(),
            target_id: None,
        }
    }

    pub fn notes_iter_owned(&self) -> impl Iterator<Item = MidiNote> {
        self.notes.values().flatten().copied()
    }
    pub fn add_note(&mut self, note: MidiNote) {
        self.notes.entry(note.start).or_default().push(note);
        self.end_tick_map
            .entry(note.end_tick())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    pub fn del_note(&mut self, hint: Either<LynId, MidiNote>) {
        match hint {
            Either::Left(id) => {
                self.notes.retain(|_, vec| {
                    let mut removed = false;
                    let mut i = 0;
                    while i < vec.len() {
                        if vec[i].id() == id {
                            if let Some(count) = self.end_tick_map.get_mut(&vec[i].end_tick()) {
                                if *count > 1 {
                                    *count -= 1;
                                } else {
                                    self.end_tick_map.remove(&vec[i].end_tick());
                                }
                            }
                            vec.remove(i);
                            removed = true;
                        } else {
                            i += 1;
                        }
                    }
                    !vec.is_empty() || !removed
                });
            }
            Either::Right(note) => {
                if let Some(vec) = self.notes.get_mut(&note.start)
                    && let Some(idx) = vec.iter().position(|n| n.id() == note.id())
                {
                    let note = vec.remove(idx);
                    if let Some(count) = self.end_tick_map.get_mut(&note.end_tick()) {
                        if *count > 1 {
                            *count -= 1;
                        } else {
                            self.end_tick_map.remove(&note.end_tick());
                        }
                    }
                    if vec.is_empty() {
                        self.notes.remove(&note.start);
                    }
                }
            }
        }
    }
    pub fn edit_note(&mut self, hint: Either<LynId, MidiNote>, f: impl FnOnce(&mut MidiNote)) {
        match hint {
            Either::Left(id) => {
                let mut clean = None;
                for (start, notes) in self.notes.iter_mut() {
                    if let Some(note) = notes.iter_mut().find(|n| n.id() == id) {
                        if let Some(count) = self.end_tick_map.get_mut(&note.end_tick()) {
                            if *count > 1 {
                                *count -= 1;
                            } else {
                                self.end_tick_map.remove(&note.end_tick());
                            }
                        }
                        f(note);
                        self.end_tick_map
                            .entry(note.end_tick())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                        if *start != note.start {
                            let moved_note = *note;
                            notes.retain(|n| n.id() != id);
                            if notes.is_empty() {
                                clean = Some(*start);
                            }
                            self.notes
                                .entry(moved_note.start)
                                .or_default()
                                .push(moved_note);
                        }
                        break;
                    }
                }
                if let Some(start) = clean {
                    self.notes.remove(&start);
                }
            }
            Either::Right(old_note) => {
                if let Some(vec) = self.notes.get_mut(&old_note.start)
                    && let Some(idx) = vec.iter().position(|n| n.id() == old_note.id())
                {
                    let note = &mut vec[idx];
                    if let Some(count) = self.end_tick_map.get_mut(&note.end_tick()) {
                        if *count > 1 {
                            *count -= 1;
                        } else {
                            self.end_tick_map.remove(&note.end_tick());
                        }
                    }
                    f(note);
                    self.end_tick_map
                        .entry(note.end_tick())
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                    if note.start != old_note.start {
                        let moved_note = vec[idx];
                        vec.remove(idx);
                        if vec.is_empty() {
                            self.notes.remove(&old_note.start);
                        }
                        self.notes
                            .entry(moved_note.start)
                            .or_default()
                            .push(moved_note);
                    }
                }
            }
        }
    }
    #[inline]
    pub fn min_beats(&self) -> u64 {
        self.end_tick_map
            .iter()
            .next_back()
            .map(|(max_end_tick, _)| max_end_tick.div_ceil(TICK_PER_BEAT))
            .unwrap_or(1)
    }
}

impl Default for MidiPattern {
    fn default() -> Self {
        Self {
            name: String::new(),
            icon: String::from("󰄛 "),
            beats: 1,
            end_tick_map: BTreeMap::new(),
            notes: HashMap::new(),
            tag: String::new(),
            target_id: None,
        }
    }
}

impl SheetPatternTrait for MidiPattern {
    #[inline]
    fn name_ref(&self) -> &String {
        &self.name
    }
    #[inline]
    fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }
    #[inline]
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[inline]
    fn icon_ref(&self) -> &String {
        &self.icon
    }
    #[inline]
    fn icon_mut(&mut self) -> &mut String {
        &mut self.icon
    }
    #[inline]
    fn set_icon(&mut self, icon: String) {
        self.icon = icon;
    }

    #[inline]
    fn beats(&self) -> u64 {
        self.beats
    }

    #[inline]
    fn msg_at(&self, tick: u64) -> Vec<SheetMessage> {
        let Some(target_id) = self.target_id else {
            return Vec::new();
        };
        self.notes.get(&tick).map_or_else(Vec::new, |notes| {
            notes
                .iter()
                .map(|note| SheetMessage {
                    target_id,
                    payload: Instruction {
                        tag: self.tag.clone(),
                        data: note.form_data(),
                    },
                })
                .collect()
        })
    }
}

// LYN: Midi Note

#[derive(Debug, Clone, Copy)]
pub struct MidiNote {
    id: LynId,
    pub midicode: u8,
    pub strength: u16,
    pub start: u64,
    pub length: u64,
}

impl MidiNote {
    pub fn new(midicode: u8, strength: u16, start: u64, length: u64) -> Self {
        Self {
            id: LynId::obtain_id(),
            midicode,
            strength,
            start,
            length,
        }
    }
    #[inline]
    pub fn id(&self) -> LynId {
        self.id
    }
    #[inline]
    pub fn end_tick(&self) -> u64 {
        self.start + self.length
    }
    #[inline]
    pub fn form_data(&self) -> DataMap {
        let mut map = DataMap::new();
        map.insert("midicode".to_string(), self.midicode.into());
        map.insert("strength".to_string(), self.strength.into());
        map.insert("length".to_string(), self.length.into());
        map
    }
}
