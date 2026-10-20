//! Smooth value transitions in a live loop
//! (D-2026-10-15-studio-redesign-fidelity-18): a bar eases from its value to
//! each new sample, the screen gets frames only while it moves (at most
//! `MAX_TRANSITION_FPS` a second), a slower screen gets fewer and a screen
//! too slow for two frames per transition gets the new value at once. The
//! sensors and the screen are the adapters' fakes; the renderer is a
//! recording double. The clock is the test's: no real waits.
#![allow(clippy::expect_used)] // helpers of a failing test panic

use std::collections::BTreeMap;
use std::time::Duration;

use bezel_core::Result;
use bezel_core::app::{ThemeRuntime, open_screen};
use bezel_core::domain::clock::{Language, LocalTime};
use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::geometry::{Orientation, Size};
use bezel_core::domain::sensor::{Reading, SensorKey, Snapshot};
use bezel_core::domain::theme::{
    AssetRef, Binding, BoxF, Direction, Element, ElementId, ElementKind, GraphStyle, Paint, Theme,
};
use bezel_core::domain::transition::{MAX_TRANSITION_FPS, ValueTransition};
use bezel_core::ports::{FrameRenderer, RenderContext};
use bezel_devices::{FakeBus, FakeConnector};
use bezel_sensors::FakeSensors;

const TIME: LocalTime = LocalTime {
    year: 2026,
    month: 10,
    day: 15,
    hour: 21,
    minute: 5,
    second: 0,
    weekday: 4,
    millisecond: 0,
};

const BAR: ElementId = ElementId(1);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn key() -> SensorKey {
    SensorKey::new("cpu.usage").expect("key")
}

/// A renderer that records the value the bar shows in every frame (the
/// easing value, else the reading).
#[derive(Default)]
struct Recorder {
    shown: Vec<f64>,
}

impl FrameRenderer for Recorder {
    fn render(
        &mut self,
        theme: &Theme,
        _: &BTreeMap<AssetRef, Vec<u8>>,
        context: RenderContext<'_>,
    ) -> Result<Frame> {
        let value = context
            .shown
            .get(BAR)
            .or_else(|| context.snapshot.get(&key()).value())
            .unwrap_or(f64::NAN);
        self.shown.push(value);
        // Each value its own frame, so the screen's diff sends it.
        let level = (value.clamp(0.0, 100.0) * 2.55) as u8;
        Ok(Frame::filled(
            theme.canvas,
            Rgba::opaque(level, level, level),
        ))
    }
}

fn bar(transition: Option<ValueTransition>) -> Element {
    Element {
        sensor_motion: None,
        rotation: 0.0,
        is_group: false,
        group_parent: None,
        card: None,
        card_member: None,
        id: BAR,
        name: "CPU".into(),
        frame: BoxF::new(10.0, 10.0, 200.0, 20.0),
        opacity: 1.0,
        visible: true,
        locked: false,
        color_by_value: None,
        visualization: None,
        value_transition: transition,
        kind: ElementKind::Bar {
            binding: Binding {
                key: key(),
                min: 0.0,
                max: 100.0,
            },
            direction: Direction::LeftToRight,
            fill: Paint::solid(Rgba::WHITE),
            track: None,
            radius: 0.0,
            segments: None,
        },
    }
}

/// An 8.8" theme refreshed every second with the bar.
fn runtime(transition: Option<ValueTransition>) -> ThemeRuntime {
    let mut theme = Theme::blank("eased", Size::new(480, 1920), Orientation::Landscape);
    theme.elements = vec![bar(transition)];
    ThemeRuntime::new(theme, BTreeMap::new(), Language::English)
}

/// Sensors reading `values` in turn (the last one repeats).
fn sensors(values: &[f64]) -> FakeSensors {
    let script = values
        .iter()
        .map(|v| {
            let mut s = Snapshot::default();
            s.insert(key(), Reading::Value(*v));
            s
        })
        .collect();
    FakeSensors::new(Vec::new(), script)
}

