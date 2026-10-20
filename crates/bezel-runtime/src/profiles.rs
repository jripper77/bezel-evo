//! Local display profiles, shared by the Studio and Light tray menus.
use crate::{Document, Engine, invalid};
use bezel_core::{
    Result,
    domain::{
        clock::LocalTime,
        cover::{self, Cover, CoverDirection, CoverEffect, CoverStage},
        playback::{CardTrigger, TriggerSource},
        sensor::{SensorKey, Snapshot, Wanted},
    },
    ports::{ThemeLocation, ThemeStore},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

/// What a rule watches (D-2026-10-20-studio-ux-v2-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    /// An app is in the foreground.
    Foreground,
    /// An app is running.
    Process,
    /// Music is playing.
    MediaPlaying,
    /// A sensor is above or below a value (for some seconds).
    Sensor,
    /// The day of the week and the time of day.
    Time,
    /// Nobody has touched the PC for some minutes.
    Idle,
    /// The PC is locked.
    Locked,
}

/// Above or below, for a sensor rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Compare {
    /// Strictly above the value.
    Above,
    /// Strictly below the value.
    Below,
}

/// How a profile's rules combine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Combine {
    /// Any true rule selects the profile.
    #[default]
    Any,
    /// Every rule must be true together.
    All,
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}

fn is_any(v: &Combine) -> bool {
    *v == Combine::Any
}

/// Longest wait a sensor rule can ask for, seconds.
pub const MAX_FOR_SECONDS: u32 = 3600;
/// Longest inactivity a rule can wait for, minutes.
pub const MAX_IDLE_MINUTES: u32 = 1440;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Rule {
    pub source: Source,
    /// The app of a foreground or process rule.
    #[serde(default)]
    pub app: String,
    pub priority: u32,
    pub return_seconds: u32,
    /// Sensor rules: the sensor.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub key: String,
    /// Sensor rules: above or below.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare: Option<Compare>,
    /// Sensor rules: the value, in the sensor's units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// Sensor rules: how long it must hold before it counts, seconds.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub for_seconds: u32,
    /// Time rules: the days, 0 = Monday … 6 = Sunday (none: every day).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub days: Vec<u8>,
    /// Time rules: from `HH:MM` …
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub from: String,
    /// … to `HH:MM` (earlier than `from`: past midnight).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub to: String,
    /// Idle rules: minutes without input.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub minutes: u32,
}

/// Minutes into the day of `HH:MM`.
fn minute_of(text: &str) -> Option<u32> {
    let (h, m) = text.split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60 && text.len() == 5).then_some(h * 60 + m)
}

impl Rule {
    /// A rule watching `source`, every other field empty.
    pub fn of(source: Source) -> Self {
        Self {
            source,
            app: String::new(),
            priority: 0,
            return_seconds: 0,
            key: String::new(),
            compare: None,
            value: None,
            for_seconds: 0,
            days: Vec::new(),
            from: String::new(),
            to: String::new(),
            minutes: 0,
        }
    }

    fn valid(&self) -> bool {
        if self.priority > 100 || self.return_seconds > 300 || self.app.len() > 256 {
            return false;
        }
        match self.source {
            Source::Foreground | Source::Process => !self.app.trim().is_empty(),
            Source::MediaPlaying | Source::Locked => true,
            Source::Sensor => {
                SensorKey::new(self.key.as_str()).is_some()
                    && self.compare.is_some()
                    && self.value.is_some_and(f64::is_finite)
                    && self.for_seconds <= MAX_FOR_SECONDS
            }
            Source::Time => {
                self.days.len() <= 7
                    && self.days.iter().all(|d| *d <= 6)
                    && minute_of(&self.from).is_some()
                    && minute_of(&self.to).is_some()
            }
            Source::Idle => (1..=MAX_IDLE_MINUTES).contains(&self.minutes),
        }
    }

    /// The sensor keys the rule needs measured.
    fn needs(&self) -> Option<String> {
        Some(match self.source {
            Source::MediaPlaying => "media.sessions".into(),
            Source::Foreground | Source::Process => "activity.processes".into(),
            Source::Idle | Source::Locked => "activity.presence".into(),
            Source::Sensor => self.key.clone(),
            Source::Time => return None,
        })
    }

    fn available(&self, snapshot: &Snapshot, time: Option<&LocalTime>) -> bool {
        match self.source {
            Source::MediaPlaying => snapshot.media_available,
            Source::Foreground | Source::Process => snapshot.activity_available,
            Source::Sensor => SensorKey::new(self.key.as_str())
                .is_some_and(|key| snapshot.get(&key).value().is_some()),
            Source::Time => time.is_some(),
            Source::Idle => snapshot.idle_seconds.is_some(),
            Source::Locked => snapshot.locked.is_some(),
        }
    }

    /// Whether the rule's condition holds now (before any wait).
    fn holds(&self, snapshot: &Snapshot, time: Option<&LocalTime>) -> bool {
        if !self.available(snapshot, time) {
            return false;
        }
        match self.source {
            Source::Foreground | Source::Process | Source::MediaPlaying => {
                self.app_matches(snapshot)
            }
            Source::Sensor => {
                let reading =
                    SensorKey::new(self.key.as_str()).and_then(|key| snapshot.get(&key).value());
                match (reading, self.value, self.compare) {
                    (Some(v), Some(limit), Some(Compare::Above)) => v > limit,
                    (Some(v), Some(limit), Some(Compare::Below)) => v < limit,
                    _ => false,
                }
            }
            Source::Time => time.is_some_and(|t| {
                let (Some(from), Some(to)) = (minute_of(&self.from), minute_of(&self.to)) else {
                    return false;
                };
                let now = u32::from(t.hour) * 60 + u32::from(t.minute);
                let (inside, day) = if from <= to {
                    (now >= from && now < to, t.weekday)
                } else if now >= from {
                    (true, t.weekday)
                } else {
                    // Past midnight: the window began the day before.
                    (now < to, (t.weekday + 6) % 7)
                };
                inside && (self.days.is_empty() || self.days.contains(&day))
            }),
            Source::Idle => snapshot
                .idle_seconds
                .is_some_and(|s| s >= self.minutes.saturating_mul(60)),
            Source::Locked => snapshot.locked == Some(true),
        }
    }

