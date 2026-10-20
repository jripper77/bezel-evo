//! Moving weather icons on the canvas (D-2026-10-20-studio-ux-v2-09).
use std::time::Duration;

use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::theme::{
    Background, BoxF, ElementKind, FontSpec, TextContent, TextStyle, Theme,
};
use bezel_core::domain::weather::{IconMotion, IconStyle, Weather};
use bezel_core::ports::FrameRenderer;

use crate::testkit::{self, Scene};

fn weather(motion: IconMotion, style: IconStyle) -> Weather {
    Weather {
        city: "Roma".into(),
        latitude: 41.9,
        longitude: 12.5,
        language: None,
        fahrenheit: false,
        show_icon: true,
        icon_style: style,
        icon_gap: Some(4.0),
        icon_size: Some(96.0),
        icon_motion: motion,
        icon_speed: 1.0,
    }
}

fn theme(w: Weather) -> Theme {
    let style = TextStyle {
        size: 4.0,
        font: FontSpec {
            family: testkit::FAMILY.into(),
            ..FontSpec::default()
        },
        ..TextStyle::default()
    };
    let element = testkit::element(
        BoxF::new(0.0, 0.0, 120.0, 120.0),
        ElementKind::Text {
            content: TextContent::Weather(w),
            style,
        },
    );
    testkit::theme(120, 120, Background::Color(Rgba::BLACK), vec![element])
}

fn at(w: &Weather, code: f64, ms: u64) -> Scene {
    let [temperature, condition] = w.keys();
    let mut scene = Scene::empty()
        .with(temperature.as_str(), 20.0)
        .with(condition.as_str(), code);
    scene.animation = Duration::from_millis(ms);
    scene
}

/// The icon's box (the text beside it is a few pixels wide).
fn icon(frame: &Frame) -> usize {
    testkit::count(frame, (0, 0, 98, 120), |p| p.r > 40 || p.g > 40 || p.b > 40)
}

#[test]
fn a_still_icon_never_moves_and_asks_for_no_frames() {
    let mut renderer = testkit::renderer();
    let w = weather(IconMotion::Still, IconStyle::Outline);
    let first = testkit::render(&mut renderer, &theme(w.clone()), &at(&w, 61.0, 200));
    let later = testkit::render(&mut renderer, &theme(w.clone()), &at(&w, 61.0, 700));
    assert_eq!(first, later);
    assert_eq!(renderer.next_change(), None);
}

#[test]
fn a_moving_icon_moves_at_its_pace_and_stops_with_reduced_motion() {
    let mut renderer = testkit::renderer();
    for style in [
        IconStyle::Outline,
        IconStyle::Filled,
        IconStyle::Colored,
        IconStyle::Dimensional,
    ] {
        let w = weather(IconMotion::Lively, style);
        let t = theme(w.clone());
        let a = testkit::render(&mut renderer, &t, &at(&w, 61.0, 200));
        assert_eq!(renderer.next_change(), Some(Duration::from_millis(233)));
        let b = testkit::render(&mut renderer, &t, &at(&w, 61.0, 700));
        assert_ne!(a, b, "{style:?}: drops fall");
    }
    let gentle = weather(IconMotion::Gentle, IconStyle::Outline);
    testkit::render(
        &mut renderer,
        &theme(gentle.clone()),
        &at(&gentle, 0.0, 100),
    );
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(150)));

    let w = weather(IconMotion::Lively, IconStyle::Outline);
    let still = weather(IconMotion::Still, IconStyle::Outline);
    let rest = testkit::render(&mut renderer, &theme(still.clone()), &at(&still, 3.0, 700));
    renderer.set_motion(false);
    let reduced = testkit::render(&mut renderer, &theme(w.clone()), &at(&w, 3.0, 700));
    assert_eq!(reduced, rest, "reduced motion: the icon at rest");
    assert_eq!(renderer.next_change(), None);
}

#[test]
fn every_family_moves_in_every_style() {
    for code in [0.0, 2.0, 3.0, 45.0, 61.0, 71.0, 95.0, 999.0] {
        for style in [
            IconStyle::Outline,
            IconStyle::Colored,
            IconStyle::Dimensional,
        ] {
            let mut renderer = testkit::renderer();
            let w = weather(IconMotion::Lively, style);
            let t = theme(w.clone());
            let frames: Vec<Frame> = [300u64, 1_100, 1_800, 2_600]
                .into_iter()
                .map(|ms| testkit::render(&mut renderer, &t, &at(&w, code, ms)))
                .collect();
            assert!(
                frames.windows(2).any(|f| f[0] != f[1]),
                "code {code} {style:?} moves"
            );
            assert!(
                frames.iter().all(|f| icon(f) > 50),
                "code {code} {style:?} stays visible"
            );
        }
    }
}

#[test]
fn a_new_condition_fades_in_over_the_old_one() {
    let w = weather(IconMotion::Lively, IconStyle::Colored);
    let t = theme(w.clone());
    let mut renderer = testkit::renderer();
    testkit::render(&mut renderer, &t, &at(&w, 0.0, 1_000));
    let fading = testkit::render(&mut renderer, &t, &at(&w, 71.0, 1_100));
    let mut fresh = testkit::renderer();
    let snow = testkit::render(&mut fresh, &t, &at(&w, 71.0, 1_100));
    assert_ne!(fading, snow, "the sun still shows while the snow fades in");
    let done = testkit::render(&mut renderer, &t, &at(&w, 71.0, 1_400));
    let snow_later = testkit::render(&mut fresh, &t, &at(&w, 71.0, 1_400));
    assert_eq!(done, snow_later, "after 300 ms only the snow");
}

#[test]
fn the_3d_storm_flashes_with_the_lightning() {
    let w = weather(IconMotion::Lively, IconStyle::Dimensional);
    let t = theme(w.clone());
    let mut renderer = testkit::renderer();
    let dark = testkit::render(&mut renderer, &t, &at(&w, 95.0, 300));
    let flash = testkit::render(&mut renderer, &t, &at(&w, 95.0, 1_800));
    assert!(
        testkit::px(&flash, 2, 14).r > testkit::px(&dark, 2, 14).r,
        "the box lights up"
    );
}
