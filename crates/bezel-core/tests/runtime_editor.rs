//! The theme runtime as an editor drives it (D-2026-09-30-studio-app-4):
//! every edit swapped in place with the assets and the graph histories
//! kept, the readings of the last sample for the sensor panel and the units
//! of a catalog read again. The sensors are the adapter's fake; the renderer
//! is a recording double defined here (no adapter ships a fake of it).
#![allow(clippy::expect_used)] // helpers of a failing test panic

use std::collections::BTreeMap;
use std::time::Duration;

use bezel_core::Result;
use bezel_core::app::ThemeRuntime;
use bezel_core::domain::clock::{Language, LocalTime};
use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::geometry::{Orientation, Size};
use bezel_core::domain::history::SeriesId;
use bezel_core::domain::sensor::{Category, Quantity, Reading, SensorInfo, SensorKey, Snapshot};
use bezel_core::domain::theme::{
    AssetRef, Binding, BoxF, Element, ElementId, ElementKind, GraphStyle, Theme,
};
use bezel_core::ports::{FrameRenderer, RenderContext};
use bezel_sensors::FakeSensors;

const TIME: LocalTime = LocalTime {
    year: 2026,
    month: 9,
    day: 30,
    hour: 21,
    minute: 5,
    second: 0,
    weekday: 2,
    millisecond: 0,
};

fn key(text: &str) -> SensorKey {
    SensorKey::new(text).expect("key")
}

/// What one frame was drawn from.
#[derive(Debug, Clone, PartialEq)]
struct Drawn {
    theme: String,
    assets: Vec<AssetRef>,
    history: Vec<Option<f64>>,
    usage: Reading,
    unit: Option<Quantity>,
}

/// A renderer that records what each frame was drawn from (the history and
/// reading of `cpu.usage`) and returns a blank frame of the canvas.
#[derive(Default)]
struct Recorder {
    seen: Vec<Drawn>,
}

impl FrameRenderer for Recorder {
    fn render(
        &mut self,
        theme: &Theme,
        assets: &BTreeMap<AssetRef, Vec<u8>>,
        context: RenderContext<'_>,
    ) -> Result<Frame> {
        let usage = key("cpu.usage");
        self.seen.push(Drawn {
            theme: theme.name.clone(),
            assets: assets.keys().cloned().collect(),
            history: context.histories.points(&series(theme), window(theme)),
            usage: context.snapshot.get(&usage),
            unit: context.quantities.get(&usage),
        });
        Ok(Frame::filled(theme.canvas, Rgba::default()))
    }
}

/// How many points the theme's graph shows (its window of the history).
fn window(theme: &Theme) -> usize {
    theme
        .elements
        .iter()
        .find_map(|e| match e.kind {
            ElementKind::Graph { history, .. } => Some(usize::from(history)),
            _ => None,
        })
        .unwrap_or(0)
}

/// The history series the theme's graph draws.
fn series(theme: &Theme) -> SeriesId {
    theme.graph_series().first().cloned().expect("a graph")
}

/// A theme graphing `cpu.usage` over `history` points, one every
/// `interval_seconds` (`None`: one per refresh), refreshed every
/// `refresh_seconds`.
fn graphing_every(history: u16, interval_seconds: Option<u32>, refresh_seconds: f32) -> Theme {
    let mut theme = graphing("timed", history);
    theme.refresh_seconds = refresh_seconds;
    if let ElementKind::Graph {
        interval_seconds: interval,
        ..
    } = &mut theme.elements[0].kind
    {
        *interval = interval_seconds;
    }
    theme
}

/// A theme graphing `cpu.usage` over `history` samples.
fn graphing(name: &str, history: u16) -> Theme {
    let mut theme = Theme::blank(name, Size::new(480, 1920), Orientation::Portrait);
    theme.elements.push(Element {
        color_by_value: None,
        visualization: None,
        value_transition: None,
        sensor_motion: None,
        card: None,
        rotation: 0.0,
        is_group: false,
        group_parent: None,
        card_member: None,
        id: ElementId(1),
        name: "usage".into(),
        frame: BoxF {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 80.0,
        },
        opacity: 1.0,
        visible: true,
        locked: false,
        kind: ElementKind::Graph {
            binding: Binding {
                key: key("cpu.usage"),
                min: 0.0,
                max: 100.0,
            },
            history,
            interval_seconds: None,
            style: GraphStyle::Line,
            color: Rgba::WHITE,
            fill: None,
            line_width: 1.0,
            autoscale: false,
        },
    });
    theme
}

