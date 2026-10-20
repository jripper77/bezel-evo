//! Rotation geometry and straight-alpha pixel probes shared with Studio.
use crate::testkit::{self, Scene};
use bezel_core::domain::{
    frame::{Frame, Rgba, dirty_rects},
    spectrum::Spectrum,
    theme::*,
};

fn box_element() -> Element {
    testkit::element(
        BoxF::new(50., 45., 60., 30.),
        ElementKind::Shape {
            image: None,
            video_window: false,
            fade: None,
            shape: ShapeKind::Rect { radius: 0. },
            fill: Some(Paint::solid(Rgba::opaque(255, 0, 0))),
            stroke: None,
        },
    )
}
fn draw(e: Element) -> Frame {
    testkit::render(
        &mut testkit::renderer(),
        &testkit::theme(160, 120, Background::Color(Rgba::default()), vec![e]),
        &Scene::empty(),
    )
}
#[test]
fn rotation_zero_keeps_pixels_identical() {
    let e = box_element();
    let plain = draw(e.clone());
    let mut zero = e;
    zero.rotation = 360.;
    assert_eq!(plain, draw(zero));
}

#[test]
fn sensor_rotors_render_real_pixels_stop_and_keep_scenes_independent() {
    use bezel_core::{
        domain::{sensor::Reading, sensor_motion::SensorMotion},
        ports::FrameRenderer,
    };
    use std::time::Duration;
    let mut e = box_element();
    e.sensor_motion = Some(SensorMotion {
        sensor: testkit::key("gpu.fan"),
        reference: 100.,
        turns_per_second: 1.,
        inertia_ms: 0,
        reverse: false,
    });
    let theme = testkit::theme(160, 120, Background::Color(Rgba::default()), vec![e]);
    let mut scene = Scene::empty();
    scene
        .snapshot
        .insert(testkit::key("gpu.fan"), Reading::Value(100.));
    let mut renderer = testkit::renderer();
    renderer.set_scene(1);
    let upright = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(20)));
    scene.animation = Duration::from_millis(100);
    let moving = testkit::render(&mut renderer, &theme, &scene);
    assert_ne!(upright, moving);
    renderer.set_scene(2);
    assert_eq!(
        testkit::render(&mut renderer, &theme, &scene),
        upright,
        "new display has its own phase"
    );
    renderer.set_scene(1);
    scene.animation = Duration::from_millis(200);
    scene
        .snapshot
        .insert(testkit::key("gpu.fan"), Reading::Value(0.));
    assert_eq!(
        testkit::render(&mut renderer, &theme, &scene),
        moving,
        "zero holds its final angle"
    );
    assert_eq!(renderer.next_change(), None);
    renderer.set_motion(false);
    assert_eq!(testkit::render(&mut renderer, &theme, &scene), upright);
    assert_eq!(renderer.next_change(), None);
    assert_eq!(
        theme.elements[0].rotation, 0.,
        "presentation never edits the theme"
    );
}