    fn app_matches(&self, snapshot: &Snapshot) -> bool {
        CardTrigger {
            source: match self.source {
                Source::Foreground => TriggerSource::Foreground,
                Source::Process => TriggerSource::Process,
                Source::MediaPlaying => TriggerSource::MediaPlaying,
                _ => return false,
            },
            app: self.app.clone(),
            face: 0,
            priority: self.priority,
            return_seconds: self.return_seconds,
            return_face: None,
            interval_seconds: None,
            next_face: false,
        }
        .matches(snapshot)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub themes: BTreeMap<String, String>,
    pub rules: Vec<Rule>,
    /// How the rules combine; `any` is omitted.
    #[serde(rename = "match", skip_serializing_if = "is_any")]
    pub combine: Combine,
}

impl<'de> Deserialize<'de> for Profile {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Stored {
            id: String,
            name: String,
            #[serde(default)]
            themes: BTreeMap<String, String>,
            #[serde(default)]
            screen: Option<String>,
            #[serde(default)]
            theme: Option<String>,
            rules: Vec<Rule>,
            #[serde(default, rename = "match")]
            combine: Combine,
        }
        let mut stored = Stored::deserialize(deserializer)?;
        match (stored.screen, stored.theme) {
            (Some(screen), Some(theme)) if stored.themes.is_empty() => {
                stored.themes.insert(screen, theme);
            }
            (None, None) => {}
            _ => return Err(serde::de::Error::custom("conflicting profile assignments")),
        }
        Ok(Self {
            id: stored.id,
            name: stored.name,
            themes: stored.themes,
            rules: stored.rules,
            combine: stored.combine,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Config {
    /// Shared effect used for manual, automatic and fallback changes.
    pub transition: TransitionSettings,
    pub profiles: Vec<Profile>,
    pub automatic: bool,
    pub manual: Option<String>,
    pub default_profile: Option<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            transition: TransitionSettings::default(),
            profiles: vec![],
            automatic: true,
            manual: None,
            default_profile: None,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        self.transition.cover()?;
        let mut ids = BTreeSet::new();
        if self.profiles.len() > 32 {
            return Err(invalid("too many profiles"));
        }
        for p in &self.profiles {
            if p.id.is_empty()
                || ["auto", "default", "manage"].contains(&p.id.as_str())
                || p.id.len() > 64
                || !p.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
                || !ids.insert(&p.id)
                || p.name.trim().is_empty()
                || p.name.len() > 160
                || p.themes.is_empty()
                || p.themes.len() > crate::MAX_SCENES
                || p.themes.iter().any(|(screen, theme)| {
                    screen.is_empty()
                        || screen.len() > 256
                        || theme.is_empty()
                        || theme.len() > 32768
                })
                || p.rules.len() > 16
            {
                return Err(invalid("invalid profile"));
            }
            if p.rules.iter().any(|r| !r.valid()) {
                return Err(invalid("invalid profile rule"));
            }
        }
        for id in [&self.manual, &self.default_profile].into_iter().flatten() {
            if !ids.contains(id) {
                return Err(invalid("unknown profile selection"));
            }
        }
        Ok(())
    }
    pub fn select(&mut self, id: Option<&str>, automatic: bool) -> Result<()> {
        let mut next = self.clone();
        next.automatic = automatic;
        next.manual = id.map(str::to_owned);
        next.validate()?;
        *self = next;
        Ok(())
    }
    fn wanted(&self) -> Wanted {
        if !self.automatic {
            return Wanted::nothing();
        }
        self.profiles
            .iter()
            .flat_map(|p| &p.rules)
            .filter_map(|r| SensorKey::new(r.needs()?))
            .collect()
    }
}

/// Whole-profile cover settings; existing configurations default to no motion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TransitionSettings {
    /// none, curtain, wipe, blinds or mosaic.
    #[serde(deserialize_with = "effect_name")]
    pub effect: String,
    /// Left, right, up or down.
    pub direction: String,
    /// Closing plus opening in milliseconds, 150 to 3000.
    pub duration_ms: u32,
}
impl Default for TransitionSettings {
    fn default() -> Self {
        Self {
            effect: "none".into(),
            direction: "left".into(),
            duration_ms: 1500,
        }
    }
}
/// The whole-scene fade, slide and flip of earlier versions moved pixels the
/// host never has under a video the screen plays itself; a saved one becomes
/// the cover closest to it.
fn effect_name<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<String, D::Error> {
    let name = String::deserialize(deserializer)?;
    Ok(match name.as_str() {
        "fade" => "mosaic",
        "slide" => "wipe",
        "flip" => "curtain",
        other => other,
    }
    .to_owned())
}
impl TransitionSettings {
    /// The cover to draw, `None` for an immediate change.
    pub(crate) fn cover(&self) -> Result<Option<Cover>> {
        let effect = match self.effect.as_str() {
            "none" => None,
            "curtain" => Some(CoverEffect::Curtain),
            "wipe" => Some(CoverEffect::Wipe),
            "blinds" => Some(CoverEffect::Blinds),
            "mosaic" => Some(CoverEffect::Mosaic),
            _ => return Err(invalid("unknown profile transition effect")),
        };
        let direction = match self.direction.as_str() {
            "left" => CoverDirection::Left,
            "right" => CoverDirection::Right,
            "up" => CoverDirection::Up,
            "down" => CoverDirection::Down,
            _ => return Err(invalid("unknown profile transition direction")),
        };
        if !(150..=3000).contains(&self.duration_ms) {
            return Err(invalid(
                "profile transition duration must be 150 to 3000 ms",
            ));
        }
        Ok(effect.map(|effect| Cover {
            effect,
            direction,
            panel: cover::DEFAULT_PANEL,
            accent: cover::DEFAULT_ACCENT,
        }))
    }
}

/// What a screen trying a profile from the profile manager says it shows; no
/// saved profile has this id (ids are letters, digits and `-`).
pub(crate) const PREVIEW_ID: &str = "__preview__";
/// How long a tried profile stays on show before the screen returns to what
/// it should show, with the same cover.
pub(crate) const PREVIEW_HOLD: Duration = Duration::from_secs(3);

/// The theme at `path` as a document the engine accepts.
pub(crate) fn load_document(path: &str) -> Option<Document> {
    bezel_themes::FsThemeStore
        .load(&ThemeLocation(path.to_owned()))
        .ok()
        .map(|(theme, assets)| Document { theme, assets })
        .filter(|d| d.validate().is_ok())
}

/// Whether `document` can be shown on `scene`'s screen (its canvas, when the
/// screen is not known yet).
pub(crate) fn fits(scene: &crate::Scene, document: &Document) -> bool {
    scene
        .screen
        .as_ref()
        .and_then(|s| s.model())
        .map_or(document.theme.canvas == scene.runtime.theme().canvas, |m| {
            document.theme.misfit(m.panel).is_none()
        })
}

/// Where a profile change stands on one screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// The cover closes over the old theme; the new one is not applied yet.
    Closing,
    /// Closed: the new theme is applied and its first frame is being drawn.
    Covered,
    /// The cover opens on the new theme's frames.
    Opening,
}

pub(crate) struct Animation {
    /// The profile applied once the cover is closed (`None`: the selected themes).
    pub target: Option<String>,
    pub phase: Phase,
    pub started: Duration,
    /// Coverage the closing starts from: a cover still moving when another
    /// profile is chosen closes from where it is.
    pub base: f32,
    pub cover: Cover,
    /// Closing and opening each take half the duration; loading the new theme
    /// happens in between and shortens neither.
    pub half: Duration,
    /// False for a preview: no profile is applied when it closes.
    pub apply: bool,
    /// A preview's theme to show once closed (`None`: it opens on the same).
    pub show: Option<Document>,
}
impl Animation {
    fn closing(
        target: Option<String>,
        cover: Cover,
        duration_ms: u32,
        now: Duration,
        base: f32,
    ) -> Self {
        Self {
            target,
            phase: Phase::Closing,
            started: now,
            base: base.clamp(0.0, 1.0),
            cover,
            half: Duration::from_millis(u64::from(duration_ms / 2)),
            apply: true,
            show: None,
        }
    }
    /// The cover the profile manager tries out: it closes over what the
    /// screen shows and opens on `show` (or the same theme), heading to the
    /// profile shown so that the service does not step in while it moves.
    pub fn preview(
        shown: Option<String>,
        cover: Cover,
        duration_ms: u32,
        now: Duration,
        base: f32,
        show: Option<Document>,
    ) -> Self {
        Self {
            apply: false,
            show,
            ..Self::closing(shown, cover, duration_ms, now, base)
        }
    }
    fn elapsed(&self, now: Duration) -> f32 {
        now.saturating_sub(self.started).as_secs_f32() / self.half.as_secs_f32().max(0.001)
    }
    pub fn coverage(&self, now: Duration) -> f32 {
        match self.phase {
            Phase::Closing => self.base + (1.0 - self.base) * cover::ease(self.elapsed(now)),
            Phase::Covered => 1.0,
            Phase::Opening => 1.0 - cover::ease(self.elapsed(now)),
        }
    }
    pub fn stage(&self) -> CoverStage {
        if self.phase == Phase::Opening {
            CoverStage::Opening
        } else {
            CoverStage::Closing
        }
    }
    pub fn closed(&self, now: Duration) -> bool {
        self.phase == Phase::Closing && (self.base >= 1.0 || self.elapsed(now) >= 1.0)
    }
    /// Starts the opening at the new theme's first frame.
    pub fn open(&mut self, now: Duration) {
        if self.phase == Phase::Covered {
            self.phase = Phase::Opening;
            self.started = now;
        }
    }
    pub fn finished(&self, now: Duration) -> bool {
        self.phase == Phase::Opening && self.elapsed(now) >= 1.0
    }
}

#[derive(Clone)]
pub struct ProfileFile(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Display {
    pub key: String,
    pub name: String,
    pub theme: Option<String>,
    pub theme_name: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Overview {
    pub config: Config,
    pub displays: Vec<Display>,
    /// What the profiles do now, when the engine runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}
/// Native tray strings, also used while no WebView is running.
pub fn menu_labels(language: bezel_core::domain::clock::Language) -> [&'static str; 4] {
    use bezel_core::domain::clock::Language;
    match language {
        Language::Italian => ["Profili", "Automatico", "Default", "Gestisci profili…"],
        Language::PortugueseBr => ["Perfis", "Automático", "Default", "Gerenciar perfis…"],
        Language::English => ["Profiles", "Automatic", "Default", "Manage profiles…"],
    }
}
impl ProfileFile {
    pub fn beside(settings: &Path) -> Self {
        Self(settings.with_file_name("profiles.json"))
    }
    pub fn overview(&self) -> Result<Overview> {
        let config = self.load()?;
        let settings = self.0.with_file_name("settings.json");
        let settings = std::fs::metadata(&settings)
            .ok()
            .filter(|m| m.len() <= 1024 * 1024)
            .and_then(|_| std::fs::read(settings).ok())
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
            .unwrap_or_default();
        let mut keys: BTreeSet<String> = config
            .profiles
            .iter()
            .flat_map(|p| p.themes.keys().cloned())
            .collect();
        if let Some(themes) = settings["screenThemes"].as_object() {
            keys.extend(themes.keys().cloned());
        }
        if let Some(key) = settings["selectedScreen"].as_str() {
            keys.insert(key.into());
        }
        if let Some(live) = settings["liveScreens"].as_array() {
            keys.extend(live.iter().filter_map(|v| v.as_str().map(str::to_owned)));
        }
        let displays = keys
            .into_iter()
            .map(|key| {
                let theme = settings["screenThemes"][&key]
                    .as_str()
                    .or_else(|| {
                        (settings["selectedScreen"].as_str() == Some(&key))
                            .then(|| settings["lastTheme"].as_str())
                            .flatten()
                    })
                    .map(str::to_owned);
                let theme_name = theme.as_ref().and_then(|path| {
                    bezel_themes::native::load_manifest(&ThemeLocation(path.clone()))
                        .ok()
                        .map(|t| t.name)
                        .or_else(|| {
                            Path::new(path)
                                .file_stem()
                                .map(|s| s.to_string_lossy().into_owned())
                        })
                });
                Display {
                    name: key.clone(),
                    key,
                    theme,
                    theme_name,
                    connected: false,
                }
            })
            .collect();
        Ok(Overview {
            config,
            displays,
            status: None,
        })
    }
    pub fn load(&self) -> Result<Config> {
        let metadata = match std::fs::metadata(&self.0) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => return Err(invalid(&e.to_string())),
        };
        if metadata.len() > 1024 * 1024 {
            return Err(invalid("profile file too large"));
        }
        let bytes = std::fs::read(&self.0).map_err(|e| invalid(&e.to_string()))?;
        let config: Config =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid profile file"))?;
        config.validate()?;
        Ok(config)
    }
    pub fn save(&self, config: &Config) -> Result<()> {
        config.validate()?;
        if let Some(parent) = self.0.parent() {
            std::fs::create_dir_all(parent).map_err(|e| invalid(&e.to_string()))?;
        }
        let bytes = serde_json::to_vec_pretty(config).map_err(|e| invalid(&e.to_string()))?;
        let temporary = self.0.with_extension(format!("{}.tmp", std::process::id()));
        std::fs::write(&temporary, bytes)
            .and_then(|()| std::fs::rename(&temporary, &self.0))
            .map_err(|e| invalid(&e.to_string()))
    }
    pub fn select(&self, id: Option<&str>, automatic: bool) -> Result<()> {
        let mut config = self.load()?;
        config.select(id, automatic)?;
        self.save(&config)
    }
}

/// Why the active profile is on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Reason {
    /// Chosen by hand (Manual mode).
    Manual,
    /// These rules (indices) are true now.
    Rules { rules: Vec<usize> },
    /// Its rules stopped being true; it stays for its return delay.
    Returning { rules: Vec<usize> },
    /// No rule is true: the default profile.
    Default,
}

/// The profile on show and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Active {
    pub profile: String,
    pub reason: Reason,
}

/// What the profiles do now: the active profile and, for every profile,
/// which of its rules are true now (D-2026-10-20-studio-ux-v2-12).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub automatic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<Active>,
    /// Per profile id, `trueNow` of each rule in order.
    pub rules: BTreeMap<String, Vec<bool>>,
}

