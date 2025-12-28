use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    num::NonZero,
    ops,
    sync::Arc,
    time::Duration,
};

use dashmap::{DashMap, DashSet, mapref::one::Ref};
use log::trace;
use lyn_util::{comm::Format, egui::LynId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use rayon::{ThreadPool, ThreadPoolBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    app::PlayerContext,
    model::{
        comm::{CommStream, CommStreamErr, CommTarget},
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

// LYN: Central State Holder

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
    pub tracks_ordering_in_id: RwLock<Vec<TrackId>>,
    pub patterns_ordering_in_id: RwLock<Vec<PatternId>>,
    pub targets_ordering_in_id: RwLock<Vec<TargetId>>,
}

impl UiState {
    pub const MIN_SIZE_PER_BEAT: f32 = 40.;
    pub const MAX_SIZE_PER_BEAT: f32 = 400.;

    pub const STORAGE_KEY_TRACK_SPB: &str = "track-size-per-beat";
    pub const STORAGE_KEY_PATTERN_SPB: &str = "pattern-size-per-beat";
    pub const STORAGE_KEY_TRACKS_ORDER: &str = "tracks-ordering";
    pub const STORAGE_KEY_PATTERNS_ORDER: &str = "patterns-ordering";
    pub const STORAGE_KEY_TARGETS_ORDER: &str = "targets-ordering";
}

#[derive(Debug)]
pub struct App {
    err_modal_message: RwLock<Option<String>>,
    selected_pattern: RwLock<Option<PatternId>>,
    player_context: RwLock<PlayerContext>,
    comm_stream: DashMap<TargetId, CommStream>,
    comm_stream_connecting: DashSet<TargetId>,
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
            comm_stream: DashMap::new(),
            comm_stream_connecting: DashSet::new(),
        };
        let ui = UiState {
            track_editor_size_per_beat: RwLock::new(UiState::MIN_SIZE_PER_BEAT),
            pattern_editor_size_per_beat: RwLock::new(UiState::MIN_SIZE_PER_BEAT),
            tracks_ordering_in_id: RwLock::new(Vec::new()),
            patterns_ordering_in_id: RwLock::new(Vec::new()),
            targets_ordering_in_id: RwLock::new(Vec::new()),
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
    pub fn app_set_err_msg(&self, msg: Option<String>) {
        *self.app.err_modal_message.write() = msg;
    }
    pub fn app_get_err_msg(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.app.err_modal_message.read()
    }
    pub fn comm_stream_exists(&self, id: &TargetId) -> bool {
        self.app.comm_stream.try_get(id).is_present()
    }
    pub fn comm_get_stream(&self, id: &TargetId) -> Option<Ref<'_, TargetId, CommStream>> {
        self.app.comm_stream.get(id)
    }
    pub fn comm_drop_stream(&self, id: &TargetId) {
        self.app.comm_stream.remove(id);
    }
    pub fn comm_connect_stream_blocking(
        &self,
        id: TargetId,
        addr: &str,
        format: Format,
    ) -> Option<Ref<'_, TargetId, CommStream>> {
        if self.app.comm_stream_connecting.get(&id).is_some() {
            return None;
        }
        self.app.comm_stream_connecting.insert(id.clone());

        struct Guard<'state, 'id> {
            state: &'state CentralState,
            id: &'id TargetId,
        }
        impl<'state, 'id> Drop for Guard<'state, 'id> {
            fn drop(&mut self) {
                self.state.app.comm_stream_connecting.remove(self.id);
            }
        }

        let _guard = Guard {
            state: self,
            id: &id,
        };
        let addr: SocketAddr = addr.parse().ok()?;
        let timeout = Duration::from_secs(3);

        self.app.comm_stream.get(&id);
        trace!("old comm stream removed");
        let stream = match format {
            Format::WsBasedJson => {
                let tcp_stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
                tcp_stream.set_read_timeout(Some(timeout)).ok()?;
                tcp_stream.set_write_timeout(Some(timeout)).ok()?;

                trace!("trying to connect (websocket)");
                let (ws, _) = ws::client(format!("ws://{}", addr), tcp_stream).ok()?;
                CommStream::WebSocket(Box::new(ws))
            }
            Format::TcpBasedOsc => {
                trace!("trying to connect (tcp)");
                let stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
                CommStream::TcpStream(stream)
            }
        };
        self.app.comm_stream.insert(id.clone(), stream);
        trace!("new comm stream inserted");
        self.app.comm_stream.get(&id)
    }
    pub fn comm_send_data_blocking(
        &self,
        id: &TargetId,
        data: Vec<u8>,
    ) -> Result<(), CommStreamErr> {
        let Some(mut entry) = self.app.comm_stream.get_mut(id) else {
            return Err(CommStreamErr::NoCommStream);
        };
        match entry.value_mut() {
            CommStream::WebSocket(ws) => ws.send(data.into())?,
            CommStream::TcpStream(stream) => stream.write_all(&data)?,
        }
        Ok(())
    }
}

impl CentralState {
    pub fn sheet_bpm(&self) -> f64 {
        *self.sheet.bpm.read()
    }
    pub fn sheet_bpm_mut(&self) -> RwLockWriteGuard<'_, f64> {
        self.sheet.bpm.write()
    }

    pub fn sheet_comm_targets_iter(
        &self,
    ) -> dashmap::iter::Iter<'_, TargetId, Arc<RwLock<CommTarget>>> {
        self.sheet.targets.iter()
    }
    pub fn sheet_add_comm_target(&self) -> WithId<TargetId, Arc<RwLock<CommTarget>>> {
        let target = Arc::new(RwLock::new(CommTarget::default()));
        let id: TargetId = LynId::obtain_string().into();
        self.sheet.targets.insert(id.clone(), target.clone());
        self.ui.targets_ordering_in_id.write().push(id.clone());
        WithId::new(id, target)
    }
    pub fn sheet_del_comm_target(
        &self,
        id: &TargetId,
    ) -> Option<WithId<TargetId, Arc<RwLock<CommTarget>>>> {
        self.ui
            .targets_ordering_in_id
            .write()
            .retain(|tid| tid != id);
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
        self.ui.patterns_ordering_in_id.write().push(id.clone());
        WithId::new(id, pat)
    }

    pub fn sheet_del_pattern(
        &self,
        id: &PatternId,
    ) -> Option<WithId<PatternId, Arc<RwLock<SheetPattern>>>> {
        self.ui
            .patterns_ordering_in_id
            .write()
            .retain(|pid| pid != id);
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
        self.ui.tracks_ordering_in_id.write().push(id.clone());
        WithId::new(id, track)
    }

    pub fn sheet_del_track(
        &self,
        id: &TrackId,
    ) -> Option<WithId<TrackId, Arc<RwLock<SheetTrack>>>> {
        self.ui
            .tracks_ordering_in_id
            .write()
            .retain(|tid| tid != id);
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
