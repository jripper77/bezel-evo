//! Media player drawing: the parts it shows, its progress bar and its natural
//! size. Themes saved before the optional parts existed must draw exactly as
//! they did, so their frames are pinned by hash.

use bezel_core::domain::frame::{Frame, Rgba};
use bezel_core::domain::playback::{MediaSession, Player};
use bezel_core::domain::theme::{
    Background, BoxF, ElementId, ElementKind, FontSpec, HAlign, Paint, TextContent, TextStyle,
    Theme, VAlign,
};
use bezel_core::ports::{Backdrop, FrameRenderer, RenderContext};

use crate::SkiaRenderer;
use crate::testkit::{self, FAMILY, Scene, element, png};

const RED: Rgba = Rgba::opaque(255, 0, 0);

/// FNV-1a over the frame bytes.
fn hash(frame: &Frame) -> u64 {
    frame.as_rgba().iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn style(align: HAlign, valign: VAlign) -> TextStyle {
    TextStyle {
        font: FontSpec {
            family: FAMILY.into(),
            weight: 400,
            italic: false,
            asset: None,
        },
        size: 14.,
        paint: Paint::solid(Rgba::WHITE),
        align,
        valign,
        casing: Default::default(),
        letter_spacing: 0.,
        ..TextStyle::default()
    }
}

/// A player with only the fields themes had before the optional parts.
fn legacy(cover: bool, progress: bool, source: bool) -> Player {
    Player {
        source: String::new(),
        show_cover: cover,
        cover_radius: 0.,
        cover_corners: None,
        cover_gap: None,
        show_progress: progress,
        show_source: source,
        hide_when_stopped: false,
        empty_text: "No media".into(),
        ..Default::default()
    }
}

fn player_theme(player: Player, style: TextStyle) -> Theme {
    testkit::theme(
        240,
        120,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 8., 220., 104.),
            ElementKind::Text {
                content: TextContent::Player(player),
                style,
            },
        )],
    )
}

fn playing() -> Scene {
    let mut scene = Scene::empty();
    scene.snapshot.media.push(MediaSession {
        source: "Spotify.exe".into(),
        title: "Track".into(),
        artist: "Artist".into(),
        playing: true,
        position: 84.,
        duration: 210.,
        cover: png(20, 20, |_, _| RED),
    });
    scene
}

fn measure(r: &mut SkiaRenderer, theme: &Theme, scene: &Scene) -> Option<BoxF> {
    let context = RenderContext {
        snapshot: &scene.snapshot,
        histories: &scene.histories,
        quantities: &bezel_core::domain::sensor::Quantities::new(),
        time: scene.time,
        animation: scene.animation,
        language: bezel_core::domain::clock::Language::English,
        backdrop: Backdrop::Poster,
        shown: &scene.shown,
    };
    r.measure_element(theme, ElementId(1), &scene.assets, context)
        .expect("measure")
}

/// The legacy configurations, each with the hash of its frame and its fit.
fn legacy_cases() -> Vec<(&'static str, Theme, Scene)> {
    let top = style(HAlign::Left, VAlign::Top);
    let mut corners = legacy(true, false, false);
    corners.cover_corners = Some([0., 6., 12., 18.]);
    corners.cover_gap = Some(16.);
    let mut radius = legacy(false, true, true);
    radius.cover_radius = 9.;
    let mut empty = Scene::empty();
    empty.snapshot.media.clear();
    let mut stopped = playing();
    stopped.snapshot.media[0].playing = false;
    stopped.snapshot.media[0].duration = 0.;
    vec![
        (
            "cover+progress",
            player_theme(legacy(true, true, false), top.clone()),
            playing(),
        ),
        (
            "progress centred",
            player_theme(
                legacy(false, true, false),
                style(HAlign::Center, VAlign::Middle),
            ),
            playing(),
        ),
        (
            "source+progress",
            player_theme(radius, top.clone()),
            playing(),
        ),
        (
            "cover corners",
            player_theme(corners, style(HAlign::Right, VAlign::Bottom)),
            playing(),
        ),
        (
            "no session",
            player_theme(legacy(true, true, false), top.clone()),
            empty,
        ),
        (
            "stopped",
            player_theme(legacy(true, true, true), top),
            stopped,
        ),
    ]
}

