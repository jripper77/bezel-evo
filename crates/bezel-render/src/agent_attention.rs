//! Stateful, bounded status-change pulses; transcript refreshes are not events.
use std::{collections::BTreeMap, time::Duration};

use bezel_core::domain::{
    agent::{AgentState, AgentWidget},
    sensor::Snapshot,
};

const LENGTH: Duration = Duration::from_secs(2);
const STEP: Duration = Duration::from_millis(33);

#[derive(Default, Debug, PartialEq)]
pub(crate) struct Pulse {
    pub strength: f32,
    pub projects: BTreeMap<String, AgentState>,
}

pub(crate) fn project_key(session: &bezel_core::domain::agent::AgentSession) -> String {
    if session.project_path.is_empty() {
        session.id.clone()
    } else {
        session
            .project_path
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_lowercase()
    }
}

#[derive(Default)]
pub(crate) struct Attention {
    widget: Option<AgentWidget>,
    states: BTreeMap<String, AgentState>,
    started: Option<Duration>,
    last: Duration,
    changed: BTreeMap<String, AgentState>,
}

impl Attention {
    /// Returns the visible phase and its next deadline in the render clock.
    pub(crate) fn update(
        &mut self,
        widget: &AgentWidget,
        snapshot: &Snapshot,
        now: Duration,
        motion: bool,
    ) -> (Pulse, Option<Duration>) {
        let sessions = if widget.project_list {
            // Detect completion even when completed rows are immediately hidden.
            let mut all = widget.clone();
            all.recent_minutes = u32::MAX;
            all.projects(snapshot)
        } else {
            widget.selected(snapshot).into_iter().collect()
        };
        let states = sessions
            .into_iter()
            .map(|s| (project_key(s), s.state))
            .collect::<BTreeMap<_, _>>();
        let reset = self.widget.as_ref() != Some(widget) || now < self.last;
        if reset || !motion || !widget.blink_on_change {
            self.started = None;
        } else {
            let changed: BTreeMap<_, _> = states
                .iter()
                .filter(|(key, state)| {
                    self.states
                        .get(*key)
                        .is_some_and(|previous| previous != *state)
                })
                .map(|(key, state)| (key.clone(), *state))
                .collect();
            if !changed.is_empty() {
                self.started = Some(now);
                self.changed = changed;
            }
        }
        self.widget = Some(widget.clone());
        self.states = states;
        self.last = now;
        let Some(start) = self.started else {
            return (Pulse::default(), None);
        };
        let elapsed = now.saturating_sub(start);
        if elapsed >= LENGTH {
            self.started = None;
            return (Pulse::default(), None);
        }
        let progress = elapsed.as_secs_f32() / LENGTH.as_secs_f32();
        // One soft breath: no hard on/off transitions, and the text never moves.
        let strength = (std::f32::consts::PI * progress).sin().powi(2);
        (
            Pulse {
                strength,
                projects: self.changed.clone(),
            },
            Some((now + STEP).min(start + LENGTH)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bezel_core::domain::agent::AgentSession;
    #[test]
    fn status_pulse_eases_in_and_out_and_stops_after_two_seconds() {
        let widget = AgentWidget {
            blink_on_change: true,
            ..Default::default()
        };
        let mut snapshot = Snapshot::default();
        snapshot.agents.push(AgentSession {
            id: "one".into(),
            state: AgentState::Working,
            ..Default::default()
        });
        let mut attention = Attention::default();
        let at = |ms| Duration::from_millis(ms);
        assert!(
            attention
                .update(&widget, &snapshot, at(0), true)
                .1
                .is_none()
        );
        snapshot.agents[0].message = "Different text".into();
        assert!(
            attention
                .update(&widget, &snapshot, at(500), true)
                .1
                .is_none()
        );
        snapshot.agents[0].state = AgentState::Completed;
        let (start, due) = attention.update(&widget, &snapshot, at(1000), true);
        assert_eq!(start.strength, 0.);
        assert_eq!(due, Some(at(1033)));
        let early = attention
            .update(&widget, &snapshot, at(1250), true)
            .0
            .strength;
        let middle = attention
            .update(&widget, &snapshot, at(2000), true)
            .0
            .strength;
        let late = attention
            .update(&widget, &snapshot, at(2750), true)
            .0
            .strength;
        assert!(early > 0. && early < 0.2 && middle > 0.99 && (early - late).abs() < 0.001);
        assert_eq!(
            attention.update(&widget, &snapshot, at(3000), true),
            (Pulse::default(), None)
        );
        snapshot.agents[0].state = AgentState::Working;
        assert!(
            attention
                .update(&widget, &snapshot, at(4000), false)
                .1
                .is_none()
        );
        assert!(
            attention
                .update(&widget, &snapshot, at(4500), true)
                .1
                .is_none()
        );
        snapshot.agents[0].state = AgentState::Waiting;
        assert!(
            attention
                .update(&widget, &snapshot, at(5000), true)
                .1
                .is_some()
        );
        assert!(
            attention
                .update(&widget, &snapshot, at(0), true)
                .1
                .is_none()
        );
    }
    #[test]
    fn only_changed_projects_are_highlighted_without_startup_or_source_switch_pulses() {
        let mut widget = AgentWidget {
            project_list: true,
            recent_minutes: 0,
            blink_on_change: true,
            ..Default::default()
        };
        let mut snapshot = Snapshot::default();
        snapshot.agents_sampled_at = 100;
        snapshot.agents.push(AgentSession {
            project_path: "F:/dev/one".into(),
            state: AgentState::Working,
            updated_at: 100,
            ..Default::default()
        });
        let mut attention = Attention::default();
        assert!(
            attention
                .update(&widget, &snapshot, Duration::ZERO, true)
                .1
                .is_none()
        );
        snapshot.agents[0].state = AgentState::Completed;
        assert!(widget.projects(&snapshot).is_empty());
        let (pulse, _) = attention.update(&widget, &snapshot, Duration::from_secs(1), true);
        assert_eq!(
            pulse.projects.get("f:/dev/one"),
            Some(&AgentState::Completed)
        );
        snapshot.agents.push(AgentSession {
            project_path: "F:/dev/two".into(),
            state: AgentState::Working,
            updated_at: 100,
            ..Default::default()
        });
        assert!(
            attention
                .update(&widget, &snapshot, Duration::from_secs(4), true)
                .1
                .is_none()
        );
        widget.project_list = false;
        widget.project = "F:/dev/two".into();
        assert!(
            attention
                .update(&widget, &snapshot, Duration::from_secs(5), true)
                .1
                .is_none()
        );
    }
}
