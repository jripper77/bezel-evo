//! Local coding-agent sessions and the display configured by one widget.
use super::{clock::Language, frame::Rgba, sensor::Snapshot};

/// Supported local coding agent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentKind {
    #[default]
    /// OpenAI Codex.
    Codex,
    /// Anthropic Claude Code.
    Claude,
}

impl AgentKind {
    /// Display name of this agent.
    pub fn name(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::Claude => "Claude Code",
        }
    }
}

/// Last state explicitly observed in the transcript.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AgentState {
    /// A turn or tool is running.
    Working,
    /// The turn was interrupted or awaits input.
    Waiting,
    /// The last turn finished.
    Completed,
    /// The agent reported an error.
    Error,
    #[default]
    /// No reliable current state is available.
    Unknown,
}

/// One account usage window.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentQuota {
    /// Consumed share of the window, from 0 to 100.
    pub used_percent: f64,
    /// Next reset as Unix seconds.
    pub resets_at: u64,
}

/// Bounded public information from one local transcript.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentSession {
    /// Agent selected for this session or widget.
    pub agent: AgentKind,
    /// Stable session identifier.
    pub id: String,
    /// Project directory name.
    pub project: String,
    /// Full working directory, used to distinguish projects with the same name.
    pub project_path: String,
    /// Latest visible assistant message.
    pub message: String,
    /// Latest tool name.
    pub activity: String,
    /// Last state explicitly observed in the transcript.
    pub state: AgentState,
    /// Transcript modification time as Unix seconds.
    pub updated_at: u64,
    /// Last quota observation as Unix seconds, independent of message updates.
    pub quota_updated_at: u64,
    /// Account usage in the five-hour window.
    pub five_hour: Option<AgentQuota>,
    /// Account usage in the weekly window.
    pub weekly: Option<AgentQuota>,
}

/// One widget selecting an agent and session.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentWidget {
    /// Agent selected for this session or widget.
    pub agent: AgentKind,
    /// Empty follows the most recently updated session of this agent.
    pub session: String,
    /// Empty follows all projects. Otherwise follows this directory and its children.
    pub project: String,
    /// Show an automatically paginated list of projects instead of one session.
    pub project_list: bool,
    /// Keep completed and unknown projects visible for this many minutes.
    pub recent_minutes: u32,
    /// Seconds each project page stays visible.
    pub page_seconds: u32,
    /// Full project paths excluded from the automatic list.
    pub excluded_projects: Vec<String>,
    /// Display only the agent, project and lifecycle status.
    pub status_only: bool,
    /// Quota text size relative to the project text, in percent.
    pub quota_size_percent: u32,
    /// Use square segmented quota meters with the Pixel orange accent.
    pub pixel_style: bool,
    /// Briefly highlight this widget when an observed project changes state.
    pub blink_on_change: bool,
    /// Display remaining quota and reset on one line per window.
    pub compact_quotas: bool,
    /// Display the latest visible assistant message.
    pub show_message: bool,
    /// Latest tool name.
    pub show_activity: bool,
    /// Display lifecycle state and update age.
    pub show_status: bool,
    /// Account usage in the five-hour window.
    pub show_five_hour: bool,
    /// Account usage in the weekly window.
    pub show_weekly: bool,
    /// Display reset countdowns.
    pub show_reset: bool,
    /// Maximum visible message lines, from 1 to 12.
    pub message_lines: usize,
    /// Optional colors; unset entries keep the built-in palette.
    pub colors: AgentColors,
    /// Remaining quota percentage at or below which meters turn to the warning color; 0 disables.
    pub warning_percent: u32,
    /// Custom header replacing the agent name; empty keeps the agent name.
    pub title: String,
    /// Show the header row of the project list.
    pub show_header: bool,
    /// Show the page counter in the project-list header.
    pub show_page: bool,
    /// Draw thin dividers between project rows.
    pub show_dividers: bool,
    /// Show the localized state label next to the status dot in project rows.
    pub show_state_text: bool,
    /// Show the time since the last update in project rows.
    pub show_age: bool,
    /// Show the current tool of working projects in their row.
    pub show_row_activity: bool,
    /// Quota figure to print; `None` follows the compact choice (remaining) or both.
    pub quota_value: Option<QuotaValue>,
    /// Draw the status dot at the start of each project row.
    pub show_dot: bool,
    /// Print the full project path instead of the folder name.
    pub show_path: bool,
    /// Print the state under the project name instead of on the right.
    pub state_below: bool,
    /// Project row height in pixels; `None` follows the text size.
    pub row_height: Option<u16>,
    /// Quota meter thickness in pixels; `None` keeps 4 (5 for Pixel meters).
    pub bar_height: Option<u8>,
    /// Gap between Pixel meter segments in pixels; `None` keeps 3.
    pub segment_gap: Option<u8>,
    /// Parts placed by hand; the others keep the automatic layout.
    pub layout: Box<AgentLayout>,
}

