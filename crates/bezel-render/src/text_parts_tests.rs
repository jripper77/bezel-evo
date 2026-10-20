//! Parts with their own style, blinking separators, lines wider than their
//! box and rolling digits (D-2026-10-20-studio-ux-v2-05, -06).
use std::time::Duration;

use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::sensor::{DisplayFormat, Snapshot};
use bezel_core::domain::theme::{
    Background, Baseline, BoxF, ElementId, ElementKind, FontSpec, PartStyle, TextContent,
    TextOverflow, TextPart, TextStyle, Theme,
};
use bezel_core::domain::transition::{TransitionMode, Transitions, ValueTransition, View};
use bezel_core::ports::FrameRenderer;

use crate::testkit::{self, Scene};

const RED: Rgba = Rgba::opaque(255, 0, 0);
const KEY: &str = "cpu.usage";

fn style(size: f32) -> TextStyle {
    TextStyle {
        size,
        font: FontSpec {
            family: testkit::FAMILY.into(),
            ..FontSpec::default()
        },
        ..TextStyle::default()
    }
}

fn text(frame: BoxF, content: TextContent, style: TextStyle) -> Theme {
    let element = testkit::element(frame, ElementKind::Text { content, style });
    let (w, h) = (frame.width as u32 + 40, frame.height as u32 + 20);
    testkit::theme(w, h, Background::Color(Rgba::BLACK), vec![element])
}

fn sensor() -> TextContent {
    TextContent::Sensor {
        key: testkit::key(KEY),
        format: DisplayFormat {
            decimals: Some(0),
            ..DisplayFormat::default()
        },
        prefix: String::new(),
        suffix: "%".into(),
    }
}

fn clock(pattern: &str) -> TextContent {
    TextContent::Clock {
        pattern: pattern.into(),
        language: None,
        casing: Default::default(),
    }
}

fn reddish(c: Rgba) -> bool {
    c.r > 150 && c.g < 80 && c.b < 80
}

fn whitish(c: Rgba) -> bool {
    c.r > 150 && c.g > 150 && c.b > 150
}

fn red_rows(frame: &Frame) -> Option<(u32, u32)> {
    let size = frame.size();
    let rows: Vec<u32> = (0..size.height)
        .filter(|&y| (0..size.width).any(|x| reddish(testkit::px(frame, x, y))))
        .collect();
    Some((*rows.first()?, *rows.last()?))
}

#[test]
fn a_part_takes_its_colour_size_and_raised_baseline() {
    let mut renderer = testkit::renderer();
    let scene = Scene::empty().with(KEY, 36.0);
    let frame = BoxF::new(10.0, 10.0, 300.0, 80.0);
    let plain = testkit::render(&mut renderer, &text(frame, sensor(), style(64.0)), &scene);
    assert!(red_rows(&plain).is_none());

    let mut styled = style(64.0);
    let suffix = PartStyle {
        color: Some(RED),
        size_percent: Some(50.0),
        ..PartStyle::default()
    };
    styled.parts.styles.insert(TextPart::Suffix, suffix.clone());
    let low = testkit::render(
        &mut renderer,
        &text(frame, sensor(), styled.clone()),
        &scene,
    );
    let (low_top, low_bottom) = red_rows(&low).expect("a red suffix");
    let number = testkit::ink(&plain, Rgba::BLACK, 40).expect("the number");
    assert!(
        low_bottom - low_top < (number.3 - number.1) * 2 / 3,
        "half the size"
    );
    assert!(
        testkit::count(&low, (0, 0, 340, 100), whitish) > 100,
        "the number stays white"
    );

    styled.parts.styles.insert(
        TextPart::Suffix,
        PartStyle {
            baseline: Baseline::Super,
            ..suffix
        },
    );
    let high = testkit::render(&mut renderer, &text(frame, sensor(), styled), &scene);
    let (high_top, high_bottom) = red_rows(&high).expect("a raised suffix");
    assert!(
        high_top + 10 < low_top && high_bottom + 10 < low_bottom,
        "raised"
    );
    assert!(
        high_top.abs_diff(number.1) <= 4,
        "its top is level with the number's: {high_top} vs {}",
        number.1
    );
}