/// Runs a live loop for `until` on the test's clock, each frame taking
/// `send` to reach the screen (told to the runtime as its pace): when each
/// frame was drawn.
fn live_loop(
    rt: &mut ThemeRuntime,
    renderer: &mut Recorder,
    sensors: &mut FakeSensors,
    send: Duration,
    until: Duration,
) -> Vec<Duration> {
    let connector = FakeConnector::default();
    let mut screen = open_screen(&FakeBus::turing_88(), &connector, None).expect("screen");
    screen
        .set_orientation(Orientation::Landscape)
        .expect("turns");
    let mut now = Duration::ZERO;
    let mut drawn = Vec::new();
    while now < until {
        let frame = rt.live_frame(sensors, renderer, TIME, now).expect("frame");
        drawn.push(now);
        screen.present(&frame).expect("shown");
        rt.screen_pace(send);
        now += send;
        now = now.max(rt.next_due());
    }
    assert_eq!(connector.log().frames.len(), drawn.len(), "one frame each");
    drawn
}

/// Frames drawn in `from..to`, per second.
fn per_second(drawn: &[Duration], from: Duration, to: Duration) -> f64 {
    let n = drawn.iter().filter(|t| **t >= from && **t < to).count();
    n as f64 / (to - from).as_secs_f64()
}

#[test]
fn steady_readings_send_one_frame_per_refresh() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut r = Recorder::default();
    let drawn = live_loop(
        &mut rt,
        &mut r,
        &mut sensors(&[42.0]),
        Duration::ZERO,
        ms(3_000),
    );
    assert_eq!(drawn, [ms(0), ms(1_000), ms(2_000)], "no transition frames");
    assert_eq!(r.shown, [42.0, 42.0, 42.0]);
    assert!(!rt.transitioning(ms(2_000)));
}

#[test]
fn a_new_reading_eases_at_most_twenty_frames_a_second() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut r = Recorder::default();
    let drawn = live_loop(
        &mut rt,
        &mut r,
        &mut sensors(&[20.0, 80.0]),
        ms(5),
        ms(2_500),
    );
    let steps: Vec<Duration> = (0..=8).map(|n| ms(1_000 + n * 50)).collect();
    let mut expected = vec![ms(0)];
    expected.extend(steps);
    expected.push(ms(2_000));
    assert_eq!(drawn, expected, "a frame every 50 ms while it moves");
    let fps = per_second(&drawn, ms(1_000), ms(1_400));
    assert_eq!(fps, f64::from(MAX_TRANSITION_FPS), "measured {fps} fps");
    println!("transition at a 5 ms screen: {fps} frames/s");
    assert_eq!(r.shown.first(), Some(&20.0));
    let moving = &r.shown[1..10];
    assert!(moving.windows(2).all(|w| w[1] > w[0]), "{moving:?}");
    assert_eq!(moving.first(), Some(&20.0), "from the value shown");
    assert_eq!(moving.last(), Some(&80.0), "lands on the reading");
    assert!(
        (r.shown[5] - 72.5).abs() < 1e-9,
        "eased half way: {}",
        r.shown[5]
    );
    assert_eq!(r.shown.last(), Some(&80.0));
}

#[test]
fn elements_without_a_transition_jump_as_before() {
    for transition in [None, Some(ValueTransition::smooth(0))] {
        let mut rt = runtime(transition);
        let mut r = Recorder::default();
        let drawn = live_loop(
            &mut rt,
            &mut r,
            &mut sensors(&[20.0, 80.0]),
            ms(5),
            ms(2_500),
        );
        assert_eq!(drawn, [ms(0), ms(1_000), ms(2_000)]);
        assert_eq!(r.shown, [20.0, 80.0, 80.0]);
    }
}

#[test]
fn a_slower_screen_gets_fewer_frames_then_the_value_at_once() {
    // 150 ms a frame: still two frames within 400 ms, at its own pace.
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut r = Recorder::default();
    let drawn = live_loop(
        &mut rt,
        &mut r,
        &mut sensors(&[20.0, 80.0]),
        ms(150),
        ms(1_900),
    );
    // The landing frame is due at 1400 ms, drawn once the screen is free.
    assert_eq!(drawn, [ms(0), ms(1_000), ms(1_150), ms(1_300), ms(1_450)]);
    let fps = per_second(&drawn, ms(1_000), ms(1_400));
    println!("transition at a 150 ms screen: {fps} frames/s");
    assert!(fps < f64::from(MAX_TRANSITION_FPS));
    assert_eq!(r.shown.last(), Some(&80.0));

    // 250 ms a frame: fewer than two frames fit, the bar snaps.
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut r = Recorder::default();
    let drawn = live_loop(
        &mut rt,
        &mut r,
        &mut sensors(&[20.0, 80.0]),
        ms(250),
        ms(2_100),
    );
    assert_eq!(drawn, [ms(0), ms(1_000), ms(2_000)], "no transition frames");
    assert_eq!(r.shown, [20.0, 80.0, 80.0], "the new value at once");
    assert!(!rt.transitioning(ms(1_100)), "snapped");
}