#[derive(Default)]
struct Selection {
    active: Option<String>,
    release: Option<Duration>,
    return_seconds: u32,
    priority: u32,
    /// The rules that made the active profile.
    reason: Vec<usize>,
    /// Whether the active profile's rules chose it this time (else it waits
    /// out its return delay).
    matched: bool,
    /// Since when each rule's condition has held, by profile id and index.
    since: BTreeMap<(String, usize), Duration>,
    status: Status,
}
impl Selection {
    /// Which rules are true now, after their wait: the conditions that hold
    /// and have held for their `for_seconds`.
    fn truths(
        &mut self,
        config: &Config,
        snapshot: &Snapshot,
        now: Duration,
        time: Option<&LocalTime>,
    ) -> BTreeMap<String, Vec<bool>> {
        let mut since = BTreeMap::new();
        let mut out = BTreeMap::new();
        for p in &config.profiles {
            let truths = p
                .rules
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    if !r.holds(snapshot, time) {
                        return false;
                    }
                    let key = (p.id.clone(), i);
                    let start = self.since.get(&key).copied().unwrap_or(now);
                    since.insert(key, start);
                    now.saturating_sub(start) >= Duration::from_secs(u64::from(r.for_seconds))
                })
                .collect();
            out.insert(p.id.clone(), truths);
        }
        self.since = since;
        out
    }

    #[cfg(test)]
    fn choose(&mut self, config: &Config, snapshot: &Snapshot, now: Duration) -> Option<String> {
        self.choose_at(config, snapshot, now, None)
    }

    fn choose_at(
        &mut self,
        config: &Config,
        snapshot: &Snapshot,
        now: Duration,
        time: Option<&LocalTime>,
    ) -> Option<String> {
        let truths = self.truths(config, snapshot, now, time);
        let chosen = self.pick(config, snapshot, now, time, &truths);
        let reason = match (&chosen, config.automatic) {
            (None, _) => None,
            (Some(_), false) => Some(Reason::Manual),
            (Some(id), true) if self.active.as_ref() == Some(id) => {
                let rules = self.reason.clone();
                Some(if self.matched {
                    Reason::Rules { rules }
                } else {
                    Reason::Returning { rules }
                })
            }
            (Some(_), true) => Some(Reason::Default),
        };
        self.status = Status {
            automatic: config.automatic,
            active: chosen
                .clone()
                .zip(reason)
                .map(|(profile, reason)| Active { profile, reason }),
            rules: truths,
        };
        chosen
    }

    fn pick(
        &mut self,
        config: &Config,
        snapshot: &Snapshot,
        now: Duration,
        time: Option<&LocalTime>,
        truths: &BTreeMap<String, Vec<bool>>,
    ) -> Option<String> {
        self.matched = false;
        if !config.automatic {
            self.active = config.manual.as_ref().cloned();
            self.release = None;
            return self.active.clone();
        }
        // Each candidate: the profile, its priority, its return delay and the rules that select it.
        let mut best: Option<(&Profile, u32, u32, Vec<usize>)> = None;
        for p in &config.profiles {
            let true_now = truths.get(&p.id).map(Vec::as_slice).unwrap_or_default();
            let candidates: Vec<(u32, u32, Vec<usize>)> = match p.combine {
                Combine::Any => p
                    .rules
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| true_now.get(*i) == Some(&true))
                    .map(|(i, r)| (r.priority, r.return_seconds, vec![i]))
                    .collect(),
                Combine::All if !p.rules.is_empty() && true_now.iter().all(|t| *t) => vec![(
                    p.rules.iter().map(|r| r.priority).max().unwrap_or(0),
                    p.rules.iter().map(|r| r.return_seconds).max().unwrap_or(0),
                    (0..p.rules.len()).collect(),
                )],
                Combine::All => Vec::new(),
            };
            for (priority, back, rules) in candidates {
                // Stable saved order wins ties.
                if best.as_ref().is_none_or(|b| priority > b.1) {
                    best = Some((p, priority, back, rules));
                }
            }
        }
        if let Some((p, priority, back, rules)) = best {
            if self.active.as_ref().is_some_and(|id| id != &p.id) && priority < self.priority {
                let until = *self
                    .release
                    .get_or_insert(now + Duration::from_secs(u64::from(self.return_seconds)));
                if now < until {
                    return self.active.clone();
                }
            }
            self.active = Some(p.id.clone());
            self.return_seconds = back;
            self.priority = priority;
            self.reason = rules;
            self.matched = true;
            self.release = None;
        } else if let Some(profile) = self
            .active
            .as_ref()
            .and_then(|id| config.profiles.iter().find(|p| &p.id == id))
        {
            if profile.rules.iter().any(|r| !r.available(snapshot, time)) {
                return self.active.clone();
            }
            let until = *self
                .release
                .get_or_insert(now + Duration::from_secs(u64::from(self.return_seconds)));
            if now >= until {
                self.active = None;
                self.release = None;
            }
        } else {
            self.active = None;
            self.release = None;
        }
        self.active.clone().or_else(|| {
            config
                .default_profile
                .as_ref()
                .filter(|id| config.profiles.iter().any(|p| &p.id == *id))
                .cloned()
        })
    }
}

