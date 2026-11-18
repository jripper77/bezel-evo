use crate::testkit::{self, Scene, element};
use bezel_core::domain::{
    agent::{AgentQuota, AgentSession, AgentWidget},
    frame::Rgba,
    theme::{Background, BoxF, ElementKind, FontSpec, Paint, TextContent, TextStyle, VAlign},
};

#[test]
fn compact_status_uses_green_for_working_and_red_for_completion_or_attention() {
    use bezel_core::domain::agent::AgentState;
    let widget = AgentWidget {
        status_only: true,
        compact_quotas: true,
        ..Default::default()
    };
    let theme = testkit::theme(
        360,
        240,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 220.),
            ElementKind::Text {
                content: TextContent::Agent(widget),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents.push(AgentSession {
        id: "session".into(),
        project: "bezel".into(),
        state: AgentState::Working,
        ..Default::default()
    });
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme, &scene);
    let dot = testkit::px(&frame, 14, 17);
    assert!(dot.g > dot.r * 2);
    for state in [
        AgentState::Completed,
        AgentState::Waiting,
        AgentState::Error,
    ] {
        scene.snapshot.agents[0].state = state;
        let frame = testkit::render(&mut renderer, &theme, &scene);
        let dot = testkit::px(&frame, 14, 17);
        assert!(dot.r > dot.g * 2);
    }
}

