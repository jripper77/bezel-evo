//! Audio spectrum contracts; no operating-system APIs or audio persistence.
use super::{
    playback,
    sensor::Snapshot,
    theme::{ElementId, ElementKind, TextContent, Theme},
};

/// Motion preset, expressed independently from display frame rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    /// Slow fall and gentle attack.
    Smooth,
    /// Responsive attack with a readable fall.
    #[default]
    Balanced,
    /// Fast attack and fall.
    Snappy,
}
/// What a spectrum listens to (D-AS-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectrumAudio {
    /// Only the app of the session its Player shows (D-AS-2).
    #[default]
    Player,
    /// Everything the PC plays on its output, chosen explicitly by the user.
    System,
}
/// Capture source key of the PC's whole output. `*` never appears in an app
/// identifier or an executable name, so it cannot name an app.
pub const SYSTEM_SOURCE: &str = "*system*";
/// How the bands are drawn (D-AS-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpectrumStyle {
    /// Bars mirrored around a horizontal centre line (D-AS-4).
    #[default]
    Mirrored,
    /// Bars rising from the bottom edge.
    Bars,
    /// Bars made of stacked blocks, lit from the bottom.
    Blocks,
    /// A smooth curve through the bands, optionally filled below.
    Line,
    /// Bars radiating from a ring around the element's centre.
    Radial,
    /// A pixel character dancing to the beat of the bass.
    Dancer,
}
/// Who dances in a [`SpectrumStyle::Dancer`] spectrum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dancer {
    /// A little robot with a visor and an antenna.
    #[default]
    Robot,
    /// A kid with headphones.
    Headphones,
    /// A cat with a swinging tail.
    Cat,
}
/// The poses of a dancer, in the order of its sprites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DancePose {
    /// Standing, breathing: no music.
    Still,
    /// Knees bent, arms out: between two beats.
    Down,
    /// Arms up.
    ArmsUp,
    /// A step to the left.
    StepLeft,
    /// A step to the right.
    StepRight,
    /// In the air: a strong beat.
    Jump,
}
/// The moves a dancer takes in turn, one a beat.
pub const MOVES: [DancePose; 4] = [
    DancePose::ArmsUp,
    DancePose::StepLeft,
    DancePose::ArmsUp,
    DancePose::StepRight,
];
/// How long a move is held after its beat, seconds.
pub const BEAT_HELD: f32 = 0.28;
/// A beat at least this strong jumps.
pub const JUMP_STRENGTH: f32 = 0.8;
/// Loudest band below which the music counts as stopped.
pub const SILENCE: f32 = 0.04;
/// Beats a minute of the demo.
pub const DEMO_TEMPO: f32 = 120.;
/// The beat of the demo at `seconds`: steady, every eighth one strong.
pub fn demo_beat(seconds: f32) -> Beat {
    let beats = (seconds.max(0.) * DEMO_TEMPO / 60.).min(u32::MAX as f32);
    let count = beats as u32;
    Beat {
        count,
        since: beats.fract() * 60. / DEMO_TEMPO,
        strength: if count % 8 == 7 { 1. } else { 0.6 },
    }
}
/// The pose of a dancer for `frame`: a move just after a beat, a jump after a
/// strong one, down between beats, still without music.
pub fn dance_pose(frame: &SpectrumFrame) -> DancePose {
    let playing = matches!(frame.state, AudioState::Live | AudioState::Demo)
        && frame.levels.iter().any(|v| *v >= SILENCE);
    match frame.beat {
        _ if !playing => DancePose::Still,
        Some(beat) if beat.since < BEAT_HELD => {
            if beat.strength >= JUMP_STRENGTH {
                DancePose::Jump
            } else {
                MOVES[beat.count as usize % MOVES.len()]
            }
        }
        _ => DancePose::Down,
    }
}
/// Standalone spectrum following exactly one theme Player.
#[derive(Debug, Clone, PartialEq)]
pub struct Spectrum {
    /// Explicit Player; absent links automatically only with a single Player.
    pub player: Option<ElementId>,
    /// Logarithmic bands, 8..=64.
    pub bands: u16,
    /// Lower frequency, Hz.
    pub low: f32,
    /// Upper frequency, Hz.
    pub high: f32,
    /// Attack/fall preset.
    pub motion: Motion,
    /// Hold and decay peak caps.
    pub peaks: bool,
    /// Explicit synthetic preview; never enables capture.
    pub demo: bool,
    /// Maximum bar thickness in pixels.
    pub thickness: f32,
    /// Space between bars in pixels.
    pub gap: f32,
    /// Shared corner ordering: top-left, top-right, bottom-right, bottom-left.
    pub corners: [f32; 4],
    /// Draw the base line: centre line, bottom edge, or inner ring when radial.
    pub centre: bool,
    /// How the bands are drawn.
    pub style: SpectrumStyle,
    /// Block height in pixels, for `Blocks`.
    pub block: f32,
    /// Line width in pixels, for `Line`.
    pub stroke: f32,
    /// Fill the area below the curve, for `Line`.
    pub fill: bool,
    /// Inner ring radius as a percentage of the outer radius, for `Radial`.
    pub inner: f32,
    /// Multiplies every band's height, 0.1..=4 (D-AS-12).
    pub gain: f32,
    /// What it listens to.
    pub audio: SpectrumAudio,
    /// Who dances, for `Dancer`.
    pub dancer: Dancer,
}
impl Default for Spectrum {
    fn default() -> Self {
        Self {
            player: None,
            bands: 32,
            low: 60.,
            high: 16000.,
            motion: Motion::Balanced,
            peaks: true,
            demo: false,
            thickness: 12.,
            gap: 3.,
            corners: [2.; 4],
            centre: true,
            style: SpectrumStyle::Mirrored,
            block: 6.,
            stroke: 2.,
            fill: true,
            inner: 40.,
            gain: 1.,
            audio: SpectrumAudio::Player,
            dancer: Dancer::Robot,
        }
    }
}
impl Spectrum {
    /// Bounded even for theme documents coming from another computer.
    pub fn valid(&self) -> bool {
        (8..=64).contains(&self.bands)
            && self.low.is_finite()
            && self.high.is_finite()
            && self.low >= 20.
            && self.high <= 24000.
            && self.low < self.high
            && self.thickness.is_finite()
            && (1. ..=256.).contains(&self.thickness)
            && self.gap.is_finite()
            && (0. ..=128.).contains(&self.gap)
            && self
                .corners
                .iter()
                .all(|v| v.is_finite() && (0. ..=256.).contains(v))
            && (1. ..=64.).contains(&self.block)
            && (1. ..=32.).contains(&self.stroke)
            && (0. ..=90.).contains(&self.inner)
            && (0.1..=4.).contains(&self.gain)
    }
    /// The followed session, selected by the Player's own selection function.
    pub fn session<'a>(
        &self,
        theme: &Theme,
        snapshot: &'a Snapshot,
    ) -> std::result::Result<Option<&'a playback::MediaSession>, AudioState> {
        let players: Vec<_> = theme
            .elements
            .iter()
            .filter_map(|e| match &e.kind {
                ElementKind::Text {
                    content: TextContent::Player(p),
                    ..
                } => Some((e.id, p)),
                _ => None,
            })
            .collect();
        let player = match self.player {
            Some(id) => players.iter().find(|(key, _)| *key == id).map(|(_, p)| *p),
            None if players.len() == 1 => Some(players[0].1),
            _ => None,
        }
        .ok_or(AudioState::PlayerMissing)?;
        Ok(playback::session(snapshot, &player.source))
    }
}
/// Fixed, non-sensitive states suitable for localization and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioState {
    /// Capturing; silence is represented by zero bars.
    Live,
    /// User has not enabled local capture.
    #[default]
    Disabled,
    /// Player reference is absent, ambiguous or deleted.
    PlayerMissing,
    /// Session cannot be safely associated with one process tree.
    Unavailable,
    /// Process loopback requires Windows build 20348 or newer.
    Unsupported,
    /// Explicit synthetic preview.
    Demo,
}
/// Latest bounded display result. No PCM leaves the capture worker.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpectrumFrame {
    /// Capture or preview state.
    pub state: AudioState,
    /// Normalized -72..0 dB levels, 0..=1.
    pub levels: Vec<f32>,
    /// Normalized peak caps, 0..=1.
    pub peaks: Vec<f32>,
    /// The last beat found in the bass; none before the first.
    pub beat: Option<Beat>,
}
/// A beat found in the bass.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Beat {
    /// Beats found so far (wrapping).
    pub count: u32,
    /// Seconds since this one, at most [`Beat::LONGEST`].
    pub since: f32,
    /// How strong against the beats before it, 0..=1: about ⅔ for a usual
    /// one, 1 for one half again as strong.
    pub strength: f32,
}
impl Beat {
    /// `since` stops counting here: a frame does not change for an old beat.
    pub const LONGEST: f32 = 1.;
}
/// Ephemeral PCM packet; owned and dropped on the analysis thread.
pub struct AudioBlock {
    /// Interleaved finite normalized samples.
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub rate: u32,
    /// Interleaved channel count.
    pub channels: u16,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::{
        geometry::{Orientation, Size},
        playback::{MediaSession, Player},
        theme::{BoxF, Element, TextStyle},
    };
    fn player(id: u32, source: &str) -> Element {
        Element {
            id: ElementId(id),
            name: "Player".into(),
            frame: BoxF::default(),
            kind: ElementKind::Text {
                content: TextContent::Player(Player {
                    source: source.into(),
                    ..Default::default()
                }),
                style: TextStyle::default(),
            },
            opacity: 1.,
            visible: true,
            locked: false,
            card: None,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card_member: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            color_by_value: None,
        }
    }
    #[test]
    fn automatic_link_uses_the_players_exact_session_and_never_guesses_among_players() {
        let mut theme = Theme::blank("s", Size::new(100, 100), Orientation::Portrait);
        theme.elements.push(player(1, "Spotify"));
        let mut snapshot = Snapshot::default();
        snapshot.media = vec![
            MediaSession {
                source: "Chrome.exe".into(),
                playing: true,
                ..Default::default()
            },
            MediaSession {
                source: "Spotify.exe".into(),
                playing: false,
                ..Default::default()
            },
        ];
        let mut s = Spectrum::default();
        assert_eq!(
            s.session(&theme, &snapshot).unwrap().unwrap().source,
            "Spotify.exe"
        );
        theme.elements.push(player(2, "Chrome"));
        assert_eq!(s.session(&theme, &snapshot), Err(AudioState::PlayerMissing));
        s.player = Some(ElementId(2));
        assert_eq!(
            s.session(&theme, &snapshot).unwrap().unwrap().source,
            "Chrome.exe"
        );
        theme.elements.pop();
        assert_eq!(s.session(&theme, &snapshot), Err(AudioState::PlayerMissing));
    }
    #[test]
    fn a_dancer_moves_on_each_beat_jumps_on_a_strong_one_and_rests_in_silence() {
        let beat = |count, since, strength| SpectrumFrame {
            state: AudioState::Live,
            levels: vec![0.5; 8],
            peaks: vec![],
            beat: Some(Beat {
                count,
                since,
                strength,
            }),
        };
        assert_eq!(dance_pose(&beat(0, 0.1, 0.5)), DancePose::ArmsUp);
        assert_eq!(dance_pose(&beat(1, 0.1, 0.5)), DancePose::StepLeft);
        assert_eq!(dance_pose(&beat(3, 0.1, 0.5)), DancePose::StepRight);
        assert_eq!(dance_pose(&beat(7, 0.1, 0.9)), DancePose::Jump);
        assert_eq!(
            dance_pose(&beat(1, 0.4, 0.9)),
            DancePose::Down,
            "between beats"
        );
        let mut quiet = beat(1, 0.1, 0.5);
        quiet.levels = vec![0.01; 8];
        assert_eq!(dance_pose(&quiet), DancePose::Still);
        let mut off = beat(1, 0.1, 0.5);
        off.state = AudioState::Disabled;
        assert_eq!(dance_pose(&off), DancePose::Still);
        assert_eq!(
            dance_pose(&SpectrumFrame {
                beat: None,
                ..beat(0, 0., 0.)
            }),
            DancePose::Down,
            "music before the first beat"
        );
    }
    #[test]
    fn the_demo_beats_twice_a_second_and_jumps_every_eighth() {
        assert_eq!(
            demo_beat(0.),
            Beat {
                count: 0,
                since: 0.,
                strength: 0.6
            }
        );
        let b = demo_beat(1.6);
        assert_eq!((b.count, (b.since * 100.).round()), (3, 10.));
        assert_eq!(demo_beat(3.6).strength, 1.);
        assert_eq!(demo_beat(-1.).count, 0);
    }
    #[test]
    fn imported_settings_are_bounded() {
        let mut s = Spectrum::default();
        assert!(s.valid());
        s.high = f32::NAN;
        assert!(!s.valid());
        s.high = 16000.;
        s.low = 20000.;
        assert!(!s.valid());
        s.low = 60.;
        s.bands = u16::MAX;
        assert!(!s.valid());
        s.bands = 32;
        s.corners[0] = f32::INFINITY;
        assert!(!s.valid());
    }
}