#[test]
fn a_sensor_rotor_inside_a_card_rotates_without_turning_the_container() {
    use bezel_core::domain::{sensor::Reading, sensor_motion::SensorMotion};
    use std::time::Duration;
    let mut rotor = box_element();
    rotor.id = ElementId(2);
    rotor.card_member = Some(CardMember {
        parent: ElementId(1),
        face: Some(0),
    });
    rotor.sensor_motion = Some(SensorMotion {
        sensor: testkit::key("gpu.fan"),
        reference: 100.,
        turns_per_second: 1.,
        inertia_ms: 0,
        reverse: false,
    });
    let mut card = box_element();
    card.id = ElementId(1);
    card.card = Some(Card {
        faces: vec!["Fan".into()],
        active_face: 0,
        triggers: vec![],
        rotation_seconds: None,
        transition: None,
    });
    card.opacity = 0.;
    let theme = testkit::theme(
        160,
        120,
        Background::Color(Rgba::default()),
        vec![card, rotor],
    );
    // A transparent parent would hide its child, so leave the container's
    // opacity intact and make its own shape transparent instead.
    let mut theme = theme;
    theme.elements[0].opacity = 1.;
    if let ElementKind::Shape { fill, .. } = &mut theme.elements[0].kind {
        *fill = None;
    }
    let mut scene = Scene::empty();
    scene
        .snapshot
        .insert(testkit::key("gpu.fan"), Reading::Value(100.));
    let mut renderer = testkit::renderer();
    let before = testkit::render(&mut renderer, &theme, &scene);
    scene.animation = Duration::from_millis(100);
    assert_ne!(testkit::render(&mut renderer, &theme, &scene), before);
    assert_eq!(theme.elements[0].applied_rotation(), 0.);
}
#[test]
fn rotation_90_turns_box_around_center() {
    let mut e = box_element();
    e.rotation = 90.;
    let f = draw(e);
    assert_eq!(f.pixel(80, 35), Some(Rgba::opaque(255, 0, 0)));
    assert_eq!(f.pixel(55, 60).unwrap().a, 0);
}
#[test]
fn rotation_45_blends_with_opacity() {
    let mut e = box_element();
    e.rotation = 45.;
    e.opacity = 0.5;
    let f = draw(e);
    let p = f.pixel(80, 60).unwrap();
    assert_eq!(p.r, 255);
    assert!((i32::from(p.a) - 128).abs() <= 1);
    assert_eq!(f.pixel(50, 45).unwrap().a, 0);
}
#[test]
fn rotation_30_matches_parity_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../bezel-core/tests/fixtures/rotation-parity.json"
    ))
    .unwrap();
    for opacity in fixture["opacities"].as_array().unwrap() {
        let mut e = box_element();
        e.rotation = fixture["rotation"].as_f64().unwrap() as f32;
        e.opacity = opacity.as_f64().unwrap() as f32;
        let expected = (255. * e.opacity).round() as i32;
        let f = draw(e);
        for point in fixture["ink"].as_array().unwrap() {
            let p = f
                .pixel(
                    point[0].as_u64().unwrap() as u32,
                    point[1].as_u64().unwrap() as u32,
                )
                .unwrap();
            assert!(
                p.r >= 252 && p.g <= 3 && p.b <= 3 && (i32::from(p.a) - expected).abs() <= 3,
                "{p:?}"
            );
        }
        for point in fixture["empty"].as_array().unwrap() {
            assert!(
                f.pixel(
                    point[0].as_u64().unwrap() as u32,
                    point[1].as_u64().unwrap() as u32
                )
                .unwrap()
                .a <= 3
            );
        }
    }
}
#[test]
fn dirty_rotated_covers_rotated_bbox() {
    let mut e = box_element();
    e.rotation = 30.;
    let f = draw(e);
    let blank = Frame::filled(f.size(), Rgba::default());
    let dirty = dirty_rects(&blank, &f);
    let mut outside = 0;
    for y in 0..120 {
        for x in 0..160 {
            if f.pixel(x, y).unwrap().a > 0 {
                if !(50..110).contains(&x) || !(45..75).contains(&y) {
                    outside += 1;
                }
                assert!(
                    dirty
                        .iter()
                        .any(|r| x >= r.x && y >= r.y && x < r.x + r.width && y < r.y + r.height)
                );
            }
        }
    }
    assert!(outside > 20);
}
#[test]
fn rotation_turns_text_graph_and_spectrum_content() {
    let kinds = [
        ElementKind::Text {
            content: TextContent::Static("Rotate".into()),
            style: TextStyle {
                size: 14.,
                font: FontSpec {
                    family: testkit::FAMILY.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ElementKind::Graph {
            binding: Binding {
                key: testkit::key("cpu.usage"),
                min: 0.,
                max: 100.,
            },
            history: 10,
            interval_seconds: None,
            style: GraphStyle::Line,
            color: Rgba::WHITE,
            fill: None,
            line_width: 2.,
            autoscale: false,
        },
        ElementKind::Text {
            content: TextContent::Spectrum(Spectrum {
                demo: true,
                ..Default::default()
            }),
            style: TextStyle::default(),
        },
        ElementKind::Image {
            asset: AssetRef("test.png".into()),
            fit: Fit::Fill,
            anchor: Default::default(),
            playback: Default::default(),
            fade: None,
        },
    ];
    let scene = Scene::empty()
        .history("cpu.usage", &[Some(10.), Some(80.), Some(30.)])
        .asset(
            "test.png",
            testkit::png(
                4,
                4,
                |x, y| if x > y { Rgba::WHITE } else { Rgba::default() },
            ),
        );
    for kind in kinds {
        let e = testkit::element(box_element().frame, kind);
        let mut t = testkit::theme(160, 120, Background::Color(Rgba::default()), vec![e]);
        let mut r = testkit::renderer();
        let original = testkit::render(&mut r, &t, &scene);
        t.elements[0].rotation = 90.;
        let turned = testkit::render(&mut r, &t, &scene);
        assert!(
            original
                .as_rgba()
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| p[3] > 0)
        );
        for y in 46..74 {
            for x in 51..109 {
                let a = original.pixel(x, y).unwrap();
                let b = turned.pixel(139 - y, x - 20).unwrap();
                assert!(
                    (i32::from(a.a) - i32::from(b.a)).abs() <= 2,
                    "{x},{y}: {a:?} {b:?}"
                );
            }
        }
    }
}
#[test]
fn rotation_ignored_for_cards_groups_and_video_windows() {
    let mut excluded = vec![];
    let mut e = box_element();
    e.is_group = true;
    excluded.push(e);
    let mut e = box_element();
    e.group_parent = Some(ElementId(9));
    excluded.push(e);
    let mut e = box_element();
    e.card_member = Some(CardMember {
        parent: ElementId(9),
        face: None,
    });
    excluded.push(e);
    let mut e = box_element();
    e.card = Some(Card {
        faces: vec!["Face".into()],
        active_face: 0,
        triggers: vec![],
        rotation_seconds: None,
        transition: None,
    });
    excluded.push(e);
    let mut e = box_element();
    if let ElementKind::Shape { video_window, .. } = &mut e.kind {
        *video_window = true;
    }
    excluded.push(e);
    for mut e in excluded {
        let plain = draw(e.clone());
        e.rotation = 30.;
        assert_eq!(e.applied_rotation(), 0.);
        assert_eq!(plain, draw(e));
    }
}
#[test]
fn rotation_keeps_offscreen_source_until_composition() {
    let mut e = box_element();
    e.frame = BoxF::new(-30., 40., 60., 20.);
    e.rotation = 90.;
    let f = draw(e);
    assert!(f.pixel(5, 25).unwrap().a > 0);
}
