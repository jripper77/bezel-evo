//! Image objects: what stays in view when the picture covers its box, and
//! how a GIF plays (D-2026-10-20-studio-ux-v2-04, Immagine).
use std::time::Duration;

use bezel_core::domain::frame::Rgba;
use bezel_core::domain::theme::{
    Anchor, AssetRef, Background, BoxF, ElementKind, Fit, Playback, Theme,
};

use crate::testkit::{self, Scene};

const RED: Rgba = Rgba::opaque(255, 0, 0);
const GREEN: Rgba = Rgba::opaque(0, 255, 0);
const BLUE: Rgba = Rgba::opaque(0, 0, 255);

fn image(asset: &str, fit: Fit, anchor: Anchor, playback: Playback) -> Theme {
    let element = testkit::element(
        BoxF::new(0.0, 0.0, 10.0, 10.0),
        ElementKind::Image {
            fade: None,
            asset: AssetRef(asset.into()),
            fit,
            anchor,
            playback,
        },
    );
    testkit::theme(10, 10, Background::Color(Rgba::BLACK), vec![element])
}

#[test]
fn the_anchor_keeps_its_part_of_a_covering_picture_in_view() {
    // 40×10: red on the left quarter, green in the middle, blue on the right.
    let wide = testkit::png(40, 10, |x, _| match x {
        0..10 => RED,
        10..30 => GREEN,
        _ => BLUE,
    });
    let scene = Scene::empty().asset("wide.png", wide);
    let mut renderer = testkit::renderer();
    let shown = |renderer: &mut crate::SkiaRenderer, anchor| {
        let frame = testkit::render(
            renderer,
            &image("wide.png", Fit::Cover, anchor, Playback::default()),
            &scene,
        );
        testkit::px(&frame, 5, 5)
    };
    assert!(testkit::close(shown(&mut renderer, Anchor::Left), RED, 8));
    assert!(testkit::close(
        shown(&mut renderer, Anchor::TopLeft),
        RED,
        8
    ));
    assert!(testkit::close(
        shown(&mut renderer, Anchor::Center),
        GREEN,
        8
    ));
    assert!(testkit::close(shown(&mut renderer, Anchor::Right), BLUE, 8));
    assert!(testkit::close(
        shown(&mut renderer, Anchor::BottomRight),
        BLUE,
        8
    ));
    let contained = testkit::render(
        &mut renderer,
        &image("wide.png", Fit::Contain, Anchor::Left, Playback::default()),
        &scene,
    );
    assert!(
        testkit::close(testkit::px(&contained, 5, 5), GREEN, 8),
        "the anchor only matters when the picture covers"
    );
}

#[test]
fn a_gif_plays_at_its_speed_and_once_stays_on_its_last_frame() {
    let gif = testkit::gif(4, 4, &[(RED, 100), (GREEN, 100)]);
    let mut renderer = testkit::renderer();
    let at = |renderer: &mut crate::SkiaRenderer, playback, ms| {
        let mut scene = Scene::empty().asset("a.gif", gif.clone());
        scene.animation = Duration::from_millis(ms);
        let frame = testkit::render(
            renderer,
            &image("a.gif", Fit::Fill, Anchor::Center, playback),
            &scene,
        );
        testkit::px(&frame, 5, 5)
    };
    let normal = Playback::default();
    assert!(testkit::close(at(&mut renderer, normal, 60), RED, 8));
    let fast = Playback {
        speed: 2.0,
        ..normal
    };
    assert!(
        testkit::close(at(&mut renderer, fast, 60), GREEN, 8),
        "120 ms in"
    );
    let slow = Playback {
        speed: 0.5,
        ..normal
    };
    assert!(
        testkit::close(at(&mut renderer, slow, 150), RED, 8),
        "75 ms in"
    );
    assert!(
        testkit::close(at(&mut renderer, normal, 450), RED, 8),
        "looping"
    );
    let once = Playback {
        once: true,
        ..normal
    };
    assert!(
        testkit::close(at(&mut renderer, once, 450), GREEN, 8),
        "the last frame stays"
    );
    assert!(testkit::close(at(&mut renderer, once, 50), RED, 8));
}