/// A part's place inside its widget, as shares of the widget's size (0..=1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl PartRect {
    /// Whether it lies inside the widget and is not empty.
    pub fn is_valid(&self) -> bool {
        let unit = 0.0..=1.0;
        [self.x, self.y, self.width, self.height]
            .iter()
            .all(|v| v.is_finite() && unit.contains(v))
            && self.width > 0.
            && self.height > 0.
            && self.x + self.width <= 1.0001
            && self.y + self.height <= 1.0001
    }
}

/// Parts placed by hand. `None` keeps a part in its automatic place.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AgentLayout {
    /// Header of the project list.
    pub title: Option<PartRect>,
    /// Project rows.
    pub rows: Option<PartRect>,
    /// Five-hour usage window.
    pub five_hour: Option<PartRect>,
    /// Weekly usage window.
    pub weekly: Option<PartRect>,
    /// Single-session text.
    pub content: Option<PartRect>,
}

/// Quota figure printed for each window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaValue {
    /// Share still available.
    Remaining,
    /// Share already consumed.
    Used,
    /// Consumed and available shares.
    Both,
}

/// Colors chosen for one agent widget. `None` keeps the default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AgentColors {
    /// A turn or tool is running.
    pub working: Option<Rgba>,
    /// The agent waits for the user or reported an error.
    pub attention: Option<Rgba>,
    /// The last turn finished.
    pub completed: Option<Rgba>,
    /// No reliable current state.
    pub unknown: Option<Rgba>,
    /// Quota meter fill.
    pub quota: Option<Rgba>,
    /// Quota meter track.
    pub track: Option<Rgba>,
    /// Quota meters and labels at or below the warning threshold.
    pub warning: Option<Rgba>,
}

/// Default green for working sessions.
pub const WORKING_COLOR: Rgba = Rgba {
    r: 48,
    g: 191,
    b: 139,
    a: 255,
};
/// Default red for sessions that need attention, finished turns and low quotas.
pub const ATTENTION_COLOR: Rgba = Rgba {
    r: 244,
    g: 93,
    b: 96,
    a: 255,
};
/// Default grey for unknown states.
pub const UNKNOWN_COLOR: Rgba = Rgba {
    r: 140,
    g: 140,
    b: 140,
    a: 255,
};

impl AgentColors {
    /// Color of a lifecycle state; completed follows attention unless set.
    pub fn state(&self, state: AgentState) -> Rgba {
        let attention = self.attention.unwrap_or(ATTENTION_COLOR);
        match state {
            AgentState::Working => self.working.unwrap_or(WORKING_COLOR),
            AgentState::Waiting | AgentState::Error => attention,
            AgentState::Completed => self.completed.unwrap_or(attention),
            AgentState::Unknown => self.unknown.unwrap_or(UNKNOWN_COLOR),
        }
    }

    /// Warning color for quotas near exhaustion.
    pub fn warning(&self) -> Rgba {
        self.warning.unwrap_or(ATTENTION_COLOR)
    }
}