/// Frame hashes and fits recorded before the optional parts existed.
const LEGACY: [(u64, (f32, f32, f32, f32)); 6] = [
    (0x58f9_f8c3_eb45_c748, (10., 8., 157., 59.)),
    (0xff7b_d224_a8d6_001d, (73.5, 30.5, 93., 59.)),
    (0x49d8_f425_2473_251a, (10., 8., 93., 76.)),
    (0x1b38_c185_9eff_9543, (129., 78., 101., 34.)),
    (0x124b_9a37_8e92_76a7, (10., 8., 68., 17.)),
    (0x00e0_bb17_4da3_93c1, (10., 8., 149., 51.)),
];

#[test]
fn legacy_players_draw_and_fit_exactly_as_before() {
    let mut r = testkit::renderer();
    for ((name, theme, scene), (expected, (x, y, w, h))) in legacy_cases().into_iter().zip(LEGACY) {
        let frame = testkit::render(&mut r, &theme, &scene);
        assert_eq!(hash(&frame), expected, "{name} pixels");
        assert_eq!(
            measure(&mut r, &theme, &scene),
            Some(BoxF::new(x, y, w, h)),
            "{name} fit"
        );
    }
}

const BLUE: Rgba = Rgba::opaque(0, 0, 255);

/// A player showing only the parts set by `edit`, with nothing on by default.
fn parts(edit: impl FnOnce(&mut Player)) -> Player {
    let mut p = Player {
        show_cover: false,
        show_progress: false,
        show_title: false,
        show_artist: false,
        show_time: Some(false),
        empty_text: "No media".into(),
        ..Player::default()
    };
    edit(&mut p);
    p
}

/// A bar alone, red on blue, 6 px thick.
fn bar(edit: impl FnOnce(&mut Player)) -> Player {
    parts(|p| {
        p.show_progress = true;
        p.progress.fill = Some(RED);
        p.progress.track = BLUE;
        p.progress.thickness = 6.;
        edit(p);
    })
}

fn draw(player: Player, valign: VAlign) -> Frame {
    let theme = player_theme(player, style(HAlign::Left, valign));
    testkit::render(&mut testkit::renderer(), &theme, &playing())
}

fn fit(player: Player, valign: VAlign) -> Option<BoxF> {
    let theme = player_theme(player, style(HAlign::Left, valign));
    let mut r = testkit::renderer();
    let b = measure(&mut r, &theme, &playing());
    if let Some(b) = b {
        let mut fitted = theme.clone();
        fitted.elements[0].frame = b;
        assert_eq!(
            measure(&mut r, &fitted, &playing()),
            Some(b),
            "fit is stable"
        );
    }
    b
}

fn static_text(text: &str) -> Theme {
    testkit::theme(
        240,
        120,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 8., 220., 104.),
            ElementKind::Text {
                content: TextContent::Static(text.into()),
                style: style(HAlign::Left, VAlign::Top),
            },
        )],
    )
}

/// The x of the first and last pixels of `row` matching `pred`.
fn span(frame: &Frame, row: u32, pred: impl Fn(Rgba) -> bool) -> Option<(u32, u32)> {
    let xs: Vec<u32> = (0..240)
        .filter(|&x| pred(testkit::px(frame, x, row)))
        .collect();
    Some((*xs.first()?, *xs.last()?))
}

/// Separate runs of pixels of `row` matching `pred`.
fn runs(frame: &Frame, row: u32, pred: impl Fn(Rgba) -> bool) -> usize {
    let on: Vec<bool> = (0..240).map(|x| pred(testkit::px(frame, x, row))).collect();
    on.windows(2).filter(|w| w[1] && !w[0]).count() + usize::from(on[0])
}