pub(crate) struct Service {
    file: ProfileFile,
    config: Config,
    stamp: Option<SystemTime>,
    poll: Duration,
    selection: Selection,
    documents: BTreeMap<(String, String), Option<Document>>,
}
impl Service {
    pub fn new(file: ProfileFile) -> Self {
        Self {
            file,
            config: Config::default(),
            stamp: None,
            poll: Duration::ZERO,
            selection: Selection::default(),
            documents: BTreeMap::new(),
        }
    }
    pub fn wanted(&self) -> Wanted {
        self.config.wanted()
    }
    /// What the profiles do now.
    pub fn status(&self) -> Status {
        self.selection.status.clone()
    }
    pub fn tick(
        &mut self,
        engine: &mut Engine,
        snapshot: &Snapshot,
        now: Duration,
        time: LocalTime,
    ) -> bool {
        let mut changed = false;
        let poll = now >= self.poll;
        if poll {
            self.poll = now + Duration::from_secs(1);
            let stamp = std::fs::metadata(&self.file.0)
                .ok()
                .and_then(|m| m.modified().ok());
            if stamp != self.stamp
                && let Ok(config) = self.file.load()
            {
                changed = true;
                self.config = config;
                self.selection = Selection::default();
                self.documents.clear();
                self.stamp = stamp;
            }
        }
        // Re-entering a profile after editing must read its freshly saved theme.
        // Keep only active documents (and those a closing cover heads to);
        // failed reads retry at most once a second.
        self.documents.retain(|(id, screen), document| {
            if document.is_none() {
                return !poll;
            }
            engine.scenes.get(screen).is_some_and(|scene| {
                scene.profile_id.as_ref() == Some(id)
                    || scene
                        .profile_animation
                        .as_ref()
                        .is_some_and(|a| a.target.as_ref() == Some(id))
            })
        });
        let config_changed = changed;
        let mut budget: usize = engine.scenes.values().map(scene_bytes).sum();
        let selected = self
            .selection
            .choose_at(&self.config, snapshot, now, Some(&time));
        let cover = self.config.transition.cover().ok().flatten();
        let duration_ms = self.config.transition.duration_ms;
        for (key, scene) in &mut engine.scenes {
            // A profile being tried stays on show for its hold, then returns.
            if scene.preview_until.is_some_and(|until| now < until) {
                continue;
            }
            let id = selected.clone().filter(|id| {
                self.config
                    .profiles
                    .iter()
                    .any(|p| &p.id == id && p.themes.contains_key(key))
            });
            // Editing pauses profile output. Its pixels never replace the editor's
            // published frame, so reopening Studio first sees its own document.
            let viewed = engine.images.viewed(key);
            let id = id.filter(|_| {
                scene.live
                    && !viewed
                    && scene
                        .profile_edit_at
                        .is_none_or(|at| at.elapsed() >= Duration::from_secs(2))
            });
            // A closing cover already heads to the profile it will apply.
            let heading = match &scene.profile_animation {
                Some(a) if a.phase == Phase::Closing => a.target.clone(),
                _ => scene.profile_id.clone(),
            };
            if config_changed || heading != id {
                // The cover paints opaque pixels only, so it also hides a video
                // the screen plays itself. It waits for a theme that can be shown.
                let animate = scene.live
                    && !viewed
                    && scene.frame.is_some()
                    && (heading.is_some() || id.is_some())
                    && id
                        .as_deref()
                        .is_none_or(|id| self.document(id, key, scene).is_some());
                match cover.filter(|_| animate) {
                    Some(cover) => {
                        let base = scene
                            .profile_animation
                            .as_ref()
                            .map_or(0.0, |a| a.coverage(now));
                        scene.profile_animation =
                            Some(Animation::closing(id, cover, duration_ms, now, base));
                        scene.dirty = true;
                    }
                    None => changed |= self.apply(key, scene, id, &mut budget),
                }
            }
            if scene
                .profile_animation
                .as_ref()
                .is_some_and(|a| a.apply && a.closed(now))
                && let Some(mut animation) = scene.profile_animation.take()
            {
                changed |= self.apply(key, scene, animation.target.clone(), &mut budget);
                animation.phase = Phase::Covered;
                scene.profile_animation = Some(animation);
                scene.dirty = true;
            }
        }
        changed
    }

