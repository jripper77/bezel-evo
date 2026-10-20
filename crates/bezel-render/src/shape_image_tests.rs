//! A shape's picture: each repeat mode, clipping to the shape, opacity over
//! the fill, offsets, SVG tiles and animated GIFs inside a shape.

use std::time::Duration;

use bezel_core::domain::frame::Rgba;
use bezel_core::domain::theme::{
    AssetRef, Background, BoxF, ElementKind, ImageRepeat, Paint, ShapeImage, ShapeKind,
};

use crate::testkit::{self, Scene, assert_px};

const RED: Rgba = Rgba::opaque(255, 0, 0);
const GREEN: Rgba = Rgba::opaque(0, 255, 0);
const BLUE: Rgba = Rgba::opaque(0, 0, 255);

/// A 40×40 canvas on black with one blue shape covering it, holding `image`.
fn frame(shape: ShapeKind, image: ShapeImage, scene: &Scene) -> bezel_core::domain::frame::Frame {
    let element = testkit::element(
        BoxF::new(0.0, 0.0, 40.0, 40.0),
        ElementKind::Shape {
            video_window: false,
            fade: None,
            shape,
            fill: Some(Paint::solid(BLUE)),
            image: Some(image),
            stroke: None,
        },
    );
    let theme = testkit::theme(40, 40, Background::Color(Rgba::BLACK), vec![element]);
    testkit::render(&mut testkit::renderer(), &theme, scene)
}

fn picture(repeat: ImageRepeat) -> ShapeImage {
    ShapeImage {
        repeat,
        ..ShapeImage::new(AssetRef("p.png".into()))
    }
}

/// A 4×4 picture: red left half, green right half.
fn halves() -> Scene {
    Scene::empty().asset(
        "p.png",
        testkit::png(4, 4, |x, _| if x < 2 { RED } else { GREEN }),
    )
}

const SQUARE: ShapeKind = ShapeKind::Rect { radius: 0.0 };

#[test]
fn tiles_repeat_the_picture_at_its_own_size_across_the_shape() {
    let f = frame(SQUARE, picture(ImageRepeat::Tile), &halves());
    for (x, color) in [
        (1, RED),
        (2, GREEN),
        (5, RED),
        (6, GREEN),
        (37, RED),
        (38, GREEN),
    ] {
        assert_px(&f, x, 1, color, 0);
        assert_px(&f, x, 37, color, 0);
    }
}

#[test]
fn a_scaled_and_offset_tile_moves_its_copies() {
    let image = ShapeImage {
        scale: 2.0,
        offset: (3.0, 0.0),
        ..picture(ImageRepeat::Tile)
    };
    let f = frame(SQUARE, image, &halves());
    // 8-pixel copies starting at x = 3: red 3..7, green 7..11, and so on.
    assert_px(&f, 4, 5, RED, 0);
    assert_px(&f, 8, 5, GREEN, 0);
    assert_px(&f, 12, 5, RED, 0);
    assert_px(&f, 1, 5, GREEN, 0);
}

#[test]
fn a_row_tile_leaves_the_fill_above_and_below_it() {
    let image = ShapeImage {
        offset: (0.0, 10.0),
        ..picture(ImageRepeat::TileX)
    };
    let f = frame(SQUARE, image, &halves());
    assert_px(&f, 1, 12, RED, 0);
    assert_px(&f, 30, 12, GREEN, 0);
    assert_px(&f, 20, 5, BLUE, 0);
    assert_px(&f, 20, 20, BLUE, 0);
}

#[test]
fn a_column_tile_leaves_the_fill_beside_it() {
    let f = frame(SQUARE, picture(ImageRepeat::TileY), &halves());
    assert_px(&f, 1, 30, RED, 0);
    assert_px(&f, 3, 30, GREEN, 0);
    assert_px(&f, 10, 30, BLUE, 0);
}

#[test]
fn a_centred_copy_keeps_its_scaled_size_and_follows_the_offset() {
    let image = ShapeImage {
        scale: 2.0,
        offset: (5.0, -2.0),
        ..picture(ImageRepeat::Center)
    };
    let f = frame(SQUARE, image, &halves());
    // 8×8 copy centred at 16..24, moved to 21..29 across and 14..22 down.
    assert_px(&f, 22, 15, RED, 0);
    assert_px(&f, 27, 20, GREEN, 0);
    assert_px(&f, 18, 18, BLUE, 0);
    assert_px(&f, 25, 25, BLUE, 0);
}