#[test]
fn each_text_part_draws_alone_like_the_same_static_text() {
    let mut r = testkit::renderer();
    let cases = [
        (parts(|p| p.show_title = true), "Track"),
        (parts(|p| p.show_artist = true), "Artist"),
        (parts(|p| p.show_time = Some(true)), "1:24 / 3:30"),
        (parts(|p| p.show_source = true), "Spotify.exe"),
        (
            parts(|p| {
                p.show_artist = true;
                p.show_time = Some(true);
                p.time_format = bezel_core::domain::playback::TimeFormat::Remaining;
            }),
            "Artist\n-2:06",
        ),
    ];
    for (player, text) in cases {
        let theme = player_theme(player, style(HAlign::Left, VAlign::Top));
        let drawn = testkit::render(&mut r, &theme, &playing());
        let expected = testkit::render(&mut r, &static_text(text), &playing());
        assert_eq!(hash(&drawn), hash(&expected), "{text:?}");
        assert_eq!(
            measure(&mut r, &theme, &playing()),
            measure(&mut r, &static_text(text), &playing()),
            "{text:?} fit"
        );
    }
}

#[test]
fn a_bar_alone_sits_by_the_vertical_alignment_with_no_space_kept() {
    use testkit::{assert_px, px};
    let frame = draw(bar(|_| {}), VAlign::Middle);
    // 104 - 6 = 98, half above: rows 57..63; 40% of 220 filled.
    assert_px(&frame, 20, 60, RED, 0);
    assert_px(&frame, 150, 60, BLUE, 0);
    assert_eq!(px(&frame, 20, 55), Rgba::BLACK);
    assert_eq!(px(&frame, 20, 64), Rgba::BLACK);
    let top = draw(bar(|_| {}), VAlign::Top);
    assert_px(&top, 20, 10, RED, 0);
    let bottom = draw(bar(|_| {}), VAlign::Bottom);
    assert_px(&bottom, 20, 109, RED, 0);
    let text_colour = draw(bar(|p| p.progress.fill = None), VAlign::Middle);
    assert_px(&text_colour, 20, 60, Rgba::WHITE, 0);
}

#[test]
fn bar_ends_knob_and_patterns() {
    use bezel_core::domain::playback::{ProgressEnds, ProgressPattern};
    use testkit::{assert_px, px};
    let round = draw(bar(|_| {}), VAlign::Middle);
    let square = draw(
        bar(|p| p.progress.ends = ProgressEnds::Square),
        VAlign::Middle,
    );
    assert_px(&square, 10, 57, RED, 0);
    assert_px(&square, 229, 62, BLUE, 0);
    assert!(px(&round, 10, 57).r < 200, "round ends cut the corner");
    assert!(px(&round, 229, 62).b < 200);

    assert_eq!(px(&round, 98, 54), Rgba::BLACK);
    let knob = draw(bar(|p| p.progress.knob = true), VAlign::Middle);
    assert_px(&knob, 98, 55, RED, 0);
    assert_px(&knob, 98, 64, RED, 0);
    assert_eq!(px(&knob, 98, 50), Rgba::BLACK);

    let segments = draw(
        bar(|p| {
            p.progress.pattern = ProgressPattern::Segments;
            p.progress.segments = 4;
        }),
        VAlign::Middle,
    );
    let lit = |c: Rgba| c != Rgba::BLACK;
    assert_eq!(runs(&segments, 60, lit), 4);
    assert_eq!(px(&segments, 63, 60), Rgba::BLACK, "a gap between blocks");
    let dots = draw(
        bar(|p| p.progress.pattern = ProgressPattern::Dots),
        VAlign::Middle,
    );
    assert_eq!(runs(&dots, 60, lit), 23);
    assert!(runs(&dots, 60, |c| c.r > 200) > 5, "filled dots");
    assert!(runs(&dots, 60, |c| c.b > 200) > 5, "empty dots");
}