#[test]
fn previews_ease_unless_motion_is_reduced() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut s = sensors(&[20.0, 80.0]);
    let mut r = Recorder::default();
    rt.sample_on_time(&mut s, ms(0)).expect("sample");
    rt.sample_on_time(&mut s, ms(1_000)).expect("sample");
    let (_, next) = rt.preview(&mut r, TIME, ms(1_200)).expect("preview");
    assert_eq!(next, Some(ms(1_250)), "the next step");
    assert!((r.shown[0] - 72.5).abs() < 1e-9);
    assert!(rt.transitioning(ms(1_200)));

    rt.still_previews(true);
    let (_, next) = rt.preview(&mut r, TIME, ms(1_210)).expect("preview");
    assert_eq!(next, None, "nothing to animate");
    assert_eq!(r.shown[1], 80.0, "the reading at once");
    assert!(rt.transitioning(ms(1_210)), "the screen still eases");

    rt.still_previews(false);
    rt.render_with(&mut r, TIME, Default::default())
        .expect("frame");
    assert!(r.shown[2] < 80.0, "at the last clock given: {}", r.shown[2]);
    let (_, next) = rt.preview(&mut r, TIME, ms(1_400)).expect("preview");
    assert_eq!(next, Some(ms(2_000)), "landed: wakes at the next sample");
    assert_eq!(r.shown[3], 80.0);
    // The loop is late with that sample: a step later, not at once.
    let (_, next) = rt.preview(&mut r, TIME, ms(2_010)).expect("preview");
    assert_eq!(next, Some(ms(2_060)));
    rt.still_previews(true);
    let (_, next) = rt.preview(&mut r, TIME, ms(2_020)).expect("preview");
    assert_eq!(next, None, "reduced motion: no frames for samples either");
}

#[test]
fn a_preview_before_any_timed_sample_does_not_poll() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut r = Recorder::default();
    let (_, next) = rt.preview(&mut r, TIME, ms(10)).expect("preview");
    assert_eq!(next, None);
    let mut plain = runtime(None);
    let mut s = sensors(&[20.0]);
    plain.sample_on_time(&mut s, ms(0)).expect("sample");
    let (_, next) = plain.preview(&mut r, TIME, ms(10)).expect("preview");
    assert_eq!(next, None, "nothing eases: no frame for the sample");
}

#[test]
fn an_edit_and_a_one_off_frame_never_ease() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut s = sensors(&[20.0, 80.0, 30.0]);
    let mut r = Recorder::default();
    rt.sample_on_time(&mut s, ms(0)).expect("sample");
    rt.sample_on_time(&mut s, ms(1_000)).expect("sample");
    assert!(rt.transitioning(ms(1_100)));
    // A narrower range: the bar shows its new value at once.
    let mut edited = rt.theme().clone();
    if let ElementKind::Bar { binding, .. } = &mut edited.elements[0].kind {
        binding.max = 50.0;
    }
    rt.replace_theme(edited);
    assert!(!rt.transitioning(ms(1_100)));
    // A sample with no clock (a one-off frame) shows its reading.
    rt.frame(&mut s, &mut r, TIME).expect("frame");
    assert_eq!(r.shown, [30.0]);
    assert!(!rt.transitioning(ms(1_100)));
}

#[test]
fn a_frame_reused_by_the_preview_holds_the_values_it_drew() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    rt.reuse_recent_frames(true);
    let mut s = sensors(&[20.0, 80.0]);
    let mut r = Recorder::default();
    rt.sample_on_time(&mut s, ms(0)).expect("sample");
    rt.sample_on_time(&mut s, ms(1_000)).expect("sample");
    rt.preview(&mut r, TIME, ms(1_100)).expect("preview");
    rt.preview(&mut r, TIME, ms(1_110)).expect("preview");
    assert_eq!(r.shown.len(), 2, "another value, drawn again");
    assert!(!rt.render_reused());
    rt.preview(&mut r, TIME, ms(1_500)).expect("preview");
    rt.preview(&mut r, TIME, ms(1_510)).expect("preview");
    assert_eq!(r.shown.len(), 3, "settled: the same frame");
    assert!(rt.render_reused());
}