    /// The profile's theme for the screen `key`, read once and cached, when
    /// it fits the screen.
    fn document(&mut self, id: &str, key: &str, scene: &crate::Scene) -> Option<&Document> {
        let cache_key = (id.to_owned(), key.to_owned());
        if !self.documents.contains_key(&cache_key) {
            let document = self
                .config
                .profiles
                .iter()
                .find(|p| p.id == id)
                .and_then(|p| p.themes.get(key))
                .and_then(|path| load_document(path));
            let cached_bytes: usize = self.documents.values().flatten().map(Document::bytes).sum();
            let document = document
                .filter(|d| cached_bytes.saturating_add(d.bytes()) <= crate::MAX_ALL_ASSETS / 2);
            self.documents.insert(cache_key.clone(), document);
        }
        self.documents
            .get(&cache_key)
            .and_then(Option::as_ref)
            .filter(|document| fits(scene, document))
    }

    /// Shows the profile `id` (or the selected themes) on the screen at once.
    /// A cover in progress is kept: it is the one hiding this change.
    fn apply(
        &mut self,
        key: &str,
        scene: &mut crate::Scene,
        id: Option<String>,
        budget: &mut usize,
    ) -> bool {
        let animation = scene.profile_animation.take();
        let before = scene_bytes(scene);
        scene.restore_profile();
        *budget = budget
            .saturating_sub(before)
            .saturating_add(scene_bytes(scene));
        let mut changed = false;
        match id {
            Some(id) => {
                if let Some(document) = self.document(&id, key, scene).cloned()
                    && budget.saturating_add(document.bytes()) <= crate::MAX_ALL_ASSETS
                {
                    scene.profile_restore = Some(Document {
                        theme: scene.runtime.theme().clone(),
                        assets: scene.runtime.assets().clone(),
                    });
                    scene.profile_id = Some(id);
                    *budget += document.bytes();
                    scene.update(document);
                    scene.runtime.automatic_cards(true);
                    changed = true;
                }
            }
            None => changed = true,
        }
        scene.profile_animation = animation;
        changed
    }
}