#[test]
fn contain_shows_the_fill_where_the_picture_does_not_reach() {
    let scene = Scene::empty().asset("p.png", testkit::png(4, 2, |_, _| RED));
    let f = frame(SQUARE, picture(ImageRepeat::Contain), &scene);
    assert_px(&f, 20, 20, RED, 0);
    assert_px(&f, 20, 4, BLUE, 0);
    assert_px(&f, 20, 36, BLUE, 0);
}

#[test]
fn cover_and_stretch_fill_the_shape() {
    let scene = Scene::empty().asset(
        "p.png",
        testkit::png(4, 2, |x, _| if x < 2 { RED } else { GREEN }),
    );
    let cover = frame(SQUARE, picture(ImageRepeat::Cover), &scene);
    assert_px(&cover, 2, 2, RED, 0);
    assert_px(&cover, 37, 37, GREEN, 0);
    let stretch = frame(SQUARE, picture(ImageRepeat::Stretch), &scene);
    assert_px(&stretch, 2, 38, RED, 0);
    assert_px(&stretch, 38, 2, GREEN, 0);
}

#[test]
fn the_picture_is_clipped_to_the_shape() {
    let scene = Scene::empty().asset("p.png", testkit::png(2, 2, |_, _| RED));
    let f = frame(ShapeKind::Ellipse, picture(ImageRepeat::Stretch), &scene);
    assert_px(&f, 20, 20, RED, 0);
    assert_px(&f, 1, 1, Rgba::BLACK, 0);
    assert_px(&f, 38, 38, Rgba::BLACK, 0);
}

#[test]
fn opacity_blends_the_picture_over_the_fill() {
    let scene = Scene::empty().asset("p.png", testkit::png(2, 2, |_, _| Rgba::WHITE));
    let image = ShapeImage {
        opacity: 0.5,
        ..picture(ImageRepeat::Cover)
    };
    let f = frame(SQUARE, image, &scene);
    assert_px(&f, 20, 20, Rgba::opaque(128, 128, 255), 2);
    let hidden = ShapeImage {
        opacity: 0.0,
        ..picture(ImageRepeat::Cover)
    };
    assert_px(&frame(SQUARE, hidden, &scene), 20, 20, BLUE, 0);
}

#[test]
fn svg_tiles_are_drawn_at_their_scaled_size() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="5" height="10" fill="#ff0000"/><rect x="5" width="5" height="10" fill="#00ff00"/></svg>"##;
    let scene = Scene::empty().asset("p.svg", svg.to_vec());
    let image = ShapeImage {
        asset: AssetRef("p.svg".into()),
        scale: 0.8,
        ..picture(ImageRepeat::Tile)
    };
    let f = frame(SQUARE, image, &scene);
    // 8-pixel copies: red 0..4, green 4..8, red 8..12.
    assert_px(&f, 1, 1, RED, 8);
    assert_px(&f, 6, 1, GREEN, 8);
    assert_px(&f, 9, 1, RED, 8);
}

#[test]
fn a_missing_or_unchosen_picture_leaves_the_fill() {
    assert_px(
        &frame(SQUARE, picture(ImageRepeat::Tile), &Scene::empty()),
        20,
        20,
        BLUE,
        0,
    );
    let unchosen = ShapeImage::new(AssetRef(String::new()));
    assert_px(&frame(SQUARE, unchosen, &halves()), 20, 20, BLUE, 0);
}

#[test]
fn an_animated_gif_in_a_shape_plays() {
    let gif = testkit::gif(2, 2, &[(Rgba::WHITE, 100), (Rgba::BLACK, 100)]);
    let image = ShapeImage {
        asset: AssetRef("a.gif".into()),
        ..picture(ImageRepeat::Cover)
    };
    let element = testkit::element(
        BoxF::new(0.0, 0.0, 40.0, 40.0),
        ElementKind::Shape {
            video_window: false,
            fade: None,
            shape: SQUARE,
            fill: Some(Paint::solid(BLUE)),
            image: Some(image),
            stroke: None,
        },
    );
    let theme = testkit::theme(40, 40, Background::Color(Rgba::BLACK), vec![element]);
    let mut renderer = testkit::renderer();
    let mut scene = Scene::empty().asset("a.gif", gif);
    // The same renderer: a cached layer would keep the first frame.
    assert_px(
        &testkit::render(&mut renderer, &theme, &scene),
        20,
        20,
        Rgba::WHITE,
        0,
    );
    scene.animation = Duration::from_millis(150);
    assert_px(
        &testkit::render(&mut renderer, &theme, &scene),
        20,
        20,
        Rgba::BLACK,
        0,
    );
}
