//! The extended needle on the canvas (D-2026-10-20-studio-ux-v2-08).
use std::time::Duration;

use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::needle::{
    CenterValue, NeedleLook, NeedleShape, PivotCap, ScaleNumbers, ScaleTicks, Track, Zone,
};
use bezel_core::domain::sensor::DisplayFormat;
use bezel_core::domain::theme::{
    Background, Binding, BoxF, ElementKind, FontSpec, HAlign, TextStyle, Theme, VAlign,
};
use bezel_core::ports::FrameRenderer;

use crate::testkit::{self, Scene};

const KEY: &str = "cpu.usage";
const GREEN: Rgba = Rgba::opaque(0, 200, 0);
const RED: Rgba = Rgba::opaque(220, 0, 0);
const BLUE: Rgba = Rgba::opaque(0, 0, 230);
const MARKS: Rgba = Rgba::opaque(200, 200, 200);

fn look() -> NeedleLook {
    NeedleLook {
        track: Track::Zones,
        zones: vec![
            Zone {
                from: 0.0,
                to: 50.0,
                color: GREEN,
            },
            Zone {
                from: 50.0,
                to: 100.0,
                color: RED,
            },
        ],
        color_from_zone: true,
        ticks: ScaleTicks::None,
        numbers: ScaleNumbers::None,
        marks: MARKS,
        cap: PivotCap::Ring,
        cap_diameter: 20.0,
        cap_color: BLUE,
        ..NeedleLook::default()
    }
}

fn theme(look: Option<NeedleLook>) -> Theme {
    let needle = ElementKind::Needle {
        binding: Binding {
            key: testkit::key(KEY),
            min: 0.0,
            max: 100.0,
        },
        asset: None,
        pivot: (0.5, 0.5),
        start_angle: -90.0,
        sweep: 180.0,
        color: Rgba::WHITE,
        width: 4.0,
        look: look.map(Box::new),
    };
    let element = testkit::element(BoxF::new(0.0, 0.0, 200.0, 200.0), needle);
    testkit::theme(200, 200, Background::Color(Rgba::BLACK), vec![element])
}

fn reading(value: f64) -> Scene {
    Scene::empty().with(KEY, value)
}

fn near(frame: &Frame, (x, y): (u32, u32), pred: impl Fn(Rgba) -> bool) -> usize {
    testkit::count(frame, (x - 3, y - 3, x + 4, y + 4), pred)
}

#[test]
fn zones_colour_the_arc_and_the_needle() {
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme(Some(look())), &reading(25.0));
    // Radius 96.5: a quarter and three quarters of the scale at ±45°.
    testkit::assert_px(&frame, 32, 32, GREEN, 30);
    testkit::assert_px(&frame, 168, 32, RED, 30);
    testkit::assert_px(&frame, 69, 69, GREEN, 30);
    testkit::assert_px(&frame, 100, 100, BLUE, 10);
    testkit::assert_px(&frame, 100, 111, GREEN, 40);

    let hot = testkit::render(&mut renderer, &theme(Some(look())), &reading(75.0));
    testkit::assert_px(&hot, 131, 69, RED, 30);
    let own = NeedleLook {
        color_from_zone: false,
        ..look()
    };
    let white = testkit::render(&mut renderer, &theme(Some(own)), &reading(25.0));
    testkit::assert_px(&white, 69, 69, Rgba::WHITE, 30);

    let single = NeedleLook {
        track: Track::Single,
        ..look()
    };
    let faint = testkit::render(&mut renderer, &theme(Some(single)), &reading(25.0));
    let arc = testkit::px(&faint, 168, 32);
    assert!(
        arc.r > 30 && arc.r < 90 && arc.r == arc.g,
        "faint marks: {arc:?}"
    );

    let gradient = NeedleLook {
        track: Track::Gradient,
        ..look()
    };
    let blended = testkit::render(&mut renderer, &theme(Some(gradient)), &reading(25.0));
    let top = testkit::px(&blended, 100, 4);
    assert!(
        top.r > 60 && top.g > 60,
        "green and red meet at the top: {top:?}"
    );
}

#[test]
fn an_old_needle_is_drawn_as_before() {
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme(None), &reading(50.0));
    testkit::assert_px(&frame, 100, 20, Rgba::WHITE, 10);
    testkit::assert_px(&frame, 32, 32, Rgba::BLACK, 0);
}