#[test]
fn clock_parts_take_their_colour_and_separators_blink_only_with_motion() {
    let mut renderer = testkit::renderer();
    let frame = BoxF::new(10.0, 10.0, 300.0, 80.0);
    let mut s = style(48.0);
    s.parts.styles.insert(
        TextPart::Minutes,
        PartStyle {
            color: Some(RED),
            ..PartStyle::default()
        },
    );
    let theme = text(frame, clock("%H:%M"), s.clone());
    let mut scene = Scene::empty();
    let shown = testkit::render(&mut renderer, &theme, &scene);
    let red = testkit::count(&shown, (0, 0, 340, 100), reddish);
    let white = testkit::count(&shown, (0, 0, 340, 100), whitish);
    assert!(
        red > 100 && white > 100,
        "red minutes, white hours: {red} {white}"
    );
    assert_eq!(renderer.next_change(), None, "nothing blinks");

    s.parts.blink_separators = true;
    let blinking = text(frame, clock("%H:%M"), s);
    let on = testkit::render(&mut renderer, &blinking, &scene);
    assert_eq!(on, shown, "first half of the second: shown");
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(500)));
    scene.animation = Duration::from_millis(600);
    let off = testkit::render(&mut renderer, &blinking, &scene);
    assert!(
        testkit::count(&off, (0, 0, 340, 100), whitish) < white,
        "colon hidden"
    );
    assert_eq!(
        testkit::count(&off, (0, 0, 340, 100), reddish),
        red,
        "the minutes keep their place"
    );
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(1000)));
    renderer.set_motion(false);
    assert_eq!(
        testkit::render(&mut renderer, &blinking, &scene),
        shown,
        "reduced motion"
    );
    assert_eq!(renderer.next_change(), None);
}

fn long(overflow: Option<TextOverflow>) -> Theme {
    let mut s = style(24.0);
    s.overflow = overflow;
    text(
        BoxF::new(20.0, 10.0, 120.0, 40.0),
        TextContent::Static("A rather long line of text".into()),
        s,
    )
}

#[test]
fn a_long_line_shrinks_ends_with_an_ellipsis_or_scrolls() {
    let mut renderer = testkit::renderer();
    let mut scene = Scene::empty();
    let ink = |frame: &Frame| testkit::ink(frame, Rgba::BLACK, 40).expect("ink");
    let clipped = ink(&testkit::render(&mut renderer, &long(None), &scene));
    assert!(clipped.2 >= 128, "drawn to the box edge: {clipped:?}");

    let shrunk = ink(&testkit::render(
        &mut renderer,
        &long(Some(TextOverflow::Shrink)),
        &scene,
    ));
    assert!(
        shrunk.0 >= 20 && shrunk.2 < 140,
        "inside the box: {shrunk:?}"
    );
    assert!(shrunk.3 - shrunk.1 < clipped.3 - clipped.1, "smaller");

    let cut = testkit::render(&mut renderer, &long(Some(TextOverflow::Ellipsis)), &scene);
    assert_ne!(cut, testkit::render(&mut renderer, &long(None), &scene));
    let cut_ink = ink(&cut);
    assert!(cut_ink.2 < 140, "ends inside the box: {cut_ink:?}");
    assert_eq!(cut_ink.3 - cut_ink.1, clipped.3 - clipped.1, "same size");
    let short = text(
        BoxF::new(20.0, 10.0, 120.0, 40.0),
        TextContent::Static("Short".into()),
        TextStyle {
            overflow: Some(TextOverflow::Ellipsis),
            ..style(24.0)
        },
    );
    let mut plain = short.clone();
    if let ElementKind::Text { style, .. } = &mut plain.elements[0].kind {
        style.overflow = None;
    }
    assert_eq!(
        testkit::render(&mut renderer, &short, &scene),
        testkit::render(&mut renderer, &plain, &scene),
        "a line that fits is left alone"
    );

    let scrolling = long(Some(TextOverflow::Scroll));
    let start = testkit::render(&mut renderer, &scrolling, &scene);
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(50)));
    scene.animation = Duration::from_secs(1);
    let later = testkit::render(&mut renderer, &scrolling, &scene);
    assert_ne!(start, later, "it moved");
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(1050)));
    renderer.set_motion(false);
    let still = testkit::render(&mut renderer, &scrolling, &scene);
    assert_eq!(still, start, "reduced motion holds the start");
    assert_eq!(renderer.next_change(), None);
}

