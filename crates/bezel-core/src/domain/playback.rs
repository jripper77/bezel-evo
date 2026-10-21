//! Application and media context shared by Studio and the unattended runtime.
use super::frame::Rgba;
use super::sensor::Snapshot;

/// Metadata published by an operating-system media session.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaSession {
    /// Application identifier or substring filter.
    pub source: String,
    /// Track title.
    pub title: String,
    /// Track artist.
    pub artist: String,
    /// True only during active playback.
    pub playing: bool,
    /// Position in seconds.
    pub position: f64,
    /// Duration in seconds.
    pub duration: f64,
    /// Bounded PNG thumbnail bytes.
    pub cover: Vec<u8>,
}
/// A media display; no playback actions are sent to applications.
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    /// Application identifier or substring filter.
    pub source: String,
    /// Display album artwork.
    pub show_cover: bool,
    /// Cover corner radius in pixels.
    pub cover_radius: f32,
    /// Independent cover radii: top-left, top-right, bottom-right, bottom-left.
    pub cover_corners: Option<[f32; 4]>,
    /// Cover to text spacing; None preserves font-relative spacing.
    pub cover_gap: Option<f32>,
    /// Opacity gradient applied only to the cover.
    pub cover_fade: Option<super::gradient::Fade>,
    /// Display timeline and progress bar.
    pub show_progress: bool,
    /// Display the application identifier.
    pub show_source: bool,
    /// Hide the widget when playback stops.
    pub hide_when_stopped: bool,
    /// Label when no session exists.
    pub empty_text: String,
    /// Display the track title.
    pub show_title: bool,
    /// Display the track artist.
    pub show_artist: bool,
    /// Display the times; `None` follows `show_progress` (themes saved
    /// before the parts were independent).
    pub show_time: Option<bool>,
    /// Which times are printed.
    pub time_format: TimeFormat,
    /// How the progress bar looks and where the times sit around it.
    pub progress: ProgressStyle,
}

impl Default for Player {
    /// Every part on, as a new player is added; the look of themes saved
    /// before the optional parts.
    fn default() -> Self {
        Self {
            source: String::new(),
            show_cover: true,
            cover_radius: 0.,
            cover_corners: None,
            cover_gap: None,
            cover_fade: None,
            show_progress: true,
            show_source: false,
            hide_when_stopped: false,
            empty_text: String::new(),
            show_title: true,
            show_artist: true,
            show_time: None,
            time_format: TimeFormat::default(),
            progress: ProgressStyle::default(),
        }
    }
}

/// The times a player prints, as `m:ss`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeFormat {
    /// `1:24 / 3:30`.
    #[default]
    ElapsedTotal,
    /// `1:24 / -2:06`.
    ElapsedRemaining,
    /// `1:24`.
    Elapsed,
    /// `-2:06`.
    Remaining,
    /// `3:30`.
    Total,
}

/// Where the times sit while the progress bar is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimePlacement {
    /// A line of the text, as before the bar had options.
    #[default]
    Text,
    /// Elapsed left of the bar, total or remaining right of it.
    Sides,
    /// On a row above the bar, at its two ends.
    Above,
}

/// How the progress bar is cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressPattern {
    /// One continuous bar.
    #[default]
    Continuous,
    /// Blocks separated by gaps.
    Segments,
    /// Dots as wide as the bar is thick.
    Dots,
}

/// The ends of the progress bar (and of its blocks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProgressEnds {
    /// Half the thickness as radius.
    #[default]
    Round,
    /// Square corners.
    Square,
}

/// Track of the progress bar before it had options.
pub const DEFAULT_PROGRESS_TRACK: Rgba = Rgba {
    r: 100,
    g: 100,
    b: 100,
    a: 100,
};
/// Thickness of the progress bar before it had options, pixels.
pub const DEFAULT_PROGRESS_THICKNESS: f32 = 4.;
/// Blocks of a segmented progress bar unless chosen.
pub const DEFAULT_PROGRESS_SEGMENTS: u16 = 12;
/// Thinnest and thickest progress bar, pixels.
pub const PROGRESS_THICKNESS: (f32, f32) = (1., 64.);
/// Fewest and most blocks of a segmented progress bar.
pub const PROGRESS_SEGMENTS: (u16, u16) = (2, 100);
/// Space between the text and the progress bar, pixels.
pub const PROGRESS_SPACING: f32 = 4.;