impl Default for AgentWidget {
    fn default() -> Self {
        Self {
            agent: AgentKind::Codex,
            session: String::new(),
            project: String::new(),
            project_list: false,
            recent_minutes: 10,
            page_seconds: 10,
            excluded_projects: Vec::new(),
            status_only: false,
            quota_size_percent: 100,
            pixel_style: false,
            blink_on_change: false,
            compact_quotas: false,
            show_message: true,
            show_activity: true,
            show_status: true,
            show_five_hour: true,
            show_weekly: true,
            show_reset: true,
            message_lines: 3,
            colors: AgentColors::default(),
            warning_percent: 20,
            title: String::new(),
            show_header: true,
            show_page: true,
            show_dividers: true,
            show_state_text: true,
            show_age: false,
            show_row_activity: false,
            quota_value: None,
            show_dot: true,
            show_path: false,
            state_below: false,
            row_height: None,
            bar_height: None,
            segment_gap: None,
            layout: Box::default(),
        }
    }
}

/// Labels shared by the native renderer, translated for every supported locale.
pub struct AgentLabels {
    /// Missing quota label.
    pub unavailable: &'static str,
    /// Missing session label.
    pub no_session: &'static str,
    /// Working, waiting, completed, error and unknown labels.
    pub states: [&'static str; 5],
    /// Consumed quota label.
    pub used: &'static str,
    /// Remaining quota label.
    pub remaining: &'static str,
    /// Reset countdown label.
    pub reset: &'static str,
    /// Account usage in the weekly window.
    pub weekly: &'static str,
    /// Update age label.
    pub updated: &'static str,
}

/// Localized labels for the native agent display.
pub fn labels(language: Language) -> AgentLabels {
    match language {
        Language::Italian => AgentLabels {
            unavailable: "Non disponibile",
            no_session: "Nessuna sessione rilevata",
            states: [
                "Al lavoro",
                "Tocca a te",
                "Finito",
                "Errore",
                "Stato sconosciuto",
            ],
            used: "utilizzato",
            remaining: "residuo",
            reset: "Ripristino tra",
            weekly: "Settimana",
            updated: "Aggiornato",
        },
        Language::PortugueseBr => AgentLabels {
            unavailable: "Indisponível",
            no_session: "Nenhuma sessão detectada",
            states: [
                "Trabalhando",
                "Aguardando",
                "Concluído",
                "Erro",
                "Estado desconhecido",
            ],
            used: "utilizado",
            remaining: "restante",
            reset: "Renova em",
            weekly: "Semana",
            updated: "Atualizado",
        },
        Language::English => AgentLabels {
            unavailable: "Unavailable",
            no_session: "No session detected",
            states: ["Working", "Waiting", "Completed", "Error", "Unknown status"],
            used: "used",
            remaining: "remaining",
            reset: "Resets in",
            weekly: "Weekly",
            updated: "Updated",
        },
    }
}

/// Compact duration rounded up to minutes.
pub fn duration(seconds: u64) -> String {
    let minutes = seconds.div_ceil(60);
    if minutes >= 1440 {
        format!("{}d {}h", minutes / 1440, minutes % 1440 / 60)
    } else {
        format!("{}h {}m", minutes / 60, minutes % 60)
    }
}

/// Update age: seconds under a minute, then the compact duration.
pub fn age(seconds: u64) -> String {
    match seconds {
        0..60 => format!("{seconds}s"),
        60..3600 => format!("{}m", seconds / 60),
        _ => duration(seconds),
    }
}

impl AgentWidget {
    /// The pinned session, or the latest session for the chosen agent.
    pub fn selected<'a>(&self, snapshot: &'a Snapshot) -> Option<&'a AgentSession> {
        snapshot
            .agents
            .iter()
            .filter(|s| s.agent == self.agent && (self.session.is_empty() || self.session == s.id))
            .filter(|s| project_matches(&self.project, &s.project_path))
            .max_by_key(|s| s.updated_at)
    }

    /// Latest account observation, independent of a pinned project or visible rows.
    pub fn account<'a>(&self, snapshot: &'a Snapshot) -> Option<&'a AgentSession> {
        snapshot
            .agents
            .iter()
            .filter(|s| s.agent == self.agent)
            .max_by_key(|s| (s.quota_updated_at, s.updated_at))
    }

    /// One current session per full project path, sorted by attention and activity.
    pub fn projects<'a>(&self, snapshot: &'a Snapshot) -> Vec<&'a AgentSession> {
        let mut latest = std::collections::BTreeMap::new();
        for session in snapshot.agents.iter().filter(|s| s.agent == self.agent) {
            let key = session
                .project_path
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_lowercase();
            if key.is_empty()
                || self
                    .excluded_projects
                    .iter()
                    .any(|p| project_matches(p, &key))
            {
                continue;
            }
            let entry = latest.entry(key).or_insert(session);
            if (session.updated_at, &session.id) > (entry.updated_at, &entry.id) {
                *entry = session;
            }
        }
        let priority = |s: &AgentSession| match s.state {
            AgentState::Waiting | AgentState::Error => 0,
            AgentState::Working => 1,
            _ => 2,
        };
        let mut projects: Vec<_> = latest
            .into_values()
            .filter(|s| {
                priority(s) < 2
                    || (self.recent_minutes > 0
                        && snapshot.agents_sampled_at.saturating_sub(s.updated_at)
                            <= u64::from(self.recent_minutes) * 60)
            })
            .collect();
        projects.sort_by(|a, b| {
            priority(a)
                .cmp(&priority(b))
                .then_with(|| b.updated_at.cmp(&a.updated_at))
                .then_with(|| a.project_path.cmp(&b.project_path))
        });
        projects
    }

    /// Header text: the custom title, or the agent name.
    pub fn header(&self) -> &str {
        match self.title.trim() {
            "" => self.agent.name(),
            title => title,
        }
    }

    /// Quota value shown; unset keeps the layout's historic choice.
    pub fn quota_shows(&self) -> QuotaValue {
        self.quota_value.unwrap_or(if self.compact_quotas {
            QuotaValue::Remaining
        } else {
            QuotaValue::Both
        })
    }

    /// Filled share of a quota meter: what remains, or what was used.
    pub fn meter_fraction(&self, quota: &AgentQuota) -> f64 {
        let used = quota.used_percent.clamp(0., 100.) / 100.;
        match self.quota_shows() {
            QuotaValue::Remaining => 1. - used,
            QuotaValue::Used => used,
            QuotaValue::Both if self.compact_quotas => 1. - used,
            QuotaValue::Both => used,
        }
    }

    /// Whether a quota window is at or below the warning threshold.
    pub fn quota_low(&self, quota: &AgentQuota) -> bool {
        self.warning_percent > 0
            && 100. - quota.used_percent.clamp(0., 100.) <= f64::from(self.warning_percent)
    }

    /// Right-hand detail of one project row: state, age and current tool as configured.
    pub fn row_detail(&self, session: &AgentSession, now: u64, language: Language) -> String {
        let mut parts = Vec::new();
        if self.show_row_activity
            && session.state == AgentState::Working
            && !session.activity.is_empty()
        {
            parts.push(session.activity.clone());
        }
        if self.show_state_text {
            parts.push(self.state_text(session.state, language).to_string());
        }
        if self.show_age {
            parts.push(age(now.saturating_sub(session.updated_at)));
        }
        parts.join(" \u{00b7} ")
    }

    /// Message and lifecycle lines for this widget.
    pub fn text(&self, snapshot: &Snapshot, language: Language) -> String {
        let labels = labels(language);
        if self.project_list {
            let projects = self.projects(snapshot);
            let mut lines = Vec::new();
            if self.show_header {
                lines.push(self.header().to_string());
            }
            for session in &projects {
                lines.push(format!(
                    "{}  \u{00b7}  {}",
                    session.project_path,
                    self.row_detail(session, snapshot.agents_sampled_at, language)
                ));
            }
            if projects.is_empty() {
                lines.push(labels.no_session.to_string());
            }
            return lines.join("\n");
        }
        let Some(session) = self.selected(snapshot) else {
            return format!("{}\n{}", self.header(), labels.no_session);
        };
        let mut lines = vec![format!("{} · {}", self.header(), session.project)];
        if self.status_only {
            lines.push(self.state_text(session.state, language).to_string());
            return lines.join("\n");
        }
        if self.show_status {
            let index = match session.state {
                AgentState::Working => 0,
                AgentState::Waiting => 1,
                AgentState::Completed => 2,
                AgentState::Error => 3,
                AgentState::Unknown => 4,
            };
            let age = snapshot
                .agents_sampled_at
                .saturating_sub(session.updated_at);
            lines.push(format!(
                "{} · {} {}",
                labels.states[index],
                labels.updated,
                self::age(age)
            ));
        }
        if self.show_message {
            let message: Vec<_> = session.message.lines().collect();
            lines.extend(
                message
                    .iter()
                    .rev()
                    .take(self.message_lines)
                    .rev()
                    .map(|s| (*s).to_string()),
            );
        }
        if self.show_activity && !session.activity.is_empty() {
            lines.push(session.activity.clone());
        }
        lines.join("\n")
    }

    /// One account usage window.
    pub fn quota_text(
        &self,
        quota: Option<&AgentQuota>,
        title: &str,
        now: u64,
        language: Language,
    ) -> String {
        let labels = labels(language);
        let Some(quota) = quota.filter(|q| q.used_percent.is_finite() && q.resets_at > now) else {
            return format!("{title} · {}", labels.unavailable);
        };
        let used = quota.used_percent.clamp(0., 100.);
        let value = match self.quota_shows() {
            QuotaValue::Remaining => format!("{:.0}% {}", 100. - used, labels.remaining),
            QuotaValue::Used => format!("{used:.0}% {}", labels.used),
            QuotaValue::Both => format!(
                "{used:.0}% {} · {:.0}% {}",
                labels.used,
                100. - used,
                labels.remaining
            ),
        };
        if self.compact_quotas {
            let title = if title == labels.weekly {
                match language {
                    Language::Italian => "7g",
                    _ => "7d",
                }
            } else {
                title
            };
            let mut text = format!("{title} · {value}");
            if self.show_reset {
                text.push_str(&format!(" · {}", duration(quota.resets_at - now)));
            }
            return text;
        }
        let mut text = format!("{title} · {value}");
        if self.show_reset {
            text.push_str(&format!(
                "\n{} {}",
                labels.reset,
                duration(quota.resets_at - now)
            ));
        }
        text
    }

    /// Localized lifecycle state for compact displays.
    pub fn state_text(&self, state: AgentState, language: Language) -> &'static str {
        let index = match state {
            AgentState::Working => 0,
            AgentState::Waiting => 1,
            AgentState::Completed => 2,
            AgentState::Error => 3,
            AgentState::Unknown => 4,
        };
        labels(language).states[index]
    }
}