fn scene_bytes(scene: &crate::Scene) -> usize {
    scene.runtime.assets().values().map(Vec::len).sum::<usize>()
        + scene.saved.bytes()
        + scene.profile_restore.as_ref().map_or(0, Document::bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transitions_are_backward_compatible_and_bounded() {
        let mut config: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(config.transition.effect, "none");
        for effect in ["none", "curtain", "wipe", "blinds", "mosaic"] {
            config.transition.effect = effect.into();
            config.validate().unwrap();
            let saved = serde_json::to_vec(&config).unwrap();
            assert_eq!(serde_json::from_slice::<Config>(&saved).unwrap(), config);
        }
        // Whole-scene effects of earlier versions load as the nearest cover.
        for (old, new) in [("fade", "mosaic"), ("slide", "wipe"), ("flip", "curtain")] {
            let saved = format!(
                r#"{{"transition":{{"effect":"{old}","direction":"down","durationMs":600}}}}"#
            );
            let loaded: Config = serde_json::from_str(&saved).unwrap();
            loaded.validate().unwrap();
            assert_eq!(
                (
                    loaded.transition.effect.as_str(),
                    loaded.transition.direction.as_str()
                ),
                (new, "down")
            );
        }
        config.transition.duration_ms = 149;
        assert!(config.validate().is_err());
        config.transition.duration_ms = 3001;
        assert!(config.validate().is_err());
        config.transition.duration_ms = 600;
        config.transition.direction = "invalid".into();
        assert!(config.validate().is_err());
        config.transition.direction = "left".into();
        config.transition.effect = "invalid".into();
        assert!(config.validate().is_err());
    }
    #[test]
    fn a_cover_closes_and_opens_in_halves_around_the_first_new_frame() {
        let cover = Cover {
            effect: CoverEffect::Curtain,
            direction: CoverDirection::Left,
            panel: cover::DEFAULT_PANEL,
            accent: cover::DEFAULT_ACCENT,
        };
        let ms = Duration::from_millis;
        let mut animation =
            Animation::closing(Some("music".into()), cover, 800, Duration::ZERO, 0.0);
        assert_eq!(animation.coverage(Duration::ZERO), 0.0);
        assert!(!animation.closed(ms(399)));
        assert!(animation.closed(ms(400)));
        assert_eq!(animation.coverage(ms(400)), 1.0);
        // Loading the theme takes 600 ms: it stays covered, the opening waits.
        animation.phase = Phase::Covered;
        assert_eq!(animation.coverage(ms(1000)), 1.0);
        animation.open(ms(1000));
        assert_eq!(
            (animation.phase, animation.stage()),
            (Phase::Opening, CoverStage::Opening)
        );
        assert_eq!(animation.coverage(ms(1000)), 1.0);
        assert!(!animation.finished(ms(1399)));
        assert!(animation.finished(ms(1400)));
        assert_eq!(animation.coverage(ms(1400)), 0.0);
        // Another profile chosen mid-opening closes again from where it stands.
        let again = Animation::closing(None, cover, 800, ms(1200), animation.coverage(ms(1200)));
        assert!((again.coverage(ms(1200)) - 0.5).abs() < 1e-6);
        assert!(!again.closed(ms(1200)));
    }
    #[test]
    fn legacy_single_screen_profiles_migrate_without_losing_their_assignment() {
        let old = r#"{"profiles":[{"id":"gaming","name":"Gaming","screen":"COM3","theme":"wide.bezeltheme","rules":[]}],"automatic":false,"manual":"gaming"}"#;
        let config: Config = serde_json::from_str(old).unwrap();
        config.validate().unwrap();
        assert_eq!(config.profiles[0].themes["COM3"], "wide.bezeltheme");
        let saved = serde_json::to_value(&config).unwrap();
        assert!(saved["profiles"][0].get("screen").is_none());
        assert!(saved["profiles"][0].get("theme").is_none());
        assert_eq!(saved["profiles"][0]["themes"]["COM3"], "wide.bezeltheme");
        let duplicate = r#"{"id":"bad","name":"Bad","screen":"COM3","theme":"old","themes":{"COM4":"new"},"rules":[]}"#;
        assert!(serde_json::from_str::<Profile>(duplicate).is_err());
    }
    fn config() -> Config {
        Config {
            profiles: vec![
                Profile {
                    id: "music".into(),
                    name: "Music".into(),
                    themes: BTreeMap::from([("screen".into(), "music.bezeltheme".into())]),
                    rules: vec![Rule {
                        priority: 10,
                        return_seconds: 3,
                        ..Rule::of(Source::MediaPlaying)
                    }],
                    combine: Combine::Any,
                },
                Profile {
                    id: "game".into(),
                    name: "Game".into(),
                    themes: BTreeMap::from([("screen".into(), "game.bezeltheme".into())]),
                    rules: vec![Rule {
                        app: "Game.exe".into(),
                        priority: 20,
                        return_seconds: 2,
                        ..Rule::of(Source::Foreground)
                    }],
                    combine: Combine::Any,
                },
            ],
            ..Config::default()
        }
    }
    #[test]
    fn manual_selection_is_stable_and_automatic_restores_the_rules() {
        let mut c = config();
        c.select(Some("music"), false).unwrap();
        let mut s = Snapshot::default();
        s.foreground = Some("game".into());
        s.activity_available = true;
        let mut choice = Selection::default();
        assert_eq!(
            choice.choose(&c, &s, Duration::ZERO).as_deref(),
            Some("music")
        );
        c.select(None, true).unwrap();
        assert_eq!(
            choice.choose(&c, &s, Duration::ZERO).as_deref(),
            Some("game")
        );
    }
    #[test]
    fn priorities_delay_and_missing_measurements_preserve_the_expected_profile() {
        let c = config();
        let mut choice = Selection::default();
        let mut s = Snapshot::default();
        s.foreground = Some("game.exe".into());
        s.activity_available = true;
        s.media_available = true;
        assert_eq!(
            choice.choose(&c, &s, Duration::ZERO).as_deref(),
            Some("game")
        );
        s.foreground = None;
        assert_eq!(
            choice.choose(&c, &s, Duration::from_secs(1)).as_deref(),
            Some("game")
        );
        assert_eq!(
            choice.choose(&c, &s, Duration::from_secs(2)).as_deref(),
            Some("game")
        );
        assert_eq!(choice.choose(&c, &s, Duration::from_secs(3)), None);
        s.foreground = Some("game".into());
        choice.choose(&c, &s, Duration::from_secs(4));
        s.activity_available = false;
        assert_eq!(
            choice.choose(&c, &s, Duration::from_secs(40)).as_deref(),
            Some("game")
        );
    }
    #[test]
    fn invalid_configuration_is_rejected_without_changing_selection() {
        let mut c = config();
        assert!(c.select(Some("missing"), false).is_err());
        assert!(c.automatic);
        c.profiles[0].rules[0].app.clear();
        assert!(c.validate().is_ok());
        c.profiles[1].rules[0].app.clear();
        assert!(c.validate().is_err());
    }

    #[test]
    fn gaming_overrides_music_then_returns_after_its_delay_and_ties_keep_saved_order() {
        use bezel_core::domain::playback::MediaSession;
        let mut c = config();
        let mut snapshot = Snapshot::default();
        snapshot.activity_available = true;
        snapshot.media_available = true;
        snapshot.media.push(MediaSession {
            source: "Spotify".into(),
            title: "Track".into(),
            artist: String::new(),
            playing: true,
            position: 0.,
            duration: 100.,
            cover: vec![],
        });
        let mut choice = Selection::default();
        assert_eq!(
            choice.choose(&c, &snapshot, Duration::ZERO).as_deref(),
            Some("music")
        );
        snapshot.foreground = Some("GAME.EXE".into());
        assert_eq!(
            choice
                .choose(&c, &snapshot, Duration::from_secs(1))
                .as_deref(),
            Some("game")
        );
        snapshot.foreground = None;
        assert_eq!(
            choice
                .choose(&c, &snapshot, Duration::from_secs(2))
                .as_deref(),
            Some("game")
        );
        assert_eq!(
            choice
                .choose(&c, &snapshot, Duration::from_secs(3))
                .as_deref(),
            Some("game")
        );
        assert_eq!(
            choice
                .choose(&c, &snapshot, Duration::from_secs(4))
                .as_deref(),
            Some("music")
        );
        snapshot.foreground = Some("game".into());
        c.profiles[1].rules[0].priority = 10;
        assert_eq!(
            Selection::default()
                .choose(&c, &snapshot, Duration::ZERO)
                .as_deref(),
            Some("music")
        );
    }
    #[test]
    fn file_round_trip_and_selection_survive_a_restart() {
        let directory = std::env::temp_dir().join(format!("bezel-profiles-{}", std::process::id()));
        let file = ProfileFile(directory.join("profiles.json"));
        file.save(&config()).unwrap();
        file.select(Some("music"), false).unwrap();
        let loaded = file.load().unwrap();
        assert!(!loaded.automatic);
        assert_eq!(loaded.manual.as_deref(), Some("music"));
        std::fs::remove_file(&file.0).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    fn profile(id: &str, combine: Combine, rules: Vec<Rule>) -> Profile {
        Profile {
            id: id.into(),
            name: id.into(),
            themes: BTreeMap::from([("screen".into(), format!("{id}.bezeltheme"))]),
            rules,
            combine,
        }
    }

    fn sensor(key: &str, compare: Compare, value: f64, for_seconds: u32) -> Rule {
        Rule {
            key: key.into(),
            compare: Some(compare),
            value: Some(value),
            for_seconds,
            priority: 50,
            ..Rule::of(Source::Sensor)
        }
    }

    fn reading(key: &str, value: f64) -> Snapshot {
        let mut s = Snapshot::default();
        s.insert(
            SensorKey::new(key).unwrap(),
            bezel_core::domain::sensor::Reading::Value(value),
        );
        s
    }

    fn at(weekday: u8, hour: u8, minute: u8) -> LocalTime {
        LocalTime {
            year: 2026,
            month: 10,
            day: 19 + weekday,
            hour,
            minute,
            second: 0,
            weekday,
            millisecond: 0,
        }
    }

    #[test]
    fn a_sensor_rule_counts_once_it_has_held_long_enough() {
        let c = Config {
            profiles: vec![profile(
                "hot",
                Combine::Any,
                vec![sensor("cpu.temperature", Compare::Above, 80.0, 10)],
            )],
            ..Config::default()
        };
        c.validate().unwrap();
        let mut choice = Selection::default();
        let s = Duration::from_secs;
        assert_eq!(
            choice.choose(&c, &reading("cpu.temperature", 85.0), s(0)),
            None
        );
        assert_eq!(
            choice.status.rules["hot"],
            vec![false],
            "not yet ten seconds"
        );
        assert_eq!(
            choice.choose(&c, &reading("cpu.temperature", 85.0), s(9)),
            None
        );
        assert_eq!(
            choice
                .choose(&c, &reading("cpu.temperature", 86.0), s(10))
                .as_deref(),
            Some("hot")
        );
        assert_eq!(
            choice.status.active,
            Some(Active {
                profile: "hot".into(),
                reason: Reason::Rules { rules: vec![0] }
            })
        );
        assert_eq!(
            choice.choose(&c, &reading("cpu.temperature", 70.0), s(11)),
            None
        );
        choice.choose(&c, &reading("cpu.temperature", 90.0), s(12));
        assert_eq!(
            choice.choose(&c, &reading("cpu.temperature", 90.0), s(20)),
            None,
            "the wait starts again after a dip"
        );
        let below = sensor("cpu.usage", Compare::Below, 10.0, 0);
        assert!(below.holds(&reading("cpu.usage", 5.0), None));
        assert!(!below.holds(&reading("cpu.usage", 15.0), None));
        assert!(!below.holds(&Snapshot::default(), None), "no reading");
    }

    #[test]
    fn time_rules_follow_days_and_windows_past_midnight() {
        let evening = Rule {
            days: vec![0, 1, 2, 3, 4],
            from: "18:00".into(),
            to: "23:30".into(),
            ..Rule::of(Source::Time)
        };
        assert!(evening.valid());
        assert!(evening.holds(&Snapshot::default(), Some(&at(1, 19, 0))));
        assert!(
            !evening.holds(&Snapshot::default(), Some(&at(1, 23, 30))),
            "the end is out"
        );
        assert!(
            !evening.holds(&Snapshot::default(), Some(&at(5, 19, 0))),
            "Saturday"
        );
        assert!(!evening.holds(&Snapshot::default(), None), "no clock");
        let night = Rule {
            days: vec![4],
            from: "22:00".into(),
            to: "02:00".into(),
            ..Rule::of(Source::Time)
        };
        assert!(
            night.holds(&Snapshot::default(), Some(&at(4, 23, 0))),
            "Friday night"
        );
        assert!(
            night.holds(&Snapshot::default(), Some(&at(5, 1, 0))),
            "still the night of Friday"
        );
        assert!(
            !night.holds(&Snapshot::default(), Some(&at(4, 1, 0))),
            "the night of Thursday"
        );
        let every_day = Rule {
            days: vec![],
            ..night.clone()
        };
        assert!(every_day.holds(&Snapshot::default(), Some(&at(2, 23, 0))));
        for bad in ["24:00", "7:00", "aa:bb", ""] {
            assert!(
                !Rule {
                    from: bad.into(),
                    ..night.clone()
                }
                .valid(),
                "{bad}"
            );
        }
        assert!(
            !Rule {
                days: vec![7],
                ..night
            }
            .valid()
        );
    }

    #[test]
    fn idle_and_locked_rules_read_presence() {
        let idle = Rule {
            minutes: 5,
            ..Rule::of(Source::Idle)
        };
        let mut s = Snapshot::default();
        assert!(!idle.holds(&s, None), "unknown");
        s.idle_seconds = Some(299);
        assert!(!idle.holds(&s, None));
        s.idle_seconds = Some(300);
        assert!(idle.holds(&s, None));
        let locked = Rule::of(Source::Locked);
        assert!(!locked.holds(&s, None));
        s.locked = Some(true);
        assert!(locked.holds(&s, None));
        assert!(locked.valid());
        assert!(!Rule::of(Source::Idle).valid(), "zero minutes");
        assert!(
            !Rule {
                minutes: 5000,
                ..idle
            }
            .valid()
        );
        let c = Config {
            profiles: vec![profile(
                "away",
                Combine::Any,
                vec![Rule::of(Source::Locked)],
            )],
            ..Config::default()
        };
        assert!(c.wanted().contains("activity.presence"));
        let hot = Config {
            profiles: vec![profile(
                "hot",
                Combine::Any,
                vec![sensor("cpu.usage", Compare::Above, 1.0, 0)],
            )],
            ..Config::default()
        };
        assert!(
            hot.wanted().contains("cpu.usage"),
            "a sensor rule wants its sensor"
        );
    }

    #[test]
    fn all_together_needs_every_rule_and_any_needs_one() {
        let rules = || {
            vec![
                sensor("gpu.usage", Compare::Above, 50.0, 0),
                Rule {
                    priority: 70,
                    return_seconds: 4,
                    ..Rule::of(Source::Locked)
                },
            ]
        };
        let c = Config {
            profiles: vec![profile("both", Combine::All, rules())],
            ..Config::default()
        };
        let mut s = reading("gpu.usage", 80.0);
        s.locked = Some(false);
        let mut choice = Selection::default();
        assert_eq!(choice.choose(&c, &s, Duration::ZERO), None);
        assert_eq!(choice.status.rules["both"], vec![true, false]);
        s.locked = Some(true);
        assert_eq!(
            choice.choose(&c, &s, Duration::ZERO).as_deref(),
            Some("both")
        );
        assert_eq!(
            choice.status.active.as_ref().map(|a| a.reason.clone()),
            Some(Reason::Rules { rules: vec![0, 1] })
        );
        assert_eq!(choice.priority, 70, "the highest of its rules");
        s.locked = Some(false);
        assert_eq!(
            choice.choose(&c, &s, Duration::from_secs(1)).as_deref(),
            Some("both"),
            "returns after the longest delay"
        );
        assert!(matches!(
            choice.status.active.as_ref().map(|a| &a.reason),
            Some(Reason::Returning { .. })
        ));
        assert_eq!(choice.choose(&c, &s, Duration::from_secs(5)), None);

        let any = Config {
            profiles: vec![profile("either", Combine::Any, rules())],
            ..Config::default()
        };
        assert_eq!(
            Selection::default()
                .choose(&any, &s, Duration::ZERO)
                .as_deref(),
            Some("either")
        );
        let empty = Config {
            profiles: vec![profile("never", Combine::All, vec![])],
            ..Config::default()
        };
        assert_eq!(
            Selection::default().choose(&empty, &s, Duration::ZERO),
            None
        );
    }

    #[test]
    fn the_status_names_manual_and_default_profiles() {
        let mut c = config();
        c.default_profile = Some("music".into());
        let mut choice = Selection::default();
        choice.choose(&c, &Snapshot::default(), Duration::ZERO);
        assert_eq!(
            choice
                .status
                .active
                .as_ref()
                .map(|a| (&a.profile[..], a.reason.clone())),
            Some(("music", Reason::Default))
        );
        c.select(Some("game"), false).unwrap();
        choice.choose(&c, &Snapshot::default(), Duration::ZERO);
        assert_eq!(
            choice.status.active.as_ref().map(|a| a.reason.clone()),
            Some(Reason::Manual)
        );
        assert!(!choice.status.automatic);
        let json = serde_json::to_value(&choice.status).unwrap();
        assert_eq!(json["active"]["reason"]["kind"], "manual");
        assert_eq!(json["rules"]["game"], serde_json::json!([false]));
    }

    #[test]
    fn new_rules_and_all_together_round_trip_and_old_files_still_read() {
        let mut c = config();
        c.profiles.push(profile(
            "work",
            Combine::All,
            vec![
                sensor("cpu.usage", Compare::Above, 60.0, 30),
                Rule {
                    days: vec![0, 4],
                    from: "09:00".into(),
                    to: "18:00".into(),
                    ..Rule::of(Source::Time)
                },
                Rule {
                    minutes: 10,
                    ..Rule::of(Source::Idle)
                },
            ],
        ));
        c.validate().unwrap();
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["profiles"][2]["match"], "all");
        assert!(json["profiles"][0].get("match").is_none(), "any is omitted");
        assert!(json["profiles"][0]["rules"][0].get("key").is_none());
        assert_eq!(json["profiles"][2]["rules"][0]["forSeconds"], 30);
        let back: Config = serde_json::from_value(json).unwrap();
        assert_eq!(back, c);
        let old = r#"{"profiles":[{"id":"g","name":"G","themes":{"s":"t"},"rules":[{"source":"foreground","app":"steam","priority":5,"returnSeconds":0}]}]}"#;
        let old: Config = serde_json::from_str(old).unwrap();
        assert_eq!(old.profiles[0].combine, Combine::Any);
        old.validate().unwrap();
        let mut bad = c.clone();
        bad.profiles[2].rules[0].value = Some(f64::NAN);
        assert!(bad.validate().is_err());
        bad.profiles[2].rules[0] = Rule {
            key: "not a key!".into(),
            ..sensor("x", Compare::Above, 1.0, 0)
        };
        assert!(bad.validate().is_err());
    }
}
