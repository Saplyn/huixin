use std::{num::NonZero, ops, sync::Arc};

use dashmap::DashMap;
use lyn_util::egui::LynId;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    app::PlayerContext,
    model::{
        CommTarget,
        pattern::{SheetPattern, SheetPatternTrait, SheetPatternType, midi::MidiPattern},
        track::{SheetTrack, SheetTrackType, pattern::PatternTrack},
    },
    routines::{RoutineId, metronome::TICK_PER_BEAT},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternId(String);
impl From<String> for PatternId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackId(String);
impl From<String> for TrackId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetId(String);
impl From<String> for TargetId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct WithId<I, T> {
    pub id: I,
    pub item: T,
}
impl<I, T> WithId<I, T> {
    pub fn new(id: I, item: T) -> Self {
        Self { id, item }
    }
}
impl<I, T> ops::Deref for WithId<I, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
impl<I, T> ops::DerefMut for WithId<I, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

#[derive(Debug)]
pub struct CentralState {
    workers: ThreadPool,
    pub ui: UiState,
    app: App,
    metro: Metronome,
    sheet: Sheet,
}

#[derive(Debug)]
pub struct UiState {
    pub track_editor_size_per_beat: RwLock<f32>,
    pub pattern_editor_size_per_beat: RwLock<f32>,
}

impl UiState {
    pub const MIN_SIZE_PER_BEAT: f32 = 40.;
    pub const MAX_SIZE_PER_BEAT: f32 = 400.;
}

#[derive(Debug)]
pub struct App {
    err_modal_message: RwLock<Option<String>>,
    selected_pattern: RwLock<Option<PatternId>>,
    player_context: RwLock<PlayerContext>,
}

#[derive(Debug)]
pub struct Metronome {
    playing: RwLock<bool>,
    curr_tick: RwLock<u64>,
    tick_memory: DashMap<RoutineId, u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sheet {
    bpm: RwLock<f64>,
    length_in_beats: RwLock<NonZero<u64>>,

    tracks: DashMap<TrackId, Arc<RwLock<SheetTrack>>>,
    patterns: DashMap<PatternId, Arc<RwLock<SheetPattern>>>,
    targets: DashMap<TargetId, Arc<RwLock<CommTarget>>>,
}

impl CentralState {
    pub fn init() -> Self {
        let app = App {
            err_modal_message: RwLock::new(None),
            selected_pattern: RwLock::new(None),
            player_context: RwLock::new(PlayerContext::Sheet),
        };
        let ui = UiState {
            track_editor_size_per_beat: RwLock::new(UiState::MIN_SIZE_PER_BEAT),
            pattern_editor_size_per_beat: RwLock::new(UiState::MIN_SIZE_PER_BEAT),
        };
        let sheet = Sheet {
            bpm: RwLock::new(130.),
            length_in_beats: RwLock::new(NonZero::<u64>::MIN),
            tracks: DashMap::new(),
            patterns: DashMap::new(),
            targets: DashMap::new(),
        };
        let metro = Metronome {
            playing: RwLock::new(false),
            curr_tick: RwLock::new(0),
            tick_memory: DashMap::default(),
        };

        Self {
            workers: ThreadPoolBuilder::new().build().unwrap(),
            ui,
            app,
            metro,
            sheet,
        }
    }
}

impl CentralState {
    pub fn selected_pattern_id(&self) -> RwLockReadGuard<'_, Option<PatternId>> {
        self.app.selected_pattern.read()
    }
    pub fn selected_pattern(&self) -> Option<WithId<PatternId, Arc<RwLock<SheetPattern>>>> {
        let pat_id_guard = self.app.selected_pattern.read();
        self.sheet
            .patterns
            .get(pat_id_guard.as_ref()?)
            .map(|entry| WithId::new(entry.key().clone(), entry.value().clone()))
    }
    pub fn select_pattern(&self, pat_id: Option<PatternId>) {
        *self.app.selected_pattern.write() = pat_id;
    }

    pub fn player_set_context(&self, context: PlayerContext) {
        *self.app.player_context.write() = context;
    }
    pub fn player_context(&self) -> PlayerContext {
        *self.app.player_context.read()
    }
    pub fn worker_spawn_task(&self, f: impl FnOnce() + Send + 'static) {
        self.workers.spawn(f);
    }
    pub fn set_err_msg(&self, msg: Option<String>) {
        *self.app.err_modal_message.write() = msg;
    }
    pub fn get_err_msg(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.app.err_modal_message.read()
    }
}