/// Windows project matching is case-insensitive and respects directory boundaries.
pub fn project_matches(filter: &str, directory: &str) -> bool {
    let normalize = |path: &str| path.replace('\\', "/").trim_end_matches('/').to_lowercase();
    let filter = normalize(filter);
    let directory = normalize(directory);
    filter.is_empty() || directory == filter || directory.starts_with(&format!("{filter}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_list_groups_full_paths_and_prioritizes_latest_project_state() {
        let mut snapshot = Snapshot::default();
        snapshot.agents_sampled_at = 2000;
        let session = |id: &str, path: &str, state, updated_at| AgentSession {
            id: id.into(),
            project: "same".into(),
            project_path: path.into(),
            state,
            updated_at,
            ..Default::default()
        };
        snapshot.agents = vec![
            session("old", "F:/dev/one", AgentState::Working, 100),
            session("new", "f:\\DEV\\one", AgentState::Completed, 1950),
            session("attention", "F:/dev/two", AgentState::Waiting, 1800),
            session("working", "F:/dev/three", AgentState::Working, 1900),
            session("stale", "F:/dev/four", AgentState::Completed, 1000),
            session("fourth", "F:/dev/five", AgentState::Completed, 1960),
        ];
        let mut widget = AgentWidget {
            project_list: true,
            ..Default::default()
        };
        assert_eq!(
            widget
                .projects(&snapshot)
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            vec!["attention", "working", "fourth", "new"]
        );
        widget.excluded_projects.push("F:/DEV/two".into());
        widget.recent_minutes = 0;
        assert_eq!(
            widget
                .projects(&snapshot)
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            vec!["working"]
        );
        widget.project = "F:/unavailable".into();
        snapshot.agents[0].quota_updated_at = 1999;
        assert!(widget.selected(&snapshot).is_none());
        assert_eq!(widget.account(&snapshot).unwrap().id, "old");
    }

    #[test]
    fn project_filter_follows_new_sessions_inside_one_workspace() {
        let widget = AgentWidget {
            project: "F:\\dev\\bezel".into(),
            status_only: true,
            ..Default::default()
        };
        let mut snapshot = Snapshot::default();
        snapshot.agents = vec![
            AgentSession {
                id: "bezel-new".into(),
                project: "bezel".into(),
                project_path: "f:/DEV/bezel".into(),
                updated_at: 20,
                state: AgentState::Working,
                message: "hidden".into(),
                ..Default::default()
            },
            AgentSession {
                id: "eidolon".into(),
                project: "EIDOLON".into(),
                project_path: "F:/dev/EIDOLON".into(),
                updated_at: 30,
                ..Default::default()
            },
            AgentSession {
                id: "other".into(),
                project_path: "F:/dev/bezel-other".into(),
                updated_at: 40,
                ..Default::default()
            },
        ];
        assert_eq!(widget.selected(&snapshot).unwrap().id, "bezel-new");
        assert_eq!(
            widget.text(&snapshot, Language::Italian),
            "Codex · bezel\nAl lavoro"
        );
        assert!(project_matches("F:/dev/bezel", "f:/dev/bezel/crates"));
        assert!(!project_matches("F:/dev/bezel", "f:/dev/bezel-other"));
    }

    #[test]
    fn compact_quotas_show_remaining_and_reset_on_one_line() {
        let widget = AgentWidget {
            compact_quotas: true,
            ..Default::default()
        };
        let quota = AgentQuota {
            used_percent: 16.,
            resets_at: 3700,
        };
        assert_eq!(
            widget.quota_text(Some(&quota), "5h", 100, Language::Italian),
            "5h · 84% residuo · 1h 0m"
        );
        assert!(
            !widget
                .quota_text(
                    Some(&quota),
                    labels(Language::Italian).weekly,
                    100,
                    Language::Italian
                )
                .contains('\n')
        );
    }

    #[test]
    fn each_widget_selects_its_agent_and_pinned_session() {
        let mut snapshot = Snapshot::default();
        snapshot.agents_sampled_at = 100;
        snapshot.agents = vec![
            AgentSession {
                id: "old".into(),
                message: "first\nsecond\nthird".into(),
                updated_at: 50,
                ..Default::default()
            },
            AgentSession {
                id: "new".into(),
                updated_at: 80,
                ..Default::default()
            },
            AgentSession {
                agent: AgentKind::Claude,
                id: "claude".into(),
                updated_at: 90,
                ..Default::default()
            },
        ];
        let mut widget = AgentWidget::default();
        assert_eq!(widget.selected(&snapshot).unwrap().id, "new");
        widget.session = "old".into();
        widget.message_lines = 2;
        let text = widget.text(&snapshot, Language::English);
        assert!(text.contains("second\nthird") && !text.contains("first"));
        widget.agent = AgentKind::Claude;
        assert!(widget.selected(&snapshot).is_none());
        widget.session.clear();
        assert_eq!(widget.selected(&snapshot).unwrap().id, "claude");
    }

    #[test]
    fn expired_or_missing_quotas_are_unavailable_in_every_language() {
        let widget = AgentWidget::default();
        for language in [Language::English, Language::Italian, Language::PortugueseBr] {
            let quota = AgentQuota {
                used_percent: 35.,
                resets_at: 200,
            };
            let text = widget.quota_text(Some(&quota), "5h", 100, language);
            assert!(text.contains("35%") && text.contains("65%"));
            for quota in [None, Some(&quota)] {
                assert!(
                    widget
                        .quota_text(quota, "5h", 200, language)
                        .contains(labels(language).unavailable)
                );
            }
        }
    }

    #[test]
    fn custom_colors_threshold_and_row_details_follow_the_widget() {
        let mut widget = AgentWidget::default();
        assert_eq!(widget.colors.state(AgentState::Completed), ATTENTION_COLOR);
        widget.colors.completed = Some(Rgba::opaque(1, 2, 3));
        widget.colors.attention = Some(Rgba::opaque(9, 9, 9));
        assert_eq!(
            widget.colors.state(AgentState::Completed),
            Rgba::opaque(1, 2, 3)
        );
        assert_eq!(
            widget.colors.state(AgentState::Error),
            Rgba::opaque(9, 9, 9)
        );
        assert_eq!(widget.colors.state(AgentState::Working), WORKING_COLOR);
        let quota = |used_percent| AgentQuota {
            used_percent,
            resets_at: 1,
        };
        assert!(widget.quota_low(&quota(80.)));
        assert!(!widget.quota_low(&quota(79.)));
        widget.warning_percent = 0;
        assert!(!widget.quota_low(&quota(100.)));
        assert_eq!(widget.header(), "Codex");
        widget.title = "  Lavoro  ".into();
        assert_eq!(widget.header(), "Lavoro");
        let session = AgentSession {
            state: AgentState::Working,
            activity: "Bash".into(),
            updated_at: 100,
            ..Default::default()
        };
        assert_eq!(
            widget.row_detail(&session, 130, Language::Italian),
            "Al lavoro"
        );
        widget.show_age = true;
        widget.show_row_activity = true;
        assert_eq!(
            widget.row_detail(&session, 130, Language::Italian),
            "Bash \u{00b7} Al lavoro \u{00b7} 30s"
        );
        widget.show_state_text = false;
        assert_eq!(
            widget.row_detail(&session, 400, Language::English),
            "Bash \u{00b7} 5m"
        );
        assert_eq!(age(7300), "2h 2m");
        let mut snapshot = Snapshot::default();
        snapshot.agents_sampled_at = 130;
        widget.project_list = true;
        widget.show_header = false;
        assert_eq!(
            widget.text(&snapshot, Language::English),
            labels(Language::English).no_session
        );
    }

    #[test]
    fn quota_value_choice_sets_text_and_meter_independently_of_layout() {
        let quota = AgentQuota {
            used_percent: 30.,
            resets_at: 100,
        };
        let mut widget = AgentWidget {
            show_reset: false,
            ..Default::default()
        };
        let text = |w: &AgentWidget| w.quota_text(Some(&quota), "5h", 0, Language::Italian);
        assert_eq!(text(&widget), "5h · 30% utilizzato · 70% residuo");
        assert!((widget.meter_fraction(&quota) - 0.3).abs() < 1e-9);
        widget.quota_value = Some(QuotaValue::Remaining);
        assert_eq!(text(&widget), "5h · 70% residuo");
        assert!((widget.meter_fraction(&quota) - 0.7).abs() < 1e-9);
        widget.compact_quotas = true;
        widget.quota_value = Some(QuotaValue::Used);
        assert_eq!(text(&widget), "5h · 30% utilizzato");
        assert!((widget.meter_fraction(&quota) - 0.3).abs() < 1e-9);
        widget.quota_value = None;
        assert_eq!(text(&widget), "5h · 70% residuo");
    }
}
