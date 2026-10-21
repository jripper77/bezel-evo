//! Analog clocks on the canvas: hands, dial, marks, numbers, date, shapes
//! and frames (D-2026-10-20-studio-ux-v2-07).
use std::time::Duration;

use bezel_core::domain::clock::{
    AnalogClock, DialDate, DialNumbers, DialTicks, HandShape, LocalTime,
};
use bezel_core::domain::frame::Rgba;
use bezel_core::domain::theme::{
    Background, BoxF, ElementKind, FontSpec, TextContent, TextStyle, Theme,
};
use bezel_core::ports::FrameRenderer;

use crate::testkit::{self, Scene};

const HOUR: Rgba = Rgba::opaque(255, 0, 0);
const MINUTE: Rgba = Rgba::opaque(0, 255, 0);
const SECOND: Rgba = Rgba::opaque(0, 0, 255);
const FACE: Rgba = Rgba::opaque(40, 40, 40);

fn theme(clock: AnalogClock) -> Theme {
    let style = TextStyle {
        font: FontSpec {
            family: testkit::FAMILY.into(),
            ..FontSpec::default()
        },
        ..TextStyle::default()
    };
    let content = TextContent::Clock {
        pattern: "%H:%M".into(),
        language: None,
        casing: Default::default(),
        analog: Some(clock),
    };
    let element = testkit::element(
        BoxF::new(0.0, 0.0, 256.0, 256.0),
        ElementKind::Text { content, style },
    );
    testkit::theme(256, 256, Background::Color(Rgba::BLACK), vec![element])
}

fn clock() -> AnalogClock {
    AnalogClock {
        face: FACE,
        hour_color: HOUR,
        minute_color: MINUTE,
        second_color: SECOND,
        ..AnalogClock::default()
    }
}

fn at(hour: u8, minute: u8, second: u8, millisecond: u16) -> Scene {
    let mut scene = Scene::empty();
    scene.time = LocalTime {
        hour,
        minute,
        second,
        millisecond,
        ..testkit::TIME
    };
    scene
}

#[test]
fn the_dial_shows_its_hands_on_its_face() {
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme(clock()), &at(3, 0, 30, 0));
    // A unit is 4 px: hours to the right, minutes up, seconds down.
    testkit::assert_px(&frame, 128 + 30, 128, HOUR, 8);
    testkit::assert_px(&frame, 128, 128 - 60, MINUTE, 8);
    testkit::assert_px(&frame, 128, 128 + 70, SECOND, 8);
    testkit::assert_px(&frame, 128 - 40, 128 + 40, FACE, 4);
    testkit::assert_px(&frame, 4, 4, Rgba::BLACK, 0);
    testkit::assert_px(&frame, 128, 128, SECOND, 8);
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(1000)));

    let no_seconds = AnalogClock {
        second_hand: false,
        ..clock()
    };
    let frame = testkit::render(&mut renderer, &theme(no_seconds), &at(3, 0, 30, 0));
    testkit::assert_px(&frame, 128, 128 + 70, FACE, 4);
    testkit::assert_px(&frame, 128, 128, MINUTE, 8);
    assert_eq!(
        renderer.next_change(),
        None,
        "the theme's refresh moves the rest"
    );
}

#[test]
fn a_sweeping_second_hand_moves_between_seconds_unless_motion_is_reduced() {
    let mut renderer = testkit::renderer();
    let sweeping = theme(AnalogClock {
        sweep: true,
        ..clock()
    });
    let on_the_second = testkit::render(&mut renderer, &sweeping, &at(3, 0, 30, 0));
    let half_past = testkit::render(&mut renderer, &sweeping, &at(3, 0, 30, 500));
    assert_ne!(on_the_second, half_past);
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(33)));
    renderer.set_motion(false);
    let still = testkit::render(&mut renderer, &sweeping, &at(3, 0, 30, 500));
    assert_eq!(still, on_the_second, "steps with reduced motion");
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(500)));
}