fn info(text: &str, quantity: Quantity) -> SensorInfo {
    SensorInfo {
        key: key(text),
        category: Category::Cpu,
        label: text.into(),
        quantity,
        source: "test".into(),
    }
}

/// Sensors whose `cpu.usage` reads 10, 20, 30… (then stays at the last).
fn usage(values: &[f64]) -> FakeSensors {
    let script = values
        .iter()
        .map(|v| {
            let mut snapshot = Snapshot::default();
            snapshot.insert(key("cpu.usage"), Reading::Value(*v));
            snapshot
        })
        .collect();
    FakeSensors::new(vec![info("cpu.usage", Quantity::Percent)], script)
}

#[test]
fn edits_keep_the_assets_and_the_graph_history() {
    let logo = AssetRef("assets/logo.png".into());
    let assets = BTreeMap::from([(logo.clone(), vec![1, 2, 3])]);
    let mut rt = ThemeRuntime::new(graphing("first", 3), assets, Language::English);
    let mut sensors = usage(&[10.0, 20.0, 30.0]);
    let mut r = Recorder::default();
    rt.sample(&mut sensors).expect("sample");
    rt.sample(&mut sensors).expect("sample");

    rt.replace_theme(graphing("edited", 3));
    let clip = AssetRef("assets/clip.png".into());
    rt.add_asset(clip.clone(), vec![4]);
    rt.render(&mut r, TIME, Default::default()).expect("frame");
    assert_eq!(rt.theme().name, "edited");
    assert_eq!(rt.assets().len(), 2);
    assert_eq!(r.seen[0].assets, [clip.clone(), logo.clone()]);
    assert_eq!(r.seen[0].history, [Some(10.0), Some(20.0)], "history kept");

    // A shorter graph keeps the newest samples; new bytes replace old ones.
    rt.sample(&mut sensors).expect("sample");
    rt.replace_theme(graphing("shorter", 2));
    rt.add_asset(clip.clone(), vec![5, 6]);
    assert_eq!(rt.assets()[&clip], [5, 6]);
    rt.render(&mut r, TIME, Default::default()).expect("frame");
    assert_eq!(r.seen[1].history, [Some(20.0), Some(30.0)]);

    // Another document brings its own assets and still keeps the history,
    // all of it: the shorter window in between cut nothing.
    rt.replace(graphing("other", 3), BTreeMap::new());
    rt.render(&mut r, TIME, Default::default()).expect("frame");
    assert!(r.seen[2].assets.is_empty());
    assert_eq!(r.seen[2].history, [Some(10.0), Some(20.0), Some(30.0)]);
}

#[test]
fn a_window_cut_short_then_widened_again_keeps_its_points() {
    let mut rt = ThemeRuntime::new(graphing("t", 4), BTreeMap::new(), Language::English);
    let mut sensors = usage(&[10.0, 20.0, 30.0, 40.0]);
    for _ in 0..4 {
        rt.sample(&mut sensors).expect("sample");
    }
    // Every preview swaps the edited theme in: a short window in between
    // must not cut what the graph recorded.
    rt.replace_theme(graphing("short", 2));
    rt.replace_theme(graphing("wide", 4));
    let mut r = Recorder::default();
    rt.render(&mut r, TIME, Default::default()).expect("frame");
    assert_eq!(
        r.seen[0].history,
        [Some(10.0), Some(20.0), Some(30.0), Some(40.0)]
    );
}

/// Runs `rt`'s refresh loop from 0 to `seconds` in steps of a second,
/// sampling when due; the seconds it sampled at.
fn run(rt: &mut ThemeRuntime, sensors: &mut FakeSensors, seconds: u64) -> Vec<u64> {
    (0..=seconds)
        .filter(|s| {
            rt.sample_on_time(sensors, Duration::from_secs(*s))
                .expect("sample")
        })
        .collect()
}