#[test]
fn times_beside_or_above_the_bar() {
    use bezel_core::domain::playback::{TimeFormat, TimePlacement};
    let red = |c: Rgba| c.r > 200 && c.g < 60;
    let blue = |c: Rgba| c.b > 200 && c.r < 60;
    let white = |c: Rgba| c.r > 120 && c.g > 120 && c.b > 120;
    let plain = draw(bar(|_| {}), VAlign::Middle);
    assert_eq!(span(&plain, 60, red).map(|s| s.0), Some(10));
    assert_eq!(span(&plain, 60, blue).map(|s| s.1), Some(229));

    let sides = draw(
        bar(|p| {
            p.show_time = Some(true);
            p.progress.times = TimePlacement::Sides;
        }),
        VAlign::Middle,
    );
    let (bar_start, _) = span(&sides, 60, red).expect("filled part");
    let (_, bar_end) = span(&sides, 60, blue).expect("track");
    assert!(bar_start > 40 && bar_end < 200, "{bar_start}..{bar_end}");
    assert!(testkit::count(&sides, (10, 40, bar_start, 80), white) > 10);
    assert!(testkit::count(&sides, (bar_end + 1, 40, 230, 80), white) > 10);

    let elapsed = draw(
        bar(|p| {
            p.show_time = Some(true);
            p.time_format = TimeFormat::Elapsed;
            p.progress.times = TimePlacement::Sides;
        }),
        VAlign::Middle,
    );
    assert_eq!(
        span(&elapsed, 60, blue).map(|s| s.1),
        Some(229),
        "no right time"
    );

    let above = draw(
        bar(|p| {
            p.show_time = Some(true);
            p.progress.times = TimePlacement::Above;
        }),
        VAlign::Top,
    );
    // Times on a 17 px line, 2 px, then the bar at rows 27..33.
    assert_eq!(span(&above, 30, red).map(|s| s.0), Some(10));
    assert_eq!(span(&above, 30, blue).map(|s| s.1), Some(229));
    assert!(testkit::count(&above, (10, 8, 60, 25), white) > 10);
    assert!(testkit::count(&above, (180, 8, 230, 25), white) > 10);
    assert_eq!(testkit::count(&above, (0, 34, 240, 120), white), 0);
}

#[test]
fn fit_measures_only_the_visible_parts() {
    use bezel_core::domain::playback::TimePlacement;
    // One line of 14 px text is 17 px tall.
    let title = fit(parts(|p| p.show_title = true), VAlign::Top).expect("title");
    assert_eq!(title.height, 17.);
    let two = fit(
        parts(|p| {
            p.show_title = true;
            p.show_artist = true;
        }),
        VAlign::Top,
    )
    .expect("two lines");
    assert_eq!(two.height, 34.);
    assert_eq!(
        fit(bar(|_| {}), VAlign::Top),
        Some(BoxF::new(10., 8., 220., 6.)),
        "a bar alone keeps its width"
    );
    assert_eq!(
        fit(bar(|p| p.progress.knob = true), VAlign::Top).map(|b| b.height),
        Some(12.)
    );
    let titled_bar = fit(bar(|p| p.show_title = true), VAlign::Top).expect("title and bar");
    assert_eq!(titled_bar.height, 17. + 4. + 6.);
    let sides = fit(
        bar(|p| {
            p.show_time = Some(true);
            p.progress.times = TimePlacement::Sides;
        }),
        VAlign::Top,
    )
    .expect("sides");
    assert_eq!(sides.height, 17.);
    assert!(sides.width > 2. * 14. * 3.);
    let above = fit(
        bar(|p| {
            p.show_title = true;
            p.show_time = Some(true);
            p.progress.times = TimePlacement::Above;
        }),
        VAlign::Top,
    )
    .expect("above");
    assert_eq!(above.height, 17. + 4. + 17. + 2. + 6.);
    assert_eq!(
        fit(parts(|p| p.show_cover = true), VAlign::Top),
        Some(BoxF::new(10., 8., 104., 104.)),
        "a lone cover is square"
    );
    assert_eq!(
        fit(bar(|p| p.show_cover = true), VAlign::Top),
        Some(BoxF::new(10., 8., 220., 104.))
    );
    assert_eq!(fit(parts(|_| {}), VAlign::Top), None, "nothing to fit");
}

#[test]
fn a_lone_cover_fills_its_box() {
    let frame = draw(parts(|p| p.show_cover = true), VAlign::Top);
    // Contained in a 104 px square at the left.
    testkit::assert_px(&frame, 100, 60, RED, 0);
    assert_eq!(testkit::px(&frame, 120, 60), Rgba::BLACK);
}