fn rolling(transitions: &Transitions, theme: &Theme, at: Duration) -> Scene {
    let mut scene = Scene::empty().with(KEY, 41.0);
    scene.shown = transitions.shown(theme, at, View::Screen);
    scene
}

#[test]
fn rolling_digits_turn_from_the_old_number_to_the_new_one() {
    let mut renderer = testkit::renderer();
    let frame = BoxF::new(10.0, 10.0, 200.0, 70.0);
    let mut theme = text(frame, sensor(), style(48.0));
    theme.elements[0].value_transition = Some(ValueTransition {
        mode: TransitionMode::Roll,
        duration_ms: 400,
        average: 1,
    });
    let mut transitions = Transitions::default();
    let mut readings = Snapshot::default();
    readings.insert(
        testkit::key(KEY),
        bezel_core::domain::sensor::Reading::Value(36.0),
    );
    transitions.retarget(&theme, &readings, Some(Duration::ZERO));
    readings.insert(
        testkit::key(KEY),
        bezel_core::domain::sensor::Reading::Value(41.0),
    );
    transitions.retarget(&theme, &readings, Some(Duration::from_secs(1)));

    let mut still = theme.clone();
    still.elements[0].value_transition = None;
    let shows = |renderer: &mut crate::SkiaRenderer, value: f64| {
        testkit::render(renderer, &still, &Scene::empty().with(KEY, value))
    };
    let thirty_six = shows(&mut renderer, 36.0);
    let forty_one = shows(&mut renderer, 41.0);
    let at = |ms: u64| Duration::from_millis(1_000 + ms);
    let first = testkit::render(&mut renderer, &theme, &rolling(&transitions, &theme, at(0)));
    assert_eq!(first, thirty_six, "starts on the old number");
    let middle = testkit::render(
        &mut renderer,
        &theme,
        &rolling(&transitions, &theme, at(100)),
    );
    assert_ne!(middle, thirty_six);
    assert_ne!(middle, forty_one);
    let inside = testkit::ink(&middle, Rgba::BLACK, 40).expect("ink");
    let line = testkit::ink(&thirty_six, Rgba::BLACK, 40).expect("ink");
    assert!(
        inside.1 + 12 >= line.1 && inside.3 <= line.3 + 12,
        "wheels stay in their line"
    );
    let last = testkit::render(
        &mut renderer,
        &theme,
        &rolling(&transitions, &theme, at(400)),
    );
    assert_eq!(last, forty_one, "lands on the new number");
    assert_eq!(ElementId(1), theme.elements[0].id);
}

#[test]
fn natural_size_counts_parts_and_equal_digits() {
    let mut renderer = testkit::renderer();
    let quantities = bezel_core::domain::sensor::Quantities::new();
    let mut measure = |theme: &Theme, scene: &Scene| {
        let context = bezel_core::ports::RenderContext {
            snapshot: &scene.snapshot,
            histories: &scene.histories,
            quantities: &quantities,
            time: scene.time,
            animation: scene.animation,
            language: bezel_core::domain::clock::Language::English,
            backdrop: bezel_core::ports::Backdrop::Poster,
            shown: &scene.shown,
        };
        renderer
            .measure_element(theme, ElementId(1), &scene.assets, context)
            .expect("measure")
            .expect("a size")
    };
    let frame = BoxF::new(10.0, 10.0, 300.0, 80.0);
    let scene = Scene::empty().with(KEY, 36.0);
    let plain = measure(&text(frame, sensor(), style(64.0)), &scene);
    let mut small = style(64.0);
    small.parts.styles.insert(
        TextPart::Suffix,
        PartStyle {
            size_percent: Some(50.0),
            ..PartStyle::default()
        },
    );
    let parted = measure(&text(frame, sensor(), small), &scene);
    assert!(parted.width < plain.width, "a smaller suffix is narrower");
    assert_eq!(parted.height, plain.height, "the number sets the line");
}
