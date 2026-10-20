//! Transient playback: never changes a saved face or editor Undo history.
use crate::domain::{
    sensor::Snapshot,
    theme::{Card, ElementId, Theme},
};
use std::{borrow::Cow, collections::BTreeMap, time::Duration};
struct State {
    config: Card,
    face: usize,
    base: usize,
    override_rule: Option<usize>,
    release: Option<Duration>,
    due: Vec<Option<Duration>>,
}
#[derive(Default)]
pub(super) struct CardTimers {
    states: BTreeMap<ElementId, State>,
    last: Duration,
    pub next: Option<Duration>,
}
impl CardTimers {
    pub fn update(
        &mut self,
        theme: &Theme,
        snapshot: &Snapshot,
        now: Duration,
        enabled: bool,
    ) -> bool {
        let mut changed = false;
        if !enabled || now < self.last {
            changed = !self.states.is_empty();
            self.states.clear();
        }
        self.last = now;
        self.next = None;
        self.states.retain(|id, _| {
            theme.element(*id).is_some_and(|e| {
                theme.is_visible(e)
                    && e.card.as_ref().is_some_and(|c| {
                        (c.rotation_seconds.is_some() || !c.triggers.is_empty())
                            && c.faces.len() > 1
                    })
            })
        });
        if !enabled {
            return changed;
        }
        for e in &theme.elements {
            let Some(card) = e.card.as_ref().filter(|c| {
                theme.is_visible(e)
                    && c.faces.len() > 1
                    && (c.rotation_seconds.is_some() || !c.triggers.is_empty())
            }) else {
                continue;
            };
            let card = card.normalized();
            let deadlines = || {
                card.triggers
                    .iter()
                    .map(|r| {
                        (r.source == crate::domain::playback::TriggerSource::Timer)
                            .then_some(r.interval_seconds)
                            .flatten()
                            .filter(|s| (5..=3600).contains(s))
                            .map(|s| now + Duration::from_secs(u64::from(s)))
                    })
                    .collect::<Vec<_>>()
            };
            let fresh = || State {
                config: card.clone().into_owned(),
                face: card.active_face,
                base: card.active_face,
                override_rule: None,
                release: None,
                due: deadlines(),
            };
            let state = self.states.entry(e.id).or_insert_with(fresh);
            if state.config != *card {
                *state = fresh();
                changed = true;
            }
            let old = state.face;
            let winner = card
                .triggers
                .iter()
                .enumerate()
                .filter(|(_, r)| r.face < card.faces.len() && r.matches(snapshot))
                .max_by(|(a, x), (b, y)| x.priority.cmp(&y.priority).then_with(|| b.cmp(a)))
                .map(|(i, _)| i);
            if let Some(i) = winner {
                if state.override_rule.is_none() {
                    state.base = state.face;
                }
                state.override_rule = Some(i);
                state.release = None;
                state.face = card.triggers[i].face;
            } else if let Some(i) = state.override_rule {
                let until = *state.release.get_or_insert(
                    now + Duration::from_secs(u64::from(card.triggers[i].return_seconds)),
                );
                if now >= until {
                    state.override_rule = None;
                    state.release = None;
                    state.face = card.triggers[i]
                        .return_face
                        .filter(|f| *f < card.faces.len())
                        .unwrap_or(state.base);
                    state.due = deadlines();
                }
            } else {
                let timed = state
                    .due
                    .iter()
                    .enumerate()
                    .filter(|(_, due)| due.is_some_and(|due| now >= due))
                    .max_by(|(a, _), (b, _)| {
                        card.triggers[*a]
                            .priority
                            .cmp(&card.triggers[*b].priority)
                            .then_with(|| b.cmp(a))
                    })
                    .map(|(i, _)| i);
                if let Some(i) = timed {
                    let r = &card.triggers[i];
                    state.face = if r.next_face {
                        (state.face + 1) % card.faces.len()
                    } else {
                        r.face
                    };
                    for (n, due) in state.due.iter_mut().enumerate() {
                        if due.is_some_and(|d| now >= d) {
                            *due = card.triggers[n]
                                .interval_seconds
                                .map(|s| now + Duration::from_secs(u64::from(s)));
                        }
                    }
                }
            }
            changed |= old != state.face;
            let next = if state.override_rule.is_some() {
                state.release
            } else {
                state.due.iter().flatten().copied().min()
            };
            if let Some(due) = next {
                self.next = Some(self.next.map_or(due, |n| n.min(due)));
            }
        }
        changed
    }
    pub fn theme<'a>(&self, theme: &'a Theme) -> Cow<'a, Theme> {
        if !self.states.iter().any(|(id, s)| {
            theme
                .element(*id)
                .and_then(|e| e.card.as_ref())
                .is_some_and(|c| c.active_face != s.face)
        }) {
            return Cow::Borrowed(theme);
        }
        let mut rendered = theme.clone();
        for e in &mut rendered.elements {
            if let Some(s) = self.states.get(&e.id)
                && let Some(c) = &mut e.card
            {
                c.active_face = s.face;
            }
        }
        Cow::Owned(rendered)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::domain::{
        geometry::{Orientation, Size},
        playback::{CardTrigger, MediaSession, TriggerSource},
        theme::{BoxF, Element, ElementKind, ShapeKind},
    };
    fn theme() -> Theme {
        let mut t = Theme::blank("rules", Size::new(480, 1920), Orientation::Portrait);
        t.elements.push(Element {
            color_by_value: None,
            visualization: None,
            value_transition: None,
            sensor_motion: None,
            id: ElementId(1),
            name: "card".into(),
            frame: BoxF::new(0., 0., 100., 100.),
            opacity: 1.,
            visible: true,
            locked: false,
            rotation: 0.0,
            is_group: false,
            group_parent: None,
            card_member: None,
            kind: ElementKind::Shape {
                image: None,
                video_window: false,
                fade: None,
                shape: ShapeKind::Rect { radius: 0. },
                fill: None,
                stroke: None,
            },
            card: Some(Card {
                faces: vec!["base".into(), "music".into(), "game".into()],
                active_face: 0,
                rotation_seconds: Some(10),
                transition: None,
                triggers: vec![
                    CardTrigger {
                        interval_seconds: None,
                        next_face: false,
                        source: TriggerSource::MediaPlaying,
                        app: "Spotify".into(),
                        face: 1,
                        priority: 1,
                        return_seconds: 3,
                        return_face: None,
                    },
                    CardTrigger {
                        interval_seconds: None,
                        next_face: false,
                        source: TriggerSource::Foreground,
                        app: "game.exe".into(),
                        face: 2,
                        priority: 2,
                        return_seconds: 0,
                        return_face: None,
                    },
                ],
            }),
        });
        t
    }
    fn face(timers: &CardTimers, t: &Theme) -> usize {
        timers.theme(t).elements[0]
            .card
            .as_ref()
            .unwrap()
            .active_face
    }
    #[test]
    fn timed_rules_choose_next_or_specific_face_and_keep_editing_pause() {
        let mut t = theme();
        let c = t.elements[0].card.as_mut().unwrap();
        c.rotation_seconds = None;
        c.triggers = vec![CardTrigger {
            source: TriggerSource::Timer,
            interval_seconds: Some(5),
            next_face: true,
            app: String::new(),
            face: 0,
            priority: 0,
            return_seconds: 0,
            return_face: None,
        }];
        let mut timers = CardTimers::default();
        let s = Snapshot::default();
        timers.update(&t, &s, Duration::ZERO, true);
        assert_eq!(timers.next, Some(Duration::from_secs(5)));
        timers.update(&t, &s, Duration::from_secs(5), true);
        assert_eq!(face(&timers, &t), 1);
        timers.update(&t, &s, Duration::from_secs(10), true);
        assert_eq!(face(&timers, &t), 2);
        timers.update(&t, &s, Duration::from_secs(15), true);
        assert_eq!(face(&timers, &t), 0);
        timers.update(&t, &s, Duration::from_secs(16), false);
        assert_eq!(timers.next, None);
        timers.update(&t, &s, Duration::from_secs(30), true);
        assert_eq!(timers.next, Some(Duration::from_secs(35)));
        let c = t.elements[0].card.as_mut().unwrap();
        c.triggers.push(CardTrigger {
            source: TriggerSource::Timer,
            interval_seconds: Some(5),
            next_face: false,
            app: String::new(),
            face: 2,
            priority: 10,
            return_seconds: 0,
            return_face: None,
        });
        timers.update(&t, &s, Duration::from_secs(31), true);
        timers.update(&t, &s, Duration::from_secs(36), true);
        assert_eq!(face(&timers, &t), 2, "higher-priority due timer wins");
        assert_eq!(
            t.elements[0].card.as_ref().unwrap().active_face,
            0,
            "playback never edits saved face"
        );
    }

    #[test]
    fn priority_pause_return_and_editing_leave_saved_faces_untouched() {
        let t = theme();
        let mut timers = CardTimers::default();
        let mut s = Snapshot::default();
        timers.update(&t, &s, Duration::ZERO, true);
        s.media.push(MediaSession {
            source: "Spotify.exe".into(),
            playing: true,
            ..MediaSession::default()
        });
        assert!(timers.update(&t, &s, Duration::from_secs(1), true));
        assert_eq!(face(&timers, &t), 1);
        s.foreground = Some("game".into());
        timers.update(&t, &s, Duration::from_secs(2), true);
        assert_eq!(face(&timers, &t), 2);
        s.foreground = None;
        timers.update(&t, &s, Duration::from_secs(3), true);
        assert_eq!(face(&timers, &t), 1);
        s.media[0].playing = false;
        timers.update(&t, &s, Duration::from_secs(4), true);
        assert_eq!(face(&timers, &t), 1);
        timers.update(&t, &s, Duration::from_secs(6), true);
        assert_eq!(face(&timers, &t), 1);
        timers.update(&t, &s, Duration::from_secs(7), true);
        assert_eq!(face(&timers, &t), 0);
        timers.update(&t, &s, Duration::from_secs(16), true);
        assert_eq!(face(&timers, &t), 0);
        timers.update(&t, &s, Duration::from_secs(17), true);
        assert_eq!(face(&timers, &t), 1);
        timers.update(&t, &s, Duration::from_secs(18), false);
        assert_eq!(face(&timers, &t), 0);
        assert_eq!(t.elements[0].card.as_ref().unwrap().active_face, 0);
    }
    #[test]
    fn explicit_return_face_applies_when_media_session_disappears() {
        let mut t = theme();
        let c = t.elements[0].card.as_mut().unwrap();
        c.active_face = 1;
        c.rotation_seconds = None;
        c.triggers.truncate(1);
        c.triggers[0].return_face = Some(0);
        c.triggers[0].return_seconds = 0;
        let mut timers = CardTimers::default();
        let mut s = Snapshot::default();
        s.media.push(MediaSession {
            source: "Spotify".into(),
            playing: true,
            ..MediaSession::default()
        });
        timers.update(&t, &s, Duration::ZERO, true);
        assert_eq!(face(&timers, &t), 1);
        s.media.clear();
        timers.update(&t, &s, Duration::from_secs(1), true);
        assert_eq!(face(&timers, &t), 0);
        assert_eq!(t.elements[0].card.as_ref().unwrap().active_face, 1);
    }

    #[test]
    fn closed_process_does_not_match_missing_measurements_and_rewind_resets() {
        let mut t = theme();
        let c = t.elements[0].card.as_mut().unwrap();
        c.rotation_seconds = None;
        c.triggers = vec![CardTrigger {
            interval_seconds: None,
            next_face: false,
            source: TriggerSource::ProcessClosed,
            app: "GAME.exe".into(),
            face: 2,
            priority: 0,
            return_seconds: 0,
            return_face: None,
        }];
        let mut timers = CardTimers::default();
        let mut s = Snapshot::default();
        timers.update(&t, &s, Duration::from_secs(20), true);
        assert_eq!(face(&timers, &t), 0);
        s.activity_available = true;
        timers.update(&t, &s, Duration::from_secs(21), true);
        assert_eq!(face(&timers, &t), 2);
        s.applications.insert("game".into());
        timers.update(&t, &s, Duration::from_secs(22), true);
        assert_eq!(face(&timers, &t), 0);
        timers.update(&t, &s, Duration::ZERO, true);
        assert_eq!(face(&timers, &t), 0);
    }
}
