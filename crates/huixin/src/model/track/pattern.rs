use std::{ops::Range, sync::Arc};

use interavl::IntervalTree;
use lyn_util::egui::LynId;
use serde::{Deserialize, Serialize, ser::SerializeStruct};

use crate::{
    model::{
        DEFAULT_ICON, DEFAULT_PATTERN_NAME, DEFAULT_SELECTABLE_COLOR,
        comm::SheetMessage,
        pattern::SheetPatternTrait,
        state::{CentralState, PatternId},
        track::SheetTrackTrait,
    },
    routines::metronome::TICK_PER_BEAT,
};

#[derive(Debug, Clone)]
pub struct PatternTrack {
    pub name: String,
    pub icon: String,
    pub color: ecolor::Color32,

    patterns: IntervalTree<u64, Vec<(LynId, PatternId)>>,
}

impl PatternTrack {
    pub fn new() -> Self {
        Self {
            name: String::from(DEFAULT_PATTERN_NAME),
            icon: String::from(DEFAULT_ICON),
            color: DEFAULT_SELECTABLE_COLOR,
            patterns: IntervalTree::default(),
        }
    }

    pub fn patterns_iter(&self) -> impl Iterator<Item = (&Range<u64>, &Vec<(LynId, PatternId)>)> {
        self.patterns.iter()
    }
    pub fn add_pattern(&mut self, range: Range<u64>, pattern_id: PatternId) {
        self.add_pattern_inner(range, pattern_id, None);
    }
    fn add_pattern_inner(
        &mut self,
        range: Range<u64>,
        pattern_id: PatternId,
        ui_id: Option<LynId>,
    ) {
        let ui_id = ui_id.unwrap_or(LynId::obtain());
        if let Some(vec) = self.patterns.get_mut(&range) {
            vec.push((ui_id, pattern_id));
        } else {
            self.patterns.insert(range, vec![(ui_id, pattern_id)]);
        }
    }
    pub fn del_pattern(&mut self, range: Range<u64>, pattern_id: PatternId) -> Result<(), ()> {
        if let Some(vec) = self.patterns.get_mut(&range)
            && let Some(pos) = vec.iter().position(|(_, id)| *id == pattern_id)
        {
            vec.remove(pos);
            if vec.is_empty() {
                self.patterns.remove(&range);
            }
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn edit_pattern_range(
        &mut self,
        old_range: Range<u64>,
        new_range: Range<u64>,
        pattern_id: (LynId, PatternId),
    ) {
        if self.del_pattern(old_range, pattern_id.1.clone()).is_ok() {
            self.add_pattern_inner(new_range, pattern_id.1, Some(pattern_id.0));
        }
    }

    pub fn beats(&self) -> u64 {
        let last_tick = self.patterns.max_interval_end().map_or(0, |end| *end);
        if last_tick.is_multiple_of(TICK_PER_BEAT) {
            last_tick
        } else {
            last_tick + (TICK_PER_BEAT - last_tick % TICK_PER_BEAT)
        }
    }
}

impl SheetTrackTrait for PatternTrack {
    #[inline]
    fn name_ref(&self) -> &String {
        &self.name
    }
    #[inline]
    fn name_mut(&mut self) -> &mut String {
        &mut self.name
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
    fn color(&self) -> ecolor::Color32 {
        self.color
    }
    #[inline]
    fn color_mut(&mut self) -> &mut ecolor::Color32 {
        &mut self.color
    }
    #[inline]
    fn msg_at(&self, tick: u64, state: Arc<CentralState>) -> Vec<SheetMessage> {
        let mut msgs = Vec::new();
        for (range, vec) in self.patterns.iter_overlaps(&(tick..tick + 1)) {
            for (_, pat_id) in vec {
                if let Some(pattern) = state.sheet_get_pattern(pat_id) {
                    let pattern = pattern.read();
                    let pattern_tick = tick - range.start;
                    msgs.append(&mut pattern.msg_at(pattern_tick));
                }
            }
        }
        msgs
    }
}

impl Serialize for PatternTrack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PatternTrack", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("icon", &self.icon)?;
        state.serialize_field("color", &self.color)?;
        let patterns: Vec<(Range<u64>, Vec<PatternId>)> = self
            .patterns
            .iter()
            .map(|entry| {
                let pattern_ids = entry.1.iter().map(|(_, id)| id.clone()).collect::<Vec<_>>();
                (entry.0.to_owned(), pattern_ids)
            })
            .collect();
        state.serialize_field("patterns", &patterns)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for PatternTrack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PatternTrackDeser {
            name: String,
            icon: String,
            color: ecolor::Color32,
            patterns: Vec<(Range<u64>, Vec<PatternId>)>,
        }
        let deser = PatternTrackDeser::deserialize(deserializer)?;
        let mut pattern_track = PatternTrack {
            name: deser.name,
            icon: deser.icon,
            color: deser.color,
            patterns: IntervalTree::default(),
        };
        for (range, pattern_ids) in deser.patterns {
            let vec = pattern_ids
                .into_iter()
                .map(|id| (LynId::obtain(), id))
                .collect::<Vec<_>>();
            pattern_track.patterns.insert(range, vec);
        }
        Ok(pattern_track)
    }
}