#[test]
fn marks_numbers_date_outline_and_hand_shapes_draw() {
    let mut renderer = testkit::renderer();
    let plain = AnalogClock {
        ticks: DialTicks::None,
        hour_hand: false,
        minute_hand: false,
        second_hand: false,
        border: 0.0,
        ..clock()
    };
    let light = |renderer: &mut crate::SkiaRenderer, c: AnalogClock| {
        let frame = testkit::render(renderer, &theme(c), &at(3, 0, 0, 0));
        testkit::count(&frame, (0, 0, 256, 256), |p| {
            p.r > 120 && p.g > 120 && p.b > 120
        })
    };
    let with = |change: fn(&mut AnalogClock)| {
        let mut c = plain;
        change(&mut c);
        c
    };
    assert_eq!(light(&mut renderer, plain), 0, "an empty dial");
    let hours = light(&mut renderer, with(|c| c.ticks = DialTicks::Hours));
    let minutes = light(&mut renderer, with(|c| c.ticks = DialTicks::Minutes));
    assert!(hours > 100 && minutes > hours, "{hours} {minutes}");
    let quarters = light(&mut renderer, with(|c| c.numbers = DialNumbers::Quarters));
    let all = light(&mut renderer, with(|c| c.numbers = DialNumbers::All));
    let roman = light(&mut renderer, with(|c| c.numbers = DialNumbers::Roman));
    assert!(
        quarters > 50 && all > quarters && roman > quarters,
        "{quarters} {all} {roman}"
    );
    let day = light(&mut renderer, with(|c| c.date = DialDate::Day));
    let day_month = light(&mut renderer, with(|c| c.date = DialDate::DayMonth));
    assert!(day > 20 && day_month > day, "{day} {day_month}");
    let bordered = testkit::render(
        &mut renderer,
        &theme(with(|c| c.border = 4.0)),
        &at(3, 0, 0, 0),
    );
    let outline = testkit::px(&bordered, 128, 128 - 112);
    assert!(
        outline.r > FACE.r && outline.r < 120,
        "a fainter outline: {outline:?}"
    );

    let red = |renderer: &mut crate::SkiaRenderer, hand_shape| {
        let frame = testkit::render(
            renderer,
            &theme(AnalogClock {
                hand_shape,
                ..clock()
            }),
            &at(3, 0, 0, 0),
        );
        testkit::count(&frame, (0, 0, 256, 256), |p| p.r > 200 && p.g < 50)
    };
    let thin = red(&mut renderer, HandShape::Thin);
    let rounded = red(&mut renderer, HandShape::Rounded);
    let arrow = red(&mut renderer, HandShape::Arrow);
    assert!(thin < rounded, "thin is thinner: {thin} {rounded}");
    assert_ne!(arrow, rounded, "an arrowhead");
}

#[test]
fn legacy_hand_length_pixels_are_stable() {
    let mut renderer = testkit::renderer();
    for (shape, expected) in [
        (HandShape::Thin, 5866446313607721224),
        (HandShape::Rounded, 4581605130595532614),
        (HandShape::Arrow, 619349141455207611),
    ] {
        let frame = testkit::render(
            &mut renderer,
            &theme(AnalogClock {
                hand_shape: shape,
                ..clock()
            }),
            &at(3, 0, 30, 0),
        );
        let hash = frame
            .as_rgba()
            .iter()
            .fold(0xcbf29ce484222325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
            });
        assert_eq!(hash, expected, "legacy {shape:?} pixels changed");
    }
}

#[test]
fn each_hand_length_changes_only_its_tip_and_preserves_the_second_tail() {
    let mut renderer = testkit::renderer();
    let scene = at(3, 0, 30, 0);
    let original = testkit::render(&mut renderer, &theme(clock()), &scene);
    for (change, beyond, inside, untouched) in [
        (0, (168, 128), (144, 128), (128, 68)),
        (1, (128, 68), (128, 100), (158, 128)),
        (2, (128, 198), (128, 160), (158, 128)),
    ] {
        let mut c = clock();
        match change {
            0 => c.hour_length = 0.5,
            1 => c.minute_length = 0.5,
            _ => c.second_length = 0.5,
        }
        let shorter = testkit::render(&mut renderer, &theme(c), &scene);
        testkit::assert_px(&shorter, beyond.0, beyond.1, FACE, 8);
        testkit::assert_px(
            &shorter,
            inside.0,
            inside.1,
            [HOUR, MINUTE, SECOND][change],
            8,
        );
        assert_eq!(
            testkit::px(&shorter, untouched.0, untouched.1),
            testkit::px(&original, untouched.0, untouched.1)
        );
        if change == 2 {
            // The second hand points down; its unchanged tail crosses the minute hand.
            assert_eq!(
                testkit::px(&shorter, 128, 110),
                testkit::px(&original, 128, 110)
            );
        }
    }
}