/// The look of a player's progress bar; the default is the bar drawn before
/// it had options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressStyle {
    /// Filled part; `None` uses the text colour.
    pub fill: Option<Rgba>,
    /// Empty track.
    pub track: Rgba,
    /// Thickness in pixels, within [`PROGRESS_THICKNESS`].
    pub thickness: f32,
    /// Round or square ends.
    pub ends: ProgressEnds,
    /// A knob at the current position.
    pub knob: bool,
    /// Continuous, blocks or dots.
    pub pattern: ProgressPattern,
    /// Number of blocks of [`ProgressPattern::Segments`], within [`PROGRESS_SEGMENTS`].
    pub segments: u16,
    /// Where the times sit while the bar is shown.
    pub times: TimePlacement,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            fill: None,
            track: DEFAULT_PROGRESS_TRACK,
            thickness: DEFAULT_PROGRESS_THICKNESS,
            ends: ProgressEnds::Round,
            knob: false,
            pattern: ProgressPattern::Continuous,
            segments: DEFAULT_PROGRESS_SEGMENTS,
            times: TimePlacement::Text,
        }
    }
}

impl ProgressStyle {
    /// Corner radius of the bar and of each block.
    pub fn radius(&self) -> f32 {
        match self.ends {
            ProgressEnds::Round => self.thickness / 2.,
            ProgressEnds::Square => 0.,
        }
    }

    /// Diameter of the knob, 0 without one.
    pub fn knob_diameter(&self) -> f32 {
        if self.knob {
            (self.thickness * 2.).max(self.thickness + 6.)
        } else {
            0.
        }
    }

    /// Height of the bar with its knob.
    pub fn bar_height(&self) -> f32 {
        self.thickness.max(self.knob_diameter())
    }

    /// Blocks along a bar `length` long, as `(start, end)` offsets: one for
    /// a continuous bar, the chosen count of segments, or as many dots as
    /// fit, each as wide as the bar is thick.
    pub fn blocks(&self, length: f32) -> Vec<(f32, f32)> {
        let t = self.thickness;
        if !(length > 0. && t > 0.) {
            return Vec::new();
        }
        let (count, gap) = match self.pattern {
            ProgressPattern::Continuous => return vec![(0., length)],
            ProgressPattern::Segments => (f32::from(self.segments.max(1)), (t * 0.75).max(2.)),
            ProgressPattern::Dots => {
                let count = ((length + t * 0.6) / (t * 1.6)).floor().max(1.);
                let gap = if count > 1. {
                    (length - count * t) / (count - 1.)
                } else {
                    0.
                };
                (count, gap)
            }
        };
        let size = (length - gap * (count - 1.)) / count;
        if size <= 0. {
            return vec![(0., length)];
        }
        (0..count as u32)
            .map(|i| {
                let a = i as f32 * (size + gap);
                (a, a + size)
            })
            .collect()
    }
}

/// `m:ss`, minutes unbounded, as the player has always printed time.
pub fn clock_time(seconds: f64) -> String {
    let s = if seconds.is_finite() {
        seconds.max(0.) as u64
    } else {
        0
    };
    format!("{}:{:02}", s / 60, s % 60)
}

/// What a player prints for one session.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerText {
    /// Lines of the text block (title, artist, app, inline times).
    pub lines: Vec<String>,
    /// Times at the ends of the progress bar: left and right.
    pub times: Option<(Option<String>, Option<String>)>,
    /// Fraction of the progress bar, when it is shown.
    pub progress: Option<f32>,
}

impl PlayerText {
    /// The text block as drawn: its lines joined.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Player {
    /// Whether the times are printed.
    pub fn shows_time(&self) -> bool {
        self.show_time.unwrap_or(self.show_progress)
    }

    /// The times of `media`: left and right of a bar (either may be absent).
    pub fn times(&self, media: &MediaSession) -> (Option<String>, Option<String>) {
        let elapsed = || Some(clock_time(media.position));
        let remaining = || Some(format!("-{}", clock_time(media.duration - media.position)));
        match self.time_format {
            TimeFormat::ElapsedTotal => (elapsed(), Some(clock_time(media.duration))),
            TimeFormat::ElapsedRemaining => (elapsed(), remaining()),
            TimeFormat::Elapsed => (elapsed(), None),
            TimeFormat::Remaining => (None, remaining()),
            TimeFormat::Total => (None, Some(clock_time(media.duration))),
        }
    }

