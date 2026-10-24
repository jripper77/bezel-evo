//! Studio's document adapter to the authoritative runtime, with lazy recovery.
use bezel_core::{
    BezelError, Result,
    ports::{RuntimeConnection, RuntimeEndpoint},
};
use bezel_runtime::{
    Document, Engine,
    wire::{self, RuntimeOp},
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
type Factory = Box<dyn Fn() -> Result<Engine> + Send + Sync>;
struct Draft {
    document: Document,
    saved: Document,
    revision: (u64, u64),
}
struct State {
    local: Option<Engine>,
    drafts: BTreeMap<String, Draft>,
    live: BTreeMap<String, bool>,
    cards: BTreeMap<String, bool>,
    revision: u64,
    options: Option<wire::Options>,
    suspended: bool,
    cursor: Option<(String, u32, u64)>,
}
/// One serialized editor connection, shared with device control requests.
pub struct EngineBridge {
    connection: Arc<dyn RuntimeConnection>,
    factory: Factory,
    state: Mutex<State>,
}
impl EngineBridge {
    pub fn new(connection: Arc<dyn RuntimeConnection>, factory: Factory) -> Self {
        Self {
            connection,
            factory,
            state: Mutex::new(State {
                local: None,
                drafts: BTreeMap::new(),
                live: BTreeMap::new(),
                cards: BTreeMap::new(),
                revision: 0,
                options: None,
                suspended: false,
                cursor: None,
            }),
        }
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, State>> {
        self.state
            .lock()
            .map_err(|_| BezelError::Transport("editor runtime lock poisoned".into()))
    }
    fn call(
        &self,
        state: &mut State,
        key: &str,
        op: RuntimeOp,
        binary: &[u8],
    ) -> Result<(Value, Vec<u8>)> {
        let metadata = wire::request(key, op)?;
        let reply = if let Some(local) = &mut state.local {
            local.request(&metadata, binary)?
        } else {
            match self.connection.request(&metadata, binary) {
                Ok(reply) => reply,
                Err(error) => {
                    if self.connection.reachable() {
                        // A fresh authenticated session must receive editor state,
                        // but never replay the uncertain operation itself.
                        self.restore_remote(state)?;
                        state.cursor = None;
                        return Err(error);
                    }
                    self.connection.take_direct_ownership()?;
                    let mut local = (self.factory)()?;
                    local.suspend(state.suspended);
                    if let Some(options) = &state.options {
                        local.options(options.clone())?;
                    }
                    for (key, draft) in &state.drafts {
                        local.document(key, 0, draft.saved.clone(), true)?;
                        local.document(key, 0, draft.document.clone(), false)?;
                        if state.live.get(key) == Some(&true) {
                            local.live(key, true)?;
                        }
                    }
                    for (key, on) in &state.cards {
                        let (reply, _) = local
                            .request(&wire::request(key, RuntimeOp::Cards { on: *on })?, &[])?;
                        wire::response(&reply)?;
                    }
                    state.cursor = None;
                    let mut request: wire::RuntimeMessage = serde_json::from_slice(&metadata)
                        .map_err(|e| BezelError::InvalidInput(e.to_string()))?;
                    if let RuntimeOp::Image { after, .. } = &mut request.op {
                        *after = 0;
                    }
                    let reply = local.request(&wire::request(key, request.op)?, binary)?;
                    state.local = Some(local);
                    reply
                }
            }
        };
        let reply = wire::unpack_reply(reply)?;
        Ok((wire::response(&reply.0)?, reply.1))
    }
    fn restore_remote(&self, state: &mut State) -> Result<()> {
        let send = |key: &str, op: RuntimeOp, binary: &[u8]| -> Result<()> {
            let reply = self.connection.request(&wire::request(key, op)?, binary)?;
            let reply = wire::unpack_reply(reply)?;
            wire::response(&reply.0)?;
            Ok(())
        };
        if let Some(options) = &state.options {
            send(
                "__options__",
                RuntimeOp::Options {
                    options: options.clone(),
                },
                &[],
            )?;
        }
        for (key, draft) in &state.drafts {
            for (document, saved) in [(&draft.saved, true), (&draft.document, false)] {
                state.revision = state.revision.saturating_add(1);
                send(
                    key,
                    RuntimeOp::Draft {
                        revision: state.revision,
                        saved,
                    },
                    &wire::document(&document.theme, Some(&document.assets))?,
                )?;
            }
            if let Some(on) = state.live.get(key) {
                send(key, RuntimeOp::Live { on: *on }, &[])?;
            }
            if let Some(on) = state.cards.get(key) {
                send(key, RuntimeOp::Cards { on: *on }, &[])?;
            }
        }
        Ok(())
    }
    pub fn needs_sync(&self, key: &str, revision: (u64, u64), saved: bool) -> bool {
        saved
            || self.lock().map_or(true, |s| {
                s.drafts.get(key).is_none_or(|d| d.revision != revision)
            })
    }
    pub fn sync(
        &self,
        key: &str,
        revision: (u64, u64),
        document: Document,
        saved: bool,
    ) -> Result<()> {
        let mut state = self.lock()?;
        if !saved
            && state
                .drafts
                .get(key)
                .is_some_and(|d| d.revision == revision)
        {
            return Ok(());
        }
        let assets = state
            .drafts
            .get(key)
            .is_none_or(|d| d.revision.1 != revision.1);
        let binary = wire::document(&document.theme, assets.then_some(&document.assets))?;
        let previous = state
            .drafts
            .get(key)
            .map(|d| d.saved.clone())
            .unwrap_or_else(|| document.clone());
        state.drafts.insert(
            key.into(),
            Draft {
                saved: if saved { document.clone() } else { previous },
                document,
                revision,
            },
        );
        state.revision = state.revision.saturating_add(1);
        state.cursor = None;
        let revision = state.revision;
        self.call(
            &mut state,
            key,
            RuntimeOp::Draft { revision, saved },
            &binary,
        )?;
        Ok(())
    }
    pub fn live(&self, key: &str, on: bool) -> Result<()> {
        let mut state = self.lock()?;
        state.live.insert(key.into(), on);
        self.call(&mut state, key, RuntimeOp::Live { on }, &[])?;
        Ok(())
    }
    /// Tries a profile cover on every live screen of the engine, Light's own
    /// included, their themes unchanged; answers how many it covers.
    pub fn preview_cover(
        &self,
        transition: &bezel_runtime::profiles::TransitionSettings,
        themes: &std::collections::BTreeMap<String, String>,
    ) -> Result<usize> {
        let mut state = self.lock()?;
        let (reply, _) = self.call(
            &mut state,
            "__profiles__",
            RuntimeOp::PreviewCover {
                transition: transition.clone(),
                themes: themes.clone(),
            },
            &[],
        )?;
        Ok(reply["screens"].as_u64().map_or(0, |n| n as usize))
    }
    pub fn try_themes(
        &self,
        themes: &std::collections::BTreeMap<String, String>,
        profile: Option<String>,
    ) -> Result<usize> {
        let mut state = self.lock()?;
        let (reply, _) = self.call(
            &mut state,
            "__profiles__",
            RuntimeOp::TryThemes {
                themes: themes.clone(),
                profile,
            },
            &[],
        )?;
        Ok(reply["screens"].as_u64().map_or(0, |n| n as usize))
    }
    /// What the profiles do now, as the engine sees them.
    pub fn profile_status(&self) -> Result<Option<bezel_runtime::profiles::Status>> {
        let mut state = self.lock()?;
        let (reply, _) = self.call(&mut state, "__profiles__", RuntimeOp::ProfileStatus, &[])?;
        Ok(serde_json::from_value(reply).ok())
    }
    pub fn request(&self, key: &str, op: RuntimeOp) -> Result<(Value, Vec<u8>)> {
        {
            let mut state = self.lock()?;
            self.call(&mut state, key, op, &[])
        }
    }
    pub fn canvas(&self, key: &str, max: u32, cards: bool) -> Result<Vec<u8>> {
        let mut state = self.lock()?;
        if state.cards.get(key).copied().unwrap_or(true) != cards {
            self.call(&mut state, key, RuntimeOp::Cards { on: cards }, &[])?;
            state.cards.insert(key.into(), cards);
        }
        let after = state
            .cursor
            .as_ref()
            .filter(|(k, m, _)| k == key && *m == max)
            .map_or(0, |(_, _, s)| *s);
        let (value, pixels) =
            self.call(&mut state, key, RuntimeOp::Image { after, max, cards }, &[])?;
        wire::frame(&value, pixels.clone())?;
        state.cursor = Some((key.into(), max, value["sequence"].as_u64().unwrap_or(0)));
        Ok(bytes(&value, pixels))
    }
    pub fn thumbnail(&self, document: Document) -> Result<bezel_core::domain::frame::Frame> {
        let binary = wire::document(&document.theme, Some(&document.assets))?;
        let mut state = self.lock()?;
        let (value, pixels) =
            self.call(&mut state, "__thumbnail__", RuntimeOp::Thumbnail, &binary)?;
        wire::frame(&value, pixels)?
            .ok_or_else(|| BezelError::InvalidInput("missing thumbnail".into()))
    }
    pub fn options(&self, options: wire::Options) -> Result<()> {
        let mut state = self.lock()?;
        state.options = Some(options.clone());
        self.call(
            &mut state,
            "__options__",
            RuntimeOp::Options { options },
            &[],
        )?;
        Ok(())
    }
    pub fn suspend(&self, on: bool) {
        if let Ok(mut state) = self.lock() {
            state.suspended = on;
            if let Some(local) = &mut state.local {
                local.suspend(on);
            }
        }
    }
    pub fn wait_idle(&self, deadline: std::time::Instant) -> bool {
        loop {
            let idle = self
                .lock()
                .is_ok_and(|state| state.local.as_ref().is_none_or(Engine::quiescent));
            if idle {
                return true;
            }
            if std::time::Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn drive(&self) -> Result<Duration> {
        self.lock()?
            .local
            .as_mut()
            .map_or(Ok(Duration::from_millis(50)), Engine::tick)
    }
}
pub fn bytes(value: &Value, pixels: Vec<u8>) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(12 + pixels.len());
    for key in ["width", "height", "nextMs"] {
        bytes.extend_from_slice(&(value[key].as_u64().unwrap_or(0) as u32).to_le_bytes());
    }
    bytes.extend(pixels);
    // Optional f64 LE trailer: measured render time, including unchanged images.
    if let Some(millis) = value["renderMillis"].as_f64() {
        bytes.extend_from_slice(&millis.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bezel_core::{
        domain::{
            clock::{Language, LocalTime},
            frame::{Frame, Rgba},
            geometry::{Orientation, Size},
            theme::{AssetRef, Theme},
        },
        ports::{FrameRenderer, RenderContext},
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    struct Renderer;
    impl FrameRenderer for Renderer {
        fn render(
            &mut self,
            t: &Theme,
            _: &BTreeMap<AssetRef, Vec<u8>>,
            _: RenderContext<'_>,
        ) -> Result<Frame> {
            Ok(Frame::filled(t.canvas, Rgba::BLACK))
        }
    }
    fn time() -> LocalTime {
        LocalTime {
            year: 2026,
            month: 10,
            day: 16,
            hour: 12,
            minute: 0,
            second: 0,
            weekday: 4,
            millisecond: 0,
        }
    }
    fn engine() -> Engine {
        Engine::new(
            Box::new(Renderer),
            Box::new(bezel_sensors::FakeSensors::demo()),
            Box::new(bezel_media::FfmpegTranscoder::new(None)),
            Arc::new(bezel_devices::FakeBus::turing_88()),
            Arc::new(bezel_devices::FakeConnector::default()),
            Language::English,
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../target/fallback-video"),
            time,
        )
        .unwrap()
    }
    struct Connection {
        server: Mutex<Engine>,
        dead: AtomicBool,
        claimed: Arc<AtomicBool>,
        calls: AtomicUsize,
    }
    impl RuntimeConnection for Connection {
        fn request(&self, meta: &[u8], pixels: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.dead.load(Ordering::SeqCst) {
                return Err(BezelError::Transport("crashed".into()));
            }
            self.server.lock().unwrap().request(meta, pixels)
        }
        fn take_direct_ownership(&self) -> Result<()> {
            self.claimed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
    fn setup() -> (EngineBridge, Arc<Connection>, Arc<AtomicUsize>) {
        let claimed = Arc::new(AtomicBool::new(false));
        let connection = Arc::new(Connection {
            server: Mutex::new(engine()),
            dead: AtomicBool::new(false),
            claimed: claimed.clone(),
            calls: AtomicUsize::new(0),
        });
        let built = Arc::new(AtomicUsize::new(0));
        let built_factory = built.clone();
        let bridge = EngineBridge::new(
            connection.clone(),
            Box::new(move || {
                assert!(claimed.load(Ordering::SeqCst));
                built_factory.fetch_add(1, Ordering::SeqCst);
                Ok(engine())
            }),
        );
        (bridge, connection, built)
    }
    fn document() -> Document {
        Document {
            theme: Theme::blank("saved", Size::new(1920, 480), Orientation::Landscape),
            assets: BTreeMap::new(),
        }
    }
    #[test]
    fn a_reachable_new_session_restores_documents_without_a_duplicate_engine() {
        struct Recover {
            server: Mutex<Engine>,
            fail: AtomicBool,
        }
        impl RuntimeConnection for Recover {
            fn request(&self, meta: &[u8], pixels: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
                if self.fail.swap(false, Ordering::SeqCst) {
                    self.server.lock().unwrap().detach()?;
                    return Err(BezelError::Transport("session closed".into()));
                }
                self.server.lock().unwrap().request(meta, pixels)
            }
            fn reachable(&self) -> bool {
                true
            }
            fn take_direct_ownership(&self) -> Result<()> {
                panic!("owner is alive")
            }
        }
        let connection = Arc::new(Recover {
            server: Mutex::new(engine()),
            fail: AtomicBool::new(false),
        });
        let bridge = EngineBridge::new(connection.clone(), Box::new(|| panic!("no second engine")));
        bridge.sync("editor", (1, 1), document(), true).unwrap();
        let mut draft = document();
        draft.theme.canvas = Size::new(320, 480);
        draft.theme.orientation = Orientation::Portrait;
        bridge.sync("editor", (2, 1), draft, false).unwrap();
        connection.fail.store(true, Ordering::SeqCst);
        assert!(bridge.canvas("editor", 320, true).is_err());
        connection.server.lock().unwrap().tick().unwrap();
        let image = bridge.canvas("editor", 320, true).unwrap();
        assert_eq!(u32::from_le_bytes(image[..4].try_into().unwrap()), 213);
        assert_eq!(u32::from_le_bytes(image[4..8].try_into().unwrap()), 320);
        assert!(bridge.lock().unwrap().local.is_none());
    }

    #[test]
    fn reachable_owner_request_error_never_builds_a_fallback() {
        struct Reject;
        impl RuntimeConnection for Reject {
            fn request(&self, _: &[u8], _: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
                Err(BezelError::Transport("request rejected".into()))
            }
            fn reachable(&self) -> bool {
                true
            }
            fn take_direct_ownership(&self) -> Result<()> {
                panic!("reachable owner must keep ownership")
            }
        }
        let bridge =
            EngineBridge::new(Arc::new(Reject), Box::new(|| panic!("no duplicate engine")));
        assert!(bridge.canvas("editor", 320, true).is_err());
        assert!(bridge.lock().unwrap().local.is_none());
    }

    #[test]
    fn remote_canvas_is_binary_scaled_and_unchanged_until_document_edit() {
        let (bridge, connection, built) = setup();
        bridge.sync("editor", (1, 1), document(), true).unwrap();
        connection.server.lock().unwrap().tick().unwrap();
        let frame = bridge.canvas("editor", 320, true).unwrap();
        assert_eq!(u32::from_le_bytes(frame[..4].try_into().unwrap()), 320);
        assert_eq!(frame.len(), 20 + 320 * 80 * 4);
        let measured = connection
            .server
            .lock()
            .unwrap()
            .image("editor", 0, 320)
            .unwrap()
            .render_millis;
        assert_eq!(
            f64::from_le_bytes(frame[frame.len() - 8..].try_into().unwrap()),
            measured
        );
        assert_eq!(bridge.canvas("editor", 320, true).unwrap().len(), 20);
        assert!(!bridge.needs_sync("editor", (1, 1), false));
        let before = connection.calls.load(Ordering::SeqCst);
        bridge.sync("editor", (1, 1), document(), false).unwrap();
        assert_eq!(before, connection.calls.load(Ordering::SeqCst));
        assert_eq!(
            built.load(Ordering::SeqCst),
            0,
            "Studio must not build a runtime while Light is alive"
        );
    }
    #[test]
    fn light_crash_claims_lock_and_recovers_same_engine_with_a_fresh_canvas() {
        let (bridge, connection, built) = setup();
        bridge.sync("editor", (1, 1), document(), true).unwrap();
        bridge.canvas("editor", 320, true).unwrap();
        connection.dead.store(true, Ordering::SeqCst);
        let pending = bridge.canvas("editor", 320, true).unwrap();
        assert_eq!(
            pending.len(),
            20,
            "cold recovery is a readonly pending snapshot"
        );
        bridge.drive().unwrap();
        let frame = bridge.canvas("editor", 320, true).unwrap();
        let measured = bridge
            .lock()
            .unwrap()
            .local
            .as_ref()
            .unwrap()
            .image("editor", 0, 320)
            .unwrap()
            .render_millis;
        assert_eq!(
            f64::from_le_bytes(frame[frame.len() - 8..].try_into().unwrap()),
            measured
        );
        assert!(
            frame.len() > 20,
            "old sequence must not suppress the recovered image"
        );
        assert!(connection.claimed.load(Ordering::SeqCst));
        assert_eq!(built.load(Ordering::SeqCst), 1);
        bridge.drive().unwrap();
        bridge.canvas("editor", 640, true).unwrap();
        assert_eq!(built.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn missing_light_recovers_but_service_validation_errors_do_not_take_over() {
        let (bridge, connection, built) = setup();
        assert!(
            bridge
                .request(
                    "unknown",
                    RuntimeOp::Image {
                        after: 0,
                        max: 9999,
                        cards: true
                    }
                )
                .is_err()
        );
        assert_eq!(built.load(Ordering::SeqCst), 0);
        connection.dead.store(true, Ordering::SeqCst);
        bridge.sync("editor", (1, 1), document(), true).unwrap();
        assert_eq!(built.load(Ordering::SeqCst), 1);
        bridge.canvas("editor", 320, true).unwrap();
    }
    #[test]
    fn a_changed_document_gets_a_full_canvas_even_when_its_pixels_are_identical() {
        let (bridge, connection, _) = setup();
        bridge.sync("editor", (1, 1), document(), true).unwrap();
        connection
            .server
            .lock()
            .unwrap()
            .tick_at(Duration::ZERO, time())
            .unwrap();
        assert!(bridge.canvas("editor", 320, true).unwrap().len() > 20);
        assert_eq!(bridge.canvas("editor", 320, true).unwrap().len(), 20);
        let mut next = document();
        next.theme.name = "Opened document".into();
        bridge.sync("editor", (2, 1), next, false).unwrap();
        connection
            .server
            .lock()
            .unwrap()
            .tick_at(Duration::from_millis(100), time())
            .unwrap();
        assert!(
            bridge.canvas("editor", 320, true).unwrap().len() > 20,
            "a cleared canvas needs the complete new snapshot, even with unchanged pixel bytes"
        );
    }
}