#[test]
fn the_peak_mark_holds_the_highest_reading_then_falls() {
    let mut renderer = testkit::renderer();
    let held = NeedleLook {
        peak_seconds: Some(10),
        ..look()
    };
    let t = theme(Some(held));
    let mut scene = reading(80.0);
    testkit::render(&mut renderer, &t, &scene);
    scene = reading(25.0);
    scene.animation = Duration::from_secs(1);
    let frame = testkit::render(&mut renderer, &t, &scene);
    // Half way out along 54° (80 % of the scale).
    let ghost = near(&frame, (140, 71), |p| p.r > 20 && p.r < 140);
    assert!(ghost > 0, "a faint hand stays at the peak");
    assert_eq!(renderer.next_change(), Some(Duration::from_secs(10)));
    scene.animation = Duration::from_secs(11);
    let fallen = testkit::render(&mut renderer, &t, &scene);
    assert_eq!(
        near(&fallen, (140, 71), |p| p.r > 20 && p.r < 140),
        0,
        "gone"
    );
}

#[test]
fn ticks_numbers_shapes_cap_and_centre_value_draw() {
    let mut renderer = testkit::renderer();
    let grey = |frame: &Frame| {
        testkit::count(frame, (0, 0, 200, 200), |p| {
            p.r > 150 && p.r == p.g && p.g == p.b && p.r < 230
        })
    };
    let plain = grey(&testkit::render(
        &mut renderer,
        &theme(Some(look())),
        &reading(25.0),
    ));
    let ticked = NeedleLook {
        ticks: ScaleTicks::MajorMinor,
        ..look()
    };
    let ticks = grey(&testkit::render(
        &mut renderer,
        &theme(Some(ticked.clone())),
        &reading(25.0),
    ));
    let major_only = grey(&testkit::render(
        &mut renderer,
        &theme(Some(NeedleLook {
            ticks: ScaleTicks::Major,
            ..look()
        })),
        &reading(25.0),
    ));
    assert!(
        ticks > major_only && major_only > plain,
        "{plain} {major_only} {ticks}"
    );
    let numbered = grey(&testkit::render(
        &mut renderer,
        &theme(Some(NeedleLook {
            numbers: ScaleNumbers::Major,
            ..ticked
        })),
        &reading(25.0),
    ));
    assert!(numbered > ticks, "{numbered} {ticks}");

    let greens: Vec<usize> = [
        NeedleShape::Thin,
        NeedleShape::Tapered,
        NeedleShape::Arrow,
        NeedleShape::Dot,
    ]
    .into_iter()
    .map(|shape| {
        let frame = testkit::render(
            &mut renderer,
            &theme(Some(NeedleLook {
                shape,
                track: Track::Single,
                cap: PivotCap::None,
                ..look()
            })),
            &reading(25.0),
        );
        testkit::count(&frame, (0, 0, 200, 200), |p| p.g > 150 && p.r < 60)
    })
    .collect();
    let distinct: std::collections::BTreeSet<_> = greens.iter().collect();
    assert_eq!(distinct.len(), 4, "every shape differs: {greens:?}");

    let filled = testkit::render(
        &mut renderer,
        &theme(Some(NeedleLook {
            cap: PivotCap::Filled,
            ..look()
        })),
        &reading(25.0),
    );
    testkit::assert_px(&filled, 100, 108, BLUE, 10);
    let none = testkit::render(
        &mut renderer,
        &theme(Some(NeedleLook {
            cap: PivotCap::None,
            ..look()
        })),
        &reading(25.0),
    );
    assert!(testkit::px(&none, 100, 100).b < 100, "no cap");

    let center = NeedleLook {
        center: Some(CenterValue {
            style: TextStyle {
                size: 20.0,
                font: FontSpec {
                    family: testkit::FAMILY.into(),
                    ..FontSpec::default()
                },
                align: HAlign::Center,
                valign: VAlign::Middle,
                ..TextStyle::default()
            },
            format: DisplayFormat::default(),
            prefix: String::new(),
            suffix: String::new(),
        }),
        ..look()
    };
    let written = testkit::render(&mut renderer, &theme(Some(center)), &reading(25.0));
    let text = testkit::count(&written, (60, 114, 140, 145), |p| {
        p.r > 200 && p.g > 200 && p.b > 200
    });
    assert!(text > 30, "the value under the pivot: {text}");
}