/// What the theme's graph draws now.
fn drawn(rt: &mut ThemeRuntime) -> Vec<Option<f64>> {
    let mut r = Recorder::default();
    rt.render(&mut r, TIME, Default::default()).expect("frame");
    r.seen.remove(0).history
}

#[test]
fn a_graph_with_its_own_interval_takes_a_point_at_its_pace() {
    // Refreshed every second; a point every 3 s.
    let mut rt = ThemeRuntime::new(
        graphing_every(3, Some(3), 1.0),
        BTreeMap::new(),
        Language::English,
    );
    let values: Vec<f64> = (0..=9).map(f64::from).collect();
    let mut sensors = usage(&values);
    assert_eq!(run(&mut rt, &mut sensors, 9).len(), 10, "every refresh");
    assert_eq!(drawn(&mut rt), [Some(3.0), Some(6.0), Some(9.0)]);
}

#[test]
fn an_interval_shorter_than_the_refresh_samples_sooner() {
    // Refreshed every 5 s; a point every second.
    let mut rt = ThemeRuntime::new(
        graphing_every(6, Some(1), 5.0),
        BTreeMap::new(),
        Language::English,
    );
    let values: Vec<f64> = (0..=5).map(f64::from).collect();
    let mut sensors = usage(&values);
    assert_eq!(run(&mut rt, &mut sensors, 5), [0, 1, 2, 3, 4, 5]);
    assert_eq!(
        rt.next_sample(),
        Duration::from_secs(6),
        "the graph's next point"
    );
    assert_eq!(drawn(&mut rt).len(), 6, "one point a second");
}

#[test]
fn faster_graph_points_do_not_refresh_other_values_at_two_or_five_seconds() {
    for seconds in [2, 5] {
        let mut rt = ThemeRuntime::new(
            graphing_every(10, Some(1), seconds as f32),
            BTreeMap::new(),
            Language::English,
        );
        let values: Vec<f64> = (0..=6).map(f64::from).collect();
        let mut sensors = usage(&values);
        let mut renderer = Recorder::default();
        for elapsed in 0..=6 {
            let now = Duration::from_secs(elapsed);
            assert!(rt.sample_on_time(&mut sensors, now).expect("graph sample"));
            rt.render(&mut renderer, TIME, now).expect("frame");
            assert_eq!(
                renderer.seen.last().expect("frame").usage,
                Reading::Value((elapsed / seconds * seconds) as f64),
                "only the graph updates between theme refreshes ({seconds} s at {elapsed} s)"
            );
            assert_eq!(
                rt.snapshot().get(&key("cpu.usage")),
                Reading::Value(elapsed as f64),
                "the sensor panel still gets the latest reading"
            );
        }
        assert_eq!(
            drawn(&mut rt),
            values.into_iter().map(Some).collect::<Vec<_>>()
        );
    }
}