    /// The widest the times of `media` get while it plays: the total in
    /// place of the elapsed time, so a bar between them keeps its length.
    pub fn widest_times(&self, media: &MediaSession) -> (Option<String>, Option<String>) {
        let at_start = MediaSession {
            position: 0.,
            ..media.clone()
        };
        let (left, right) = self.times(&at_start);
        let total = clock_time(media.duration);
        (left.map(|_| total.clone()), right)
    }

    /// What is printed for `media`; times need a known duration.
    pub fn text(&self, media: &MediaSession) -> PlayerText {
        let timed = media.duration > 0.;
        let bar = self.show_progress && timed;
        let times = (self.shows_time() && timed).then(|| self.times(media));
        let mut lines = Vec::new();
        if self.show_title {
            lines.push(media.title.clone());
        }
        if self.show_artist {
            lines.push(media.artist.clone());
        }
        if self.show_source {
            lines.push(media.source.clone());
        }
        let (inline, beside) = match times {
            Some(t) if bar && self.progress.times != TimePlacement::Text => (None, Some(t)),
            other => (other, None),
        };
        if let Some((left, right)) = inline {
            let joined: Vec<String> = [left, right].into_iter().flatten().collect();
            lines.push(joined.join(" / "));
        }
        PlayerText {
            lines,
            times: beside,
            progress: bar.then(|| (media.position / media.duration).clamp(0., 1.) as f32),
        }
    }
}
/// State condition used to override a card face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerSource {
    /// Periodic face change, independent of desktop activity.
    Timer,
    /// Executable is running.
    Process,
    /// Executable is absent from a successful process scan.
    ProcessClosed,
    /// Executable owns the foreground window.
    Foreground,
    /// Matching media session is playing.
    MediaPlaying,
}
/// Rules with a higher priority win; ties keep their saved order.
#[derive(Debug, Clone, PartialEq)]
pub struct CardTrigger {
    /// Interval of a timed rule, 5 to 3600 seconds.
    pub interval_seconds: Option<u32>,
    /// Timed rules may advance from the currently displayed face.
    pub next_face: bool,
    /// Application condition.
    pub source: TriggerSource,
    /// Executable name or media source filter.
    pub app: String,
    /// Target face index.
    pub face: usize,
    /// Higher values win, 0 to 100.
    pub priority: u32,
    /// Delay before restoring the previous face, 0 to 300.
    pub return_seconds: u32,
    /// Explicit return face; None restores the face preceding the trigger.
    pub return_face: Option<usize>,
}
/// Case-insensitive executable basename without its optional extension.
pub fn app_name(value: &str) -> String {
    let lower = value.trim().to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_owned()
}
/// Media sources may be packaged app IDs, hence a substring filter.
pub fn session<'a>(snapshot: &'a Snapshot, source: &str) -> Option<&'a MediaSession> {
    let name = app_name(source);
    snapshot
        .media
        .iter()
        .filter(|s| name.is_empty() || s.source.to_lowercase().contains(&name))
        .max_by(|a, b| {
            a.playing
                .cmp(&b.playing)
                .then_with(|| b.source.cmp(&a.source))
        })
}
impl CardTrigger {
    /// Evaluate a rule against one measured snapshot.
    pub fn matches(&self, snapshot: &Snapshot) -> bool {
        let app = app_name(&self.app);
        match self.source {
            TriggerSource::Timer => false,
            TriggerSource::Process => {
                snapshot.activity_available && snapshot.applications.contains(&app)
            }
            TriggerSource::ProcessClosed => {
                snapshot.activity_available && !snapshot.applications.contains(&app)
            }
            TriggerSource::Foreground => snapshot
                .foreground
                .as_ref()
                .is_some_and(|s| app_name(s) == app),
            TriggerSource::MediaPlaying => snapshot
                .media
                .iter()
                .any(|s| s.playing && (app.is_empty() || s.source.to_lowercase().contains(&app))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> MediaSession {
        MediaSession {
            source: "Spotify".into(),
            title: "Track".into(),
            artist: "Artist".into(),
            playing: true,
            position: 84.,
            duration: 210.,
            cover: Vec::new(),
        }
    }

    #[test]
    fn legacy_players_print_title_artist_and_times_with_the_bar() {
        let p = Player::default();
        let text = p.text(&session());
        assert_eq!(text.text(), "Track\nArtist\n1:24 / 3:30");
        assert_eq!(text.times, None);
        assert_eq!(text.progress, Some(0.4));
        let no_bar = Player {
            show_progress: false,
            ..Player::default()
        };
        assert_eq!(no_bar.text(&session()).text(), "Track\nArtist");
        assert_eq!(no_bar.text(&session()).progress, None);
    }

    #[test]
    fn each_part_is_independent() {
        let only_title = Player {
            show_artist: false,
            show_progress: false,
            ..Player::default()
        };
        assert_eq!(only_title.text(&session()).lines, ["Track"]);
        let artist_time = Player {
            show_title: false,
            show_progress: false,
            show_time: Some(true),
            ..Player::default()
        };
        assert_eq!(
            artist_time.text(&session()).lines,
            ["Artist", "1:24 / 3:30"]
        );
        let bar_only = Player {
            show_title: false,
            show_artist: false,
            show_time: Some(false),
            ..Player::default()
        };
        let text = bar_only.text(&session());
        assert!(text.lines.is_empty() && text.times.is_none());
        assert_eq!(text.progress, Some(0.4));
        let untimed = MediaSession {
            duration: 0.,
            ..session()
        };
        assert_eq!(
            artist_time.text(&untimed).lines,
            ["Artist"],
            "no times without a duration"
        );
    }

    #[test]
    fn time_formats_and_placements() {
        let mut p = Player {
            show_title: false,
            show_artist: false,
            ..Player::default()
        };
        let cases = [
            (TimeFormat::ElapsedTotal, "1:24 / 3:30"),
            (TimeFormat::ElapsedRemaining, "1:24 / -2:06"),
            (TimeFormat::Elapsed, "1:24"),
            (TimeFormat::Remaining, "-2:06"),
            (TimeFormat::Total, "3:30"),
        ];
        for (format, inline) in cases {
            p.time_format = format;
            assert_eq!(p.text(&session()).lines, [inline]);
        }
        p.time_format = TimeFormat::ElapsedRemaining;
        p.progress.times = TimePlacement::Sides;
        let text = p.text(&session());
        assert!(text.lines.is_empty());
        assert_eq!(
            text.times,
            Some((Some("1:24".into()), Some("-2:06".into())))
        );
        assert_eq!(
            p.widest_times(&session()),
            (Some("3:30".into()), Some("-3:30".into()))
        );
        p.show_progress = false;
        assert_eq!(p.text(&session()).times, None, "no bar: times are text");
        assert_eq!(clock_time(f64::NAN), "0:00");
        assert_eq!(clock_time(-3.), "0:00");
        assert_eq!(clock_time(4_500.), "75:00");
    }

    #[test]
    fn progress_geometry() {
        let mut s = ProgressStyle::default();
        assert_eq!(
            (s.radius(), s.knob_diameter(), s.bar_height()),
            (2., 0., 4.)
        );
        assert_eq!(s.blocks(100.), [(0., 100.)]);
        assert!(s.blocks(0.).is_empty());
        s.knob = true;
        assert_eq!((s.knob_diameter(), s.bar_height()), (10., 10.));
        s.ends = ProgressEnds::Square;
        assert_eq!(s.radius(), 0.);
        s.pattern = ProgressPattern::Segments;
        s.segments = 4;
        let blocks = s.blocks(100.);
        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[3].1, 100.);
        assert!((blocks[1].0 - blocks[0].1 - 3.).abs() < 1e-4);
        s.pattern = ProgressPattern::Dots;
        let dots = s.blocks(100.);
        assert!(dots.iter().all(|(a, b)| (b - a - 4.).abs() < 1e-3));
        assert!((dots.last().unwrap().1 - 100.).abs() < 1e-3);
        assert_eq!(dots.len(), 16);
        assert_eq!(s.blocks(3.), [(0., 3.)], "a dot never exceeds the bar");
        s.segments = 200;
        s.pattern = ProgressPattern::Segments;
        assert_eq!(s.blocks(10.), [(0., 10.)], "too many blocks: one bar");
    }
}