#[test]
fn agent_quota_bars_update_and_expired_quotas_empty_the_bar() {
    let widget = AgentWidget {
        show_weekly: false,
        show_reset: false,
        ..Default::default()
    };
    let theme = testkit::theme(
        360,
        240,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 220.),
            ElementKind::Text {
                content: TextContent::Agent(widget),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    scene.snapshot.agents.push(AgentSession {
        id: "session".into(),
        project: "bezel".into(),
        message: "Checking tests".into(),
        five_hour: Some(AgentQuota {
            used_percent: 50.,
            resets_at: 1000,
        }),
        ..Default::default()
    });
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert!(testkit::px(&frame, 30, 226).r > 200);
    assert!(testkit::px(&frame, 300, 226).r < 150);
    scene.snapshot.agents[0]
        .five_hour
        .as_mut()
        .unwrap()
        .used_percent = 90.;
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert!(testkit::px(&frame, 300, 226).r > 200);
    scene.snapshot.agents_sampled_at = 1000;
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert!(testkit::px(&frame, 30, 226).r < 150);
}

#[test]
fn automatic_projects_rotate_pages_and_keep_account_usage_when_projects_are_hidden() {
    use bezel_core::{domain::agent::AgentState, ports::FrameRenderer};
    use std::time::Duration;
    let widget = AgentWidget {
        project_list: true,
        show_five_hour: false,
        show_weekly: false,
        ..Default::default()
    };
    let mut theme = testkit::theme(
        360,
        140,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 120.),
            ElementKind::Text {
                content: TextContent::Agent(widget),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    for i in 0..5 {
        scene.snapshot.agents.push(AgentSession {
            id: format!("session-{i}"),
            project: format!("project-{i}"),
            project_path: format!("F:/dev/project-{i}"),
            state: AgentState::Working,
            updated_at: 100 - i,
            ..Default::default()
        });
    }
    let mut renderer = testkit::renderer();
    let first = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(renderer.next_change(), Some(Duration::from_secs(10)));
    assert!(testkit::px(&first, 20, 41).g > testkit::px(&first, 20, 41).r * 2);
    scene.animation = Duration::from_secs(10);
    let second = testkit::render(&mut renderer, &theme, &scene);
    assert_ne!(first.as_rgba(), second.as_rgba());
    scene.animation = Duration::from_secs(20);
    assert_eq!(
        first.as_rgba(),
        testkit::render(&mut renderer, &theme, &scene).as_rgba()
    );
    if let ElementKind::Text {
        content: TextContent::Agent(a),
        ..
    } = &mut theme.elements[0].kind
    {
        a.excluded_projects = vec!["F:/dev".into()];
        a.show_five_hour = true;
        a.compact_quotas = true;
    }
    scene.snapshot.agents[0].five_hour = Some(AgentQuota {
        used_percent: 25.,
        resets_at: 1000,
    });
    scene.snapshot.agents[0].quota_updated_at = 100;
    let hidden = testkit::render(&mut renderer, &theme, &scene);
    assert!(testkit::px(&hidden, 30, 126).r > 200);
    assert!(renderer.next_change().is_none());
}

#[test]
fn pixel_quota_segments_warn_at_twenty_percent_and_ignore_expired_data() {
    let mut theme = testkit::theme(
        360,
        240,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 220.),
            ElementKind::Text {
                content: TextContent::Agent(AgentWidget {
                    pixel_style: true,
                    compact_quotas: true,
                    show_weekly: false,
                    ..Default::default()
                }),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    scene.snapshot.agents.push(AgentSession {
        five_hour: Some(AgentQuota {
            used_percent: 79.9,
            resets_at: 1000,
        }),
        ..Default::default()
    });
    let mut renderer = testkit::renderer();
    let normal = testkit::render(&mut renderer, &theme, &scene);
    let orange = testkit::px(&normal, 12, 226);
    assert!(orange.r > 240 && orange.g > 130 && orange.g < 160);
    assert_eq!(
        testkit::px(&normal, 18, 226),
        Rgba::BLACK,
        "pixel gaps stay unpainted"
    );
    scene.snapshot.agents[0]
        .five_hour
        .as_mut()
        .unwrap()
        .used_percent = 80.;
    let warning = testkit::render(&mut renderer, &theme, &scene);
    let red = testkit::px(&warning, 12, 226);
    assert!(red.r > red.g * 2);
    scene.snapshot.agents_sampled_at = 1000;
    let expired = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(testkit::px(&expired, 12, 226), Rgba::opaque(34, 34, 32));
    // The ordinary bar also warns without requiring the Pixel option.
    if let ElementKind::Text {
        content: TextContent::Agent(a),
        ..
    } = &mut theme.elements[0].kind
    {
        a.pixel_style = false;
    }
    scene.snapshot.agents_sampled_at = 100;
    let ordinary = testkit::render(&mut renderer, &theme, &scene);
    let red = testkit::px(&ordinary, 12, 226);
    assert!(red.r > red.g * 2);
}

#[test]
fn changed_status_softly_highlights_only_its_agent_then_stops() {
    use bezel_core::{
        domain::agent::{AgentKind, AgentState},
        ports::FrameRenderer,
    };
    use std::time::Duration;
    let mut elements = Vec::new();
    for (index, agent) in [AgentKind::Codex, AgentKind::Claude]
        .into_iter()
        .enumerate()
    {
        let mut e = element(
            BoxF::new(10. + index as f32 * 180., 10., 160., 180.),
            ElementKind::Text {
                content: TextContent::Agent(AgentWidget {
                    agent,
                    blink_on_change: true,
                    show_five_hour: false,
                    show_weekly: false,
                    ..Default::default()
                }),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        );
        e.id = bezel_core::domain::theme::ElementId(index as u32 + 1);
        elements.push(e);
    }
    let theme = testkit::theme(360, 200, Background::Color(Rgba::BLACK), elements);
    let mut scene = Scene::empty();
    for agent in [AgentKind::Codex, AgentKind::Claude] {
        scene.snapshot.agents.push(AgentSession {
            agent,
            id: "session".into(),
            state: AgentState::Working,
            ..Default::default()
        });
    }
    let mut renderer = testkit::renderer();
    let first = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(testkit::px(&first, 12, 11), Rgba::BLACK);
    scene.animation = Duration::from_secs(1);
    scene.snapshot.agents[0].state = AgentState::Completed;
    testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(renderer.next_change(), Some(Duration::from_millis(1033)));
    scene.animation = Duration::from_secs(2);
    let pulse = testkit::render(&mut renderer, &theme, &scene);
    assert!(testkit::px(&pulse, 11, 48).r > 100);
    assert_eq!(testkit::px(&pulse, 191, 48), Rgba::BLACK);
    scene.animation = Duration::from_secs(3);
    let done = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(testkit::px(&done, 11, 48), Rgba::BLACK);
    assert!(renderer.next_change().is_none());
}

#[test]
fn project_list_pulse_stays_on_the_changed_row_after_sorting() {
    use bezel_core::domain::agent::AgentState;
    use std::time::Duration;
    let widget = AgentWidget {
        project_list: true,
        blink_on_change: true,
        show_five_hour: false,
        show_weekly: false,
        ..Default::default()
    };
    let theme = testkit::theme(
        360,
        140,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 120.),
            ElementKind::Text {
                content: TextContent::Agent(widget),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    for i in 0..2 {
        scene.snapshot.agents.push(AgentSession {
            id: format!("s{i}"),
            project: format!("p{i}"),
            project_path: format!("F:/dev/p{i}"),
            state: AgentState::Working,
            updated_at: 100 - i,
            ..Default::default()
        });
    }
    let mut renderer = testkit::renderer();
    testkit::render(&mut renderer, &theme, &scene);
    scene.snapshot.agents[0].state = AgentState::Completed;
    scene.animation = Duration::from_secs(1);
    testkit::render(&mut renderer, &theme, &scene);
    scene.animation = Duration::from_secs(2);
    let pulse = testkit::render(&mut renderer, &theme, &scene);
    assert!(testkit::px(&pulse, 11, 62).r > 100);
    assert_eq!(testkit::px(&pulse, 11, 40), Rgba::BLACK);
    assert_eq!(testkit::px(&pulse, 11, 11), Rgba::BLACK);
}

#[test]
fn custom_palette_threshold_and_hidden_header_reach_the_native_frame() {
    use bezel_core::domain::agent::{AgentColors, AgentState};
    let blue = Rgba::opaque(40, 90, 250);
    let violet = Rgba::opaque(180, 60, 220);
    let mut theme = testkit::theme(
        360,
        240,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 220.),
            ElementKind::Text {
                content: TextContent::Agent(AgentWidget {
                    project_list: true,
                    compact_quotas: true,
                    show_weekly: false,
                    warning_percent: 35,
                    colors: AgentColors {
                        completed: Some(blue),
                        warning: Some(violet),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    scene.snapshot.agents.push(AgentSession {
        id: "done".into(),
        project: "bezel".into(),
        project_path: "F:/dev/bezel".into(),
        state: AgentState::Completed,
        updated_at: 90,
        quota_updated_at: 100,
        five_hour: Some(AgentQuota {
            used_percent: 70.,
            resets_at: 1000,
        }),
        ..Default::default()
    });
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(testkit::px(&frame, 20, 41), blue, "row below the header");
    assert_eq!(
        testkit::px(&frame, 12, 226),
        violet,
        "30% left is under 35%"
    );
    if let ElementKind::Text {
        content: TextContent::Agent(a),
        ..
    } = &mut theme.elements[0].kind
    {
        a.show_header = false;
        a.warning_percent = 0;
        a.colors.quota = Some(blue);
    }
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert_eq!(testkit::px(&frame, 20, 17), blue, "first row moves up");
    assert_eq!(testkit::px(&frame, 12, 226), blue, "warning disabled");
}

#[test]
fn agent_parts_move_rows_and_hide_the_dot() {
    use bezel_core::domain::{
        agent::AgentState,
        theme::{PartStyle, TextPart},
    };
    let green = |px: Rgba| px.g > 150 && u16::from(px.g) > u16::from(px.r) * 2;
    let mut style = TextStyle {
        font: FontSpec {
            family: testkit::FAMILY.into(),
            ..Default::default()
        },
        size: 14.,
        paint: Paint::solid(Rgba::WHITE),
        valign: VAlign::Top,
        ..Default::default()
    };
    style.parts.styles.insert(
        TextPart::AgentTitle,
        PartStyle {
            size_percent: Some(200.),
            ..PartStyle::default()
        },
    );
    let mut theme = testkit::theme(
        360,
        240,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 220.),
            ElementKind::Text {
                content: TextContent::Agent(AgentWidget {
                    project_list: true,
                    show_five_hour: false,
                    show_weekly: false,
                    ..Default::default()
                }),
                style,
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    scene.snapshot.agents.push(AgentSession {
        id: "one".into(),
        project: "bezel".into(),
        project_path: "F:/dev/bezel".into(),
        state: AgentState::Working,
        updated_at: 100,
        ..Default::default()
    });
    let mut renderer = testkit::renderer();
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert!(
        green(testkit::px(&frame, 20, 63)),
        "a taller title pushes the row down"
    );
    assert_eq!(testkit::px(&frame, 20, 41), Rgba::BLACK);
    if let ElementKind::Text {
        content: TextContent::Agent(a),
        ..
    } = &mut theme.elements[0].kind
    {
        a.show_dot = false;
    }
    let frame = testkit::render(&mut renderer, &theme, &scene);
    assert!(!green(testkit::px(&frame, 20, 63)), "no dot");
}

#[test]
fn a_part_placed_by_hand_moves_and_the_others_keep_their_places() {
    use bezel_core::domain::agent::{AgentLayout, AgentState, PartRect};
    let green = |px: Rgba| px.g > 150 && u16::from(px.g) > u16::from(px.r) * 2;
    let mut theme = testkit::theme(
        360,
        240,
        Background::Color(Rgba::BLACK),
        vec![element(
            BoxF::new(10., 10., 340., 220.),
            ElementKind::Text {
                content: TextContent::Agent(AgentWidget {
                    project_list: true,
                    compact_quotas: true,
                    show_weekly: false,
                    ..Default::default()
                }),
                style: TextStyle {
                    font: FontSpec {
                        family: testkit::FAMILY.into(),
                        ..Default::default()
                    },
                    size: 14.,
                    paint: Paint::solid(Rgba::WHITE),
                    valign: VAlign::Top,
                    ..Default::default()
                },
            },
        )],
    );
    let mut scene = Scene::empty();
    scene.snapshot.agents_sampled_at = 100;
    scene.snapshot.agents.push(AgentSession {
        id: "one".into(),
        project: "bezel".into(),
        project_path: "F:/dev/bezel".into(),
        state: AgentState::Working,
        updated_at: 100,
        quota_updated_at: 100,
        five_hour: Some(AgentQuota {
            used_percent: 0.,
            resets_at: 1000,
        }),
        ..Default::default()
    });
    let mut renderer = testkit::renderer();
    let before = testkit::render(&mut renderer, &theme, &scene);
    assert!(green(testkit::px(&before, 20, 41)), "automatic row");
    assert!(
        testkit::px(&before, 100, 226).r > 200,
        "automatic quota bar at the bottom"
    );
    if let ElementKind::Text {
        content: TextContent::Agent(a),
        ..
    } = &mut theme.elements[0].kind
    {
        *a.layout = AgentLayout {
            rows: Some(PartRect {
                x: 0.5,
                y: 0.5,
                width: 0.5,
                height: 0.3,
            }),
            ..AgentLayout::default()
        };
    }
    let after = testkit::render(&mut renderer, &theme, &scene);
    assert!(!green(testkit::px(&after, 20, 41)), "the row left");
    assert!(
        green(testkit::px(&after, 190, 127)),
        "the row is in its own place"
    );
    assert!(testkit::px(&after, 100, 226).r > 200, "the quota stays");
}