#[test]
fn slowing_refresh_rejoins_the_graph_clock_instead_of_sampling_twice() {
    for offset in [0, 137] {
        let mut rt = ThemeRuntime::new(
            graphing_every(10, Some(1), 0.5),
            BTreeMap::new(),
            Language::English,
        );
        let mut sensors = usage(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
        let mut sampled = Vec::new();
        for elapsed in (0..=4_000).step_by(10) {
            if elapsed == 1_250 {
                let mut edited = rt.theme().clone();
                edited.refresh_seconds = 1.0;
                rt.replace_theme(edited);
            }
            if rt
                .sample_on_time(&mut sensors, Duration::from_millis(offset + elapsed))
                .expect("sample")
            {
                sampled.push(elapsed);
            }
        }
        assert_eq!(sampled, [0, 500, 1_000, 2_000, 3_000, 4_000]);
        assert_eq!(
            drawn(&mut rt),
            [Some(10.0), Some(30.0), Some(40.0), Some(50.0), Some(60.0)]
        );
    }
}

#[test]
fn changing_refresh_both_ways_keeps_the_graph_history_and_original_phase() {
    let mut rt = ThemeRuntime::new(
        graphing_every(10, Some(1), 1.0),
        BTreeMap::new(),
        Language::English,
    );
    let mut sensors = usage(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
    let mut sampled = Vec::new();
    for elapsed in (0..=4_000).step_by(10) {
        if matches!(elapsed, 1_250 | 1_750 | 2_250) {
            let mut edited = rt.theme().clone();
            if elapsed == 1_750 {
                edited.name = "a name edit keeps the schedule".into();
            } else {
                edited.refresh_seconds = if elapsed == 1_250 { 0.5 } else { 1.0 };
            }
            rt.replace_theme(edited);
        }
        if rt
            .sample_on_time(&mut sensors, Duration::from_millis(elapsed))
            .expect("sample")
        {
            sampled.push(elapsed);
        }
    }
    assert_eq!(sampled, [0, 1_000, 1_500, 2_000, 3_000, 4_000]);
    assert_eq!(
        drawn(&mut rt),
        [Some(10.0), Some(20.0), Some(40.0), Some(50.0), Some(60.0)]
    );
}

#[test]
fn rapid_refresh_edits_do_not_invent_an_overdue_sample() {
    let mut rt = ThemeRuntime::new(
        graphing_every(10, Some(1), 0.5),
        BTreeMap::new(),
        Language::English,
    );
    let mut sensors = usage(&[10.0]);
    let mut sampled = Vec::new();
    for elapsed in (0..=2_000).step_by(10) {
        if matches!(elapsed, 750 | 800) {
            let mut edited = rt.theme().clone();
            edited.refresh_seconds = if elapsed == 750 { 1.0 } else { 0.5 };
            rt.replace_theme(edited);
        }
        if rt
            .sample_on_time(&mut sensors, Duration::from_millis(elapsed))
            .expect("sample")
        {
            sampled.push(elapsed);
        }
    }
    assert_eq!(sampled, [0, 500, 1_000, 1_500, 2_000]);
}

#[test]
fn a_graph_without_an_interval_takes_a_point_per_refresh_as_before() {
    let mut rt = ThemeRuntime::new(
        graphing_every(10, None, 2.0),
        BTreeMap::new(),
        Language::English,
    );
    let values: Vec<f64> = (0..=6).map(f64::from).collect();
    let mut sensors = usage(&values);
    assert_eq!(run(&mut rt, &mut sensors, 6), [0, 2, 4, 6]);
    assert_eq!(drawn(&mut rt), [Some(0.0), Some(1.0), Some(2.0), Some(3.0)]);
    assert_eq!(rt.next_sample(), Duration::from_secs(8));
}

#[test]
fn a_new_interval_starts_its_graph_again() {
    let mut rt = ThemeRuntime::new(
        graphing_every(4, Some(1), 1.0),
        BTreeMap::new(),
        Language::English,
    );
    let mut sensors = usage(&[1.0, 2.0, 3.0]);
    run(&mut rt, &mut sensors, 2);
    assert_eq!(drawn(&mut rt).len(), 3);
    rt.replace_theme(graphing_every(8, Some(1), 1.0));
    assert_eq!(
        drawn(&mut rt).len(),
        3,
        "a longer window of the same cadence"
    );
    rt.replace_theme(graphing_every(4, Some(2), 1.0));
    assert!(drawn(&mut rt).is_empty(), "no point of the old cadence");
    rt.replace_theme(graphing_every(4, Some(1), 1.0));
    assert!(
        drawn(&mut rt).is_empty(),
        "points not recorded meanwhile never come back next to new ones"
    );
}

#[test]
fn the_last_sample_feeds_the_sensor_panel() {
    let mut rt = ThemeRuntime::new(graphing("t", 4), BTreeMap::new(), Language::English);
    assert!(rt.snapshot().is_empty(), "nothing sampled yet");
    assert!(rt.quantities().is_empty());
    let mut sensors = usage(&[10.0, 42.0]);
    rt.sample(&mut sensors).expect("sample");
    rt.sample(&mut sensors).expect("sample");
    assert_eq!(rt.snapshot().get(&key("cpu.usage")), Reading::Value(42.0));
    assert_eq!(
        rt.quantities().get(&key("cpu.usage")),
        Some(Quantity::Percent),
        "the first sample reads the catalog"
    );

    let mut r = Recorder::default();
    rt.render_with(&mut r, TIME, Default::default())
        .expect("preview");
    assert_eq!(r.seen[0].usage, Reading::Value(42.0));
    assert_eq!(r.seen[0].history, [Some(10.0), Some(42.0)]);
}

#[test]
fn a_catalog_read_again_gives_the_units() {
    let mut rt = ThemeRuntime::new(graphing("t", 4), BTreeMap::new(), Language::English);
    rt.use_catalog(&[info("cpu.usage", Quantity::Celsius)]);
    // The runtime keeps the catalog it was given: a sample does not read it.
    rt.sample(&mut usage(&[1.0])).expect("sample");
    let mut r = Recorder::default();
    rt.render_with(&mut r, TIME, Default::default())
        .expect("preview");
    assert_eq!(r.seen[0].unit, Some(Quantity::Celsius));
    rt.use_catalog(&[]);
    assert!(rt.quantities().is_empty(), "the sensor is gone");
}

#[test]
fn recent_frames_are_shared_without_extending_their_lifetime() {
    use std::time::Duration;
    let mut runtime = ThemeRuntime::new(graphing("reuse", 8), BTreeMap::new(), Language::English);
    runtime.reuse_recent_frames(true);
    let mut renderer = Recorder::default();
    runtime
        .preview(&mut renderer, TIME, Duration::ZERO)
        .unwrap();
    runtime
        .render_at(
            &mut renderer,
            TIME,
            Duration::from_millis(10),
            Duration::ZERO,
        )
        .unwrap();
    assert!(runtime.render_reused());
    assert_eq!(renderer.seen.len(), 1);
    runtime
        .preview(&mut renderer, TIME, Duration::from_millis(30))
        .unwrap();
    assert!(runtime.render_reused());
    runtime
        .preview(&mut renderer, TIME, Duration::from_millis(34))
        .unwrap();
    assert!(!runtime.render_reused());
    assert_eq!(renderer.seen.len(), 2);
    // Backward clocks and different wall-clock seconds also force fresh output.
    runtime
        .preview(&mut renderer, TIME, Duration::from_millis(33))
        .unwrap();
    assert!(!runtime.render_reused());
    let mut later = TIME;
    later.second += 1;
    runtime
        .preview(&mut renderer, later, Duration::from_millis(34))
        .unwrap();
    assert!(!runtime.render_reused());
}

#[test]
fn edits_assets_samples_units_and_motion_invalidate_recent_frames() {
    use std::time::Duration;
    let mut runtime = ThemeRuntime::new(graphing("reuse", 8), BTreeMap::new(), Language::English);
    runtime.reuse_recent_frames(true);
    let mut renderer = Recorder::default();
    let mut sensors = usage(&[10., 20.]);
    for change in 0..5 {
        runtime
            .preview(&mut renderer, TIME, Duration::ZERO)
            .unwrap();
        match change {
            0 => runtime.replace_theme(graphing("edited", 8)),
            1 => runtime.add_asset(AssetRef("icon.png".into()), vec![1]),
            2 => runtime.sample(&mut sensors).unwrap(),
            3 => runtime.use_catalog(&[info("cpu.usage", Quantity::Percent)]),
            _ => {
                runtime.reuse_recent_frames(false);
                runtime.reuse_recent_frames(true);
            }
        }
        runtime
            .preview(&mut renderer, TIME, Duration::from_millis(1))
            .unwrap();
        assert!(!runtime.render_reused(), "change {change}");
    }
    let seen = renderer.seen.len();
    runtime.reuse_recent_frames(false);
    runtime
        .preview(&mut renderer, TIME, Duration::from_millis(2))
        .unwrap();
    runtime
        .preview(&mut renderer, TIME, Duration::from_millis(3))
        .unwrap();
    assert_eq!(renderer.seen.len(), seen + 2);
}