impl CentralState {
    pub fn sheet_bpm(&self) -> f64 {
        *self.sheet.bpm.read()
    }
    pub fn sheet_bpm_mut(&self) -> RwLockWriteGuard<'_, f64> {
        self.sheet.bpm.write()
    }

    pub fn sheet_comm_targets_iter(&self) -> &DashMap<TargetId, Arc<RwLock<CommTarget>>> {
        &self.sheet.targets
    }
    pub fn sheet_add_comm_target(&self) -> WithId<TargetId, Arc<RwLock<CommTarget>>> {
        let target = Arc::new(RwLock::new(CommTarget::default()));
        let id: TargetId = LynId::obtain_string().into();
        self.sheet.targets.insert(id.clone(), target.clone());
        WithId::new(id, target)
    }
    pub fn sheet_del_comm_target(
        &self,
        id: &TargetId,
    ) -> Option<WithId<TargetId, Arc<RwLock<CommTarget>>>> {
        self.sheet
            .targets
            .remove(id)
            .map(|entry| WithId::new(entry.0, entry.1))
    }
    pub fn sheet_get_comm_target(&self, id: &TargetId) -> Option<Arc<RwLock<CommTarget>>> {
        self.sheet.targets.get(id).map(|item| item.clone())
    }

    pub fn sheet_length_in_beats(&self) -> u64 {
        self.sheet.length_in_beats.read().get()
    }
    pub fn sheet_length_in_beats_mut(&self) -> RwLockWriteGuard<'_, NonZero<u64>> {
        self.sheet.length_in_beats.write()
    }

    pub fn sheet_add_pattern(
        &self,
        pattern_type: SheetPatternType,
    ) -> WithId<PatternId, Arc<RwLock<SheetPattern>>> {
        let pat = Arc::new(RwLock::new(match pattern_type {
            SheetPatternType::Midi => SheetPattern::Midi(MidiPattern::new()),
        }));
        let id: PatternId = LynId::obtain_string().into();
        self.sheet.patterns.insert(id.clone(), pat.clone());
        WithId::new(id, pat)
    }

    pub fn sheet_del_pattern(
        &self,
        id: &PatternId,
    ) -> Option<WithId<PatternId, Arc<RwLock<SheetPattern>>>> {
        self.sheet
            .patterns
            .remove(id)
            .map(|entry| WithId::new(entry.0, entry.1))
    }

    pub fn sheet_get_pattern(&self, id: &PatternId) -> Option<Arc<RwLock<SheetPattern>>> {
        self.sheet.patterns.get(id).map(|item| item.clone())
    }

    pub fn sheet_patterns_iter(
        &self,
    ) -> dashmap::iter::Iter<'_, PatternId, Arc<RwLock<SheetPattern>>> {
        self.sheet.patterns.iter()
    }

    pub fn sheet_add_track(
        &self,
        track_type: SheetTrackType,
    ) -> WithId<TrackId, Arc<RwLock<SheetTrack>>> {
        let track = Arc::new(RwLock::new(match track_type {
            SheetTrackType::Pattern => SheetTrack::Pattern(PatternTrack::new()),
        }));
        let id: TrackId = LynId::obtain_string().into();
        self.sheet.tracks.insert(id.clone(), track.clone());
        WithId::new(id, track)
    }

    pub fn sheet_del_track(
        &self,
        id: &TrackId,
    ) -> Option<WithId<TrackId, Arc<RwLock<SheetTrack>>>> {
        self.sheet
            .tracks
            .remove(id)
            .map(|entry| WithId::new(entry.0, entry.1))
    }

    pub fn sheet_get_track(&self, id: &TrackId) -> Option<Arc<RwLock<SheetTrack>>> {
        self.sheet.tracks.get(id).map(|item| item.clone())
    }

    pub fn sheet_tracks_iter(&self) -> dashmap::iter::Iter<'_, TrackId, Arc<RwLock<SheetTrack>>> {
        self.sheet.tracks.iter()
    }
}

impl CentralState {
    /// Returns the tick limit for metronome.
    pub fn metro_tick_limit(&self) -> u64 {
        match *self.app.player_context.read() {
            PlayerContext::Sheet => self.sheet.length_in_beats.read().get() * TICK_PER_BEAT - 1,
            PlayerContext::Pattern => self
                .selected_pattern()
                .as_ref()
                .map(|pat| pat.item.read().beats() * TICK_PER_BEAT - 1)
                .unwrap(),
        }
    }
    pub fn metro_playing(&self) -> bool {
        *self.metro.playing.read()
    }

    pub fn metro_stopped(&self) -> bool {
        *self.metro.curr_tick.read() == 0 && !*self.metro.playing.read()
    }

    pub fn metro_toggle_playing(&self, value: Option<bool>) {
        let mut playing = self.metro.playing.write();
        *playing = value.unwrap_or(!*playing);
    }

    pub fn metro_make_stop(&self) {
        *self.metro.playing.write() = false;
        *self.metro.curr_tick.write() = 0;
        self.metro.tick_memory.clear();
    }

    pub fn metro_tick_mut(&self) -> RwLockWriteGuard<'_, u64> {
        self.metro.curr_tick.write()
    }

    /// Requests the current tick for the given routine.
    ///
    /// A routine may only receive a tick once, any subsequent requests within the same
    /// tick will return `None`. If the metronome is not playing, `None` is returned.
    /// To get the current tick without context and restrictions, use `.query_tick()`.
    pub fn metro_request_tick(&self, id: RoutineId) -> Option<u64> {
        if !*self.metro.playing.read() {
            return None;
        }

        let curr_tick = self.metro.curr_tick.read();
        if let Some(last_tick) = self.metro.tick_memory.get(&id)
            && *last_tick == *curr_tick
        {
            None
        } else {
            self.metro.tick_memory.insert(id, *curr_tick);
            Some(*curr_tick)
        }
    }

    pub fn sheet_to_json_string_pretty(&self) -> Result<String, json::Error> {
        json::to_string_pretty(&self.sheet)
    }
    pub fn sheet_from_json_str(&self, s: &str) -> Result<(), json::Error> {
        let sheet: Sheet = json::from_str(s)?;
        *self.sheet.bpm.write() = *sheet.bpm.read();
        *self.sheet.length_in_beats.write() = *sheet.length_in_beats.read();
        self.sheet.tracks.clear();
        for entry in sheet.tracks.iter() {
            self.sheet
                .tracks
                .insert(entry.key().clone(), entry.value().clone());
        }
        self.sheet.patterns.clear();
        for entry in sheet.patterns.iter() {
            self.sheet
                .patterns
                .insert(entry.key().clone(), entry.value().clone());
        }
        self.sheet.targets.clear();
        for entry in sheet.targets.iter() {
            self.sheet
                .targets
                .insert(entry.key().clone(), entry.value().clone());
        }
        Ok(())
    }
}