#[test]
fn a_forgotten_screen_forgets_its_pace() {
    let mut rt = runtime(Some(ValueTransition::smooth(400)));
    let mut s = sensors(&[20.0, 80.0]);
    rt.sample_on_time(&mut s, ms(0)).expect("sample");
    rt.screen_pace(ms(300));
    rt.sample_on_time(&mut s, ms(1_000)).expect("sample");
    assert!(!rt.transitioning(ms(1_100)), "too slow: snaps");
    rt.forget_screen();
    assert!(rt.transitioning(ms(1_100)), "a new screen eases");
}

#[test]
fn changing_half_second_refresh_back_to_one_keeps_sampling_once_per_second() {
    for transition in [None, Some(ValueTransition::smooth(400))] {
        let mut rt = runtime(transition);
        let mut theme = rt.theme().clone();
        theme.refresh_seconds = 0.5;
        rt.replace_theme(theme);
        let mut readings = sensors(&[10.0, 30.0, 50.0, 70.0, 90.0]);
        let mut renderer = Recorder::default();
        let mut sampled = Vec::new();
        let mut drawn = Vec::new();
        for t in (0..=3_000).step_by(10) {
            let now = ms(t);
            if t == 750 {
                let mut theme = rt.theme().clone();
                theme.refresh_seconds = 1.0;
                rt.replace_theme(theme);
            }
            if now < rt.next_due() {
                continue;
            }
            let before = readings.samples_taken();
            rt.live_frame(&mut readings, &mut renderer, TIME, now)
                .expect("live frame");
            drawn.push(now);
            if readings.samples_taken() != before {
                sampled.push(now);
            }
        }
        assert_eq!(sampled, [ms(0), ms(500), ms(1_000), ms(2_000), ms(3_000)]);
        assert_eq!(rt.refresh(), ms(1_000));
        if transition.is_some() {
            assert!(
                drawn.len() > sampled.len(),
                "animation frames do not take extra samples"
            );
        } else {
            assert_eq!(drawn, sampled);
        }
        println!(
            "refresh 0.5 -> 1 s, smooth={}: samples={sampled:?}, frames={}",
            transition.is_some(),
            drawn.len()
        );
    }
}

#[test]
fn one_second_graphs_do_not_restart_two_or_five_second_value_transitions() {
    for seconds in [2, 5] {
        let mut rt = runtime(Some(ValueTransition::smooth(400)));
        let mut theme = rt.theme().clone();
        theme.refresh_seconds = seconds as f32;
        let mut graph = bar(None);
        graph.id = ElementId(2);
        graph.kind = ElementKind::Graph {
            binding: Binding {
                key: key(),
                min: 0.0,
                max: 100.0,
            },
            history: 10,
            interval_seconds: Some(1),
            style: GraphStyle::Line,
            color: Rgba::WHITE,
            fill: None,
            line_width: 1.0,
            autoscale: false,
        };
        theme.elements.push(graph);
        rt.replace_theme(theme);
        let mut readings = sensors(&[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0]);
        let mut renderer = Recorder::default();
        for elapsed in (0..=seconds * 1_000 + 1_000).step_by(50) {
            rt.live_frame(&mut readings, &mut renderer, TIME, ms(elapsed))
                .expect("frame");
            if elapsed == 1_000 {
                assert_eq!(
                    renderer.shown.last(),
                    Some(&10.0),
                    "graph-only sample keeps the value"
                );
                assert!(!rt.transitioning(ms(elapsed)));
            }
            if elapsed == seconds * 1_000 + 50 {
                assert!(
                    rt.transitioning(ms(elapsed)),
                    "theme refresh starts the transition"
                );
            }
            if elapsed == seconds * 1_000 + 1_000 {
                assert_eq!(renderer.shown.last(), Some(&(10.0 + seconds as f64 * 10.0)));
                assert!(
                    !rt.transitioning(ms(elapsed)),
                    "the graph does not restart it"
                );
            }
        }
    }
}
