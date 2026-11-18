//! Bounded, incremental readers of local agent transcripts. No credential access.
use crate::provider::Provider;
use bezel_core::domain::{
    agent::{AgentKind, AgentQuota, AgentSession, AgentState},
    sensor::{SensorInfo, Snapshot, Wanted},
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const CHUNK: u64 = 1024 * 1024;
const MAX_SESSIONS: usize = 40;

fn unix(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn now() -> u64 {
    unix(SystemTime::now())
}
fn string(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}
fn bounded(text: &str, length: usize) -> String {
    text.chars().take(length).collect()
}

// Windows can retain a transcript's old modification time until its writer
// closes the handle. The timestamp in a complete event stays current.
fn event_time(value: &Value) -> Option<u64> {
    let at = chrono::DateTime::parse_from_rfc3339(value["timestamp"].as_str()?).ok()?;
    u64::try_from(at.timestamp()).ok()
}

pub(crate) struct AgentProvider {
    enabled: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    latest: Arc<Mutex<Vec<AgentSession>>>,
    worker: Option<std::thread::JoinHandle<()>>,
    claude_enabled: Arc<AtomicBool>,
    usage_worker: Option<std::thread::JoinHandle<()>>,
}

impl Default for AgentProvider {
    fn default() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            latest: Arc::default(),
            worker: None,
            claude_enabled: Arc::new(AtomicBool::new(false)),
            usage_worker: None,
        }
    }
}

impl Drop for AgentProvider {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if let Some(worker) = self.usage_worker.take() {
            let _ = worker.join();
        }
    }
}

impl Provider for AgentProvider {
    fn catalog(&self) -> Vec<SensorInfo> {
        vec![]
    }
    fn want(&mut self, wanted: &Wanted) {
        let enabled = wanted.contains("agent.sessions");
        self.enabled.store(enabled, Ordering::Relaxed);
        self.claude_enabled
            .store(wanted.contains("agent.claude"), Ordering::Relaxed);
        if enabled && self.usage_worker.is_none() {
            let enabled = Arc::clone(&self.claude_enabled);
            let stop = Arc::clone(&self.stop);
            self.usage_worker = Some(std::thread::spawn(move || {
                let mut next = Instant::now();
                while !stop.load(Ordering::Relaxed) {
                    if enabled.load(Ordering::Relaxed) && Instant::now() >= next {
                        if let Err(error) = crate::claude_usage::refresh(&stop) {
                            tracing::warn!(%error, "Claude account usage refresh failed; keeping last valid windows");
                        }
                        next = Instant::now() + Duration::from_secs(60);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }));
        }
        if enabled && self.worker.is_none() {
            let (enabled, stop, latest) = (
                Arc::clone(&self.enabled),
                Arc::clone(&self.stop),
                Arc::clone(&self.latest),
            );
            self.worker = Some(std::thread::spawn(move || {
                let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) else {
                    return;
                };
                let codex = std::env::var_os("CODEX_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".codex"));
                let claude = std::env::var_os("CLAUDE_CONFIG_DIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| home.join(".claude"));
                let mut reader = TranscriptReader::default();
                let mut next = Instant::now();
                while !stop.load(Ordering::Relaxed) {
                    if enabled.load(Ordering::Relaxed) && Instant::now() >= next {
                        let sessions = reader.scan(&codex, &claude, usage_dir().as_deref());
                        *latest.lock().unwrap_or_else(PoisonError::into_inner) = sessions;
                        next = Instant::now() + Duration::from_secs(3);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }));
        }
    }
    fn sample(&mut self, _: Instant, out: &mut Snapshot) {
        if self.enabled.load(Ordering::Relaxed) {
            out.agents = self
                .latest
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone();
            out.agents_sampled_at = now();
        }
    }
}

fn usage_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Bezel Evo").join("agents"))
}

#[derive(Default)]
struct TranscriptReader {
    files: BTreeMap<PathBuf, Transcript>,
}
struct Transcript {
    session: AgentSession,
    offset: u64,
    pending: Vec<u8>,
    modified: SystemTime,
    last_event_at: Option<u64>,
}

/// Traverse only the transcript directory depth. Claude subagents are excluded.
fn logs(
    root: &Path,
    depth: usize,
    agent: AgentKind,
    out: &mut Vec<(SystemTime, PathBuf, AgentKind)>,
) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonl") {
            if let Ok(meta) = entry.metadata() {
                out.push((meta.modified().unwrap_or(UNIX_EPOCH), path, agent));
            }
        } else if depth > 0 && entry.file_type().is_ok_and(|t| t.is_dir()) {
            logs(&path, depth - 1, agent, out);
        }
    }
}

impl TranscriptReader {
    fn scan(&mut self, codex: &Path, claude: &Path, usage: Option<&Path>) -> Vec<AgentSession> {
        let mut found = Vec::new();
        logs(&codex.join("sessions"), 3, AgentKind::Codex, &mut found);
        logs(&claude.join("projects"), 1, AgentKind::Claude, &mut found);
        found.sort_by_key(|(at, _, _)| std::cmp::Reverse(*at));
        // Limit each agent independently so one busy history cannot hide the other.
        let mut counts = [0, 0];
        found.retain(|(_, _, agent)| {
            let index = usize::from(*agent == AgentKind::Claude);
            counts[index] += 1;
            counts[index] <= MAX_SESSIONS
        });
        self.files
            .retain(|path, _| found.iter().any(|(_, p, _)| p == path));
        for (modified, path, agent) in found {
            let Ok(meta) = fs::metadata(&path) else {
                continue;
            };
            if self.files.get(&path).is_some_and(|t| {
                meta.len() < t.offset || (meta.len() == t.offset && modified != t.modified)
            }) {
                self.files.remove(&path);
            }
            let transcript = self
                .files
                .entry(path.clone())
                .or_insert_with(|| Transcript::new(&path, agent, modified, meta.len()));
            if transcript.offset < meta.len() {
                transcript.read(&path, modified);
            }
            if agent == AgentKind::Claude {
                transcript.read_usage(usage);
            }
        }
        let mut sessions: Vec<_> = self
            .files
            .values()
            .map(|t| t.session.clone())
            .filter(|s| !s.id.is_empty())
            .collect();
        if let Some(usage) = crate::claude_usage::cached(claude) {
            for session in sessions
                .iter_mut()
                .filter(|s| s.agent == AgentKind::Claude && s.quota_updated_at <= usage.at)
            {
                session.five_hour = usage.five_hour();
                session.weekly = usage.weekly();
                session.quota_updated_at = usage.at;
            }
        }
        // Usage belongs to the account represented by each agent's local home.
        let quotas: Vec<_> = [AgentKind::Codex, AgentKind::Claude]
            .into_iter()
            .filter_map(|agent| {
                sessions
                    .iter()
                    .filter(|s| s.agent == agent && s.quota_updated_at > 0)
                    .max_by_key(|s| s.quota_updated_at)
                    .map(|s| {
                        (
                            agent,
                            s.five_hour.clone(),
                            s.weekly.clone(),
                            s.quota_updated_at,
                        )
                    })
            })
            .collect();
        for session in &mut sessions {
            if let Some((_, five_hour, weekly, updated_at)) =
                quotas.iter().find(|(agent, ..)| *agent == session.agent)
            {
                session.five_hour = five_hour.clone();
                session.weekly = weekly.clone();
                session.quota_updated_at = *updated_at;
            }
            if session.state == AgentState::Working
                && now().saturating_sub(session.updated_at) > 300
            {
                session.state = AgentState::Unknown;
            }
        }
        sessions.sort_by_key(|s| std::cmp::Reverse(s.updated_at));
        sessions
    }
}

impl Transcript {
    fn new(path: &Path, agent: AgentKind, modified: SystemTime, length: u64) -> Self {
        let mut transcript = Self {
            session: AgentSession {
                agent,
                id: path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                updated_at: unix(modified),
                ..Default::default()
            },
            offset: 0,
            pending: Vec::new(),
            modified,
            last_event_at: None,
        };
        // Read the first metadata record even when the history is very large.
        if let Ok(file) = File::open(path) {
            let mut head = Vec::new();
            let _ = file.take(32 * 1024).read_to_end(&mut head);
            if let Some(end) = head.iter().rposition(|b| *b == b'\n') {
                for line in head[..end].split(|b| *b == b'\n') {
                    transcript.record(line);
                }
            }
        }
        if length > CHUNK {
            transcript.offset = length - CHUNK;
            // Starting inside a record is safe: discard that first partial line.
            if let Ok(mut file) = File::open(path) {
                let _ = file.seek(SeekFrom::Start(transcript.offset));
                let mut tail = Vec::new();
                let _ = file.take(CHUNK).read_to_end(&mut tail);
                if let Some(end) = tail.iter().position(|b| *b == b'\n') {
                    transcript.offset += end as u64 + 1;
                } else {
                    transcript.offset = length;
                }
            }
        }
        transcript
    }
    fn read(&mut self, path: &Path, modified: SystemTime) {
        self.modified = modified;
        let Ok(mut file) = File::open(path) else {
            return;
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return;
        }
        let mut bytes = Vec::new();
        if file.take(CHUNK * 2).read_to_end(&mut bytes).is_err() {
            return;
        }
        self.offset += bytes.len() as u64;
        self.pending.extend(bytes);
        let Some(end) = self.pending.iter().rposition(|b| *b == b'\n') else {
            if self.pending.len() > CHUNK as usize * 2 {
                self.pending.clear();
            }
            return;
        };
        let complete: Vec<_> = self.pending.drain(..=end).collect();
        for line in complete.split(|b| *b == b'\n') {
            self.record(line);
        }
        self.modified = modified;
        self.session.updated_at = self.last_event_at.unwrap_or_else(|| unix(modified));
    }
    fn record(&mut self, line: &[u8]) {
        let Ok(v) = serde_json::from_slice::<Value>(line) else {
            return;
        };
        if v["isSidechain"] == true {
            return;
        }
        if let Some(at) = event_time(&v) {
            self.last_event_at = Some(self.last_event_at.map_or(at, |previous| previous.max(at)));
        }
        match self.session.agent {
            AgentKind::Codex => self.codex(&v),
            AgentKind::Claude => self.claude(&v),
        }
    }
    fn codex(&mut self, v: &Value) {
        let p = &v["payload"];
        if v["type"] == "session_meta" {
            self.session.id = string(p, "id");
            self.session.project_path = string(p, "cwd");
            self.session.project = project(&self.session.project_path);
        }
        if v["type"] == "event_msg" {
            match p["type"].as_str().unwrap_or_default() {
                "task_started" | "turn_started" | "user_message" => {
                    self.session.state = AgentState::Working;
                    self.session.activity.clear();
                }
                "task_complete" | "turn_complete" => {
                    self.session.state = AgentState::Completed;
                    self.session.activity.clear();
                }
                "turn_aborted" => {
                    self.session.state = AgentState::Waiting;
                    self.session.activity.clear();
                }
                "error" => self.session.state = AgentState::Error,
                "agent_message" => {
                    self.session.message = bounded(&string(p, "message"), 8192);
                    if p["phase"] == "final_answer" {
                        self.session.state = AgentState::Completed;
                        self.session.activity.clear();
                    }
                }
                "token_count" => {
                    let limits = &p["rate_limits"];
                    for window in ["primary", "secondary"] {
                        let q = &limits[window];
                        match q["window_minutes"].as_u64() {
                            Some(300) => {
                                self.session.five_hour = quota(q, "used_percent", "resets_at")
                            }
                            Some(10080) => {
                                self.session.weekly = quota(q, "used_percent", "resets_at")
                            }
                            _ => {}
                        }
                    }
                    if self.session.five_hour.is_some() || self.session.weekly.is_some() {
                        self.session.quota_updated_at =
                            event_time(v).unwrap_or_else(|| unix(self.modified));
                    }
                }
                _ => {}
            }
        }
        if v["type"] == "response_item" {
            match p["type"].as_str().unwrap_or_default() {
                "function_call" | "custom_tool_call" => {
                    self.session.state = AgentState::Working;
                    self.session.activity = bounded(&string(p, "name"), 160);
                    if self.session.activity.ends_with("request_user_input") {
                        self.session.state = AgentState::Waiting;
                    }
                }
                "message" if p["role"] == "assistant" => {
                    if let Some(parts) = p["content"].as_array() {
                        let text = parts
                            .iter()
                            .filter_map(|part| part["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !text.is_empty() {
                            self.session.message = bounded(&text, 8192);
                        }
                    }
                    if p["phase"] == "final_answer" {
                        self.session.state = AgentState::Completed;
                        self.session.activity.clear();
                    }
                }
                _ => {}
            }
        }
    }
    fn claude(&mut self, v: &Value) {
        if let Some(id) = v["sessionId"].as_str() {
            self.session.id = bounded(id, 128);
        }
        if let Some(cwd) = v["cwd"].as_str()
            && self.session.project_path.is_empty()
        {
            self.session.project_path = bounded(cwd, 4096);
            self.session.project = project(cwd);
        }
        match v["type"].as_str().unwrap_or_default() {
            "user" => {
                self.session.state = AgentState::Working;
                self.session.activity.clear();
            }
            "assistant" => {
                let m = &v["message"];
                if let Some(parts) = m["content"].as_array() {
                    let text = parts
                        .iter()
                        .filter(|p| p["type"] == "text")
                        .filter_map(|p| p["text"].as_str())
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !text.is_empty() {
                        self.session.message = bounded(&text, 8192);
                    }
                    for part in parts.iter().filter(|p| p["type"] == "tool_use") {
                        self.session.activity = bounded(&string(part, "name"), 160);
                        self.session.state = match self.session.activity.as_str() {
                            "AskUserQuestion" | "ExitPlanMode" => AgentState::Waiting,
                            _ => AgentState::Working,
                        };
                    }
                }
                if m["stop_reason"] == "end_turn" {
                    self.session.state = AgentState::Completed;
                    self.session.activity.clear();
                }
            }
            "system" if v["subtype"] == "turn_duration" => {
                self.session.state = AgentState::Completed;
                self.session.activity.clear();
            }
            _ => {}
        }
    }
    fn read_usage(&mut self, usage: Option<&Path>) {
        if !self
            .session
            .id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return;
        }
        let Some(root) = usage else {
            return;
        };
        let Ok(file) = File::open(root.join(format!("{}.json", self.session.id))) else {
            return;
        };
        let modified = file
            .metadata()
            .and_then(|m| m.modified())
            .map(unix)
            .unwrap_or_default();
        let Ok(v) = serde_json::from_reader::<_, Value>(file.take(16384)) else {
            return;
        };
        self.session.five_hour = quota(
            &v["rate_limits"]["five_hour"],
            "used_percentage",
            "resets_at",
        );
        self.session.weekly = quota(
            &v["rate_limits"]["seven_day"],
            "used_percentage",
            "resets_at",
        );
        self.session.quota_updated_at = modified;
    }
}

fn project(cwd: &str) -> String {
    bounded(
        cwd.trim_end_matches(['/', '\\'])
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or_default(),
        160,
    )
}
fn quota(value: &Value, used: &str, reset: &str) -> Option<AgentQuota> {
    let used_percent = value[used].as_f64()?;
    if !used_percent.is_finite() || !(0.0..=100.0).contains(&used_percent) {
        return None;
    }
    Some(AgentQuota {
        used_percent,
        resets_at: value[reset].as_u64()?,
    })
}

/// Installs a status-line bridge, preserving an existing command and a backup.
pub fn connect_claude() -> std::io::Result<()> {
    let home = std::env::var_os("USERPROFILE")
        .ok_or_else(|| std::io::Error::other("USERPROFILE unavailable"))?;
    let root = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join(".claude"));
    install_statusline(
        &root,
        &usage_dir().ok_or_else(|| std::io::Error::other("LOCALAPPDATA unavailable"))?,
    )
}

/// Refreshes subscription quotas through the signed-in native Claude client.
pub fn refresh_claude_usage() -> std::io::Result<()> {
    crate::claude_usage::refresh(&AtomicBool::new(false))
}

fn install_statusline(root: &Path, output: &Path) -> std::io::Result<()> {
    fs::create_dir_all(root)?;
    fs::create_dir_all(output)?;
    let settings_path = root.join("settings.json");
    let original = match fs::read(&settings_path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    let mut settings: Value = match &original {
        Some(bytes) => serde_json::from_slice(bytes)?,
        None => serde_json::json!({}),
    };
    if !settings.is_object() {
        return Err(std::io::Error::other(
            "Claude settings must be a JSON object",
        ));
    }
    let script_path = output.join("claude-statusline.ps1");
    let command = format!(
        "powershell.exe -NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File \"{}\"",
        script_path.display()
    );
    if settings["statusLine"]["command"]
        .as_str()
        .is_some_and(|current| {
            current == command || current.starts_with(&format!("{command} -PassThrough | ("))
        })
    {
        return Ok(());
    }
    let previous = settings["statusLine"]["command"]
        .as_str()
        .unwrap_or_default();
    let script = format!(
        "\u{feff}param([switch]$PassThrough)\r\n$bezelOutput = '{}'\r\n{}",
        output.display().to_string().replace('\'', "''"),
        include_str!("agent-statusline.ps1")
    );
    // The original status command continues to run through Claude's own shell.
    let command = if previous.is_empty() {
        command
    } else {
        format!("{command} -PassThrough | ({previous})")
    };
    write_atomic(&script_path, script.as_bytes())?;
    let backup_path = root.join(format!("settings.bezel-agent-{}.json.bak", now()));
    if let Some(bytes) = &original {
        File::options()
            .write(true)
            .create_new(true)
            .open(backup_path)?
            .write_all(bytes)?;
    }
    let mut status = settings["statusLine"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    status.insert("type".into(), Value::String("command".into()));
    status.insert("command".into(), Value::String(command));
    settings["statusLine"] = Value::Object(status);
    // Do not overwrite a concurrent change to the user's configuration.
    if fs::read(&settings_path).ok() != original {
        return Err(std::io::Error::other(
            "Claude settings changed during setup; retry",
        ));
    }
    write_atomic(&settings_path, &serde_json::to_vec_pretty(&settings)?)
}

use std::io::Write;

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("missing parent directory"))?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn explicit_question_tools_report_waiting_for_the_user() {
        let mut transcript =
            Transcript::new(Path::new("missing"), AgentKind::Claude, UNIX_EPOCH, 0);
        transcript.record(json!({"type":"assistant","message":{"content":[{"type":"tool_use","name":"AskUserQuestion"}]}}).to_string().as_bytes());
        assert_eq!(transcript.session.state, AgentState::Waiting);
        transcript.session.agent = AgentKind::Codex;
        transcript.record(json!({"type":"response_item","payload":{"type":"function_call","name":"request_user_input"}}).to_string().as_bytes());
        assert_eq!(transcript.session.state, AgentState::Waiting);
    }

    fn append(path: &Path, record: Value) {
        let mut file = File::options()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        writeln!(file, "{record}").unwrap();
    }

    #[test]
    fn event_timestamps_keep_open_sessions_current_with_old_windows_file_times() {
        let temp = tempfile::tempdir().unwrap();
        let codex = temp.path().join("codex");
        let folder = codex.join("sessions/2026/11/16");
        fs::create_dir_all(&folder).unwrap();
        let current = now();
        let timestamp = |at: u64| {
            chrono::DateTime::from_timestamp(at as i64, 0)
                .unwrap()
                .to_rfc3339()
        };
        for (id, event_at, file_at) in [
            ("open", current, current - 3600),
            ("closed", current - 1800, current),
        ] {
            let path = folder.join(format!("{id}.jsonl"));
            append(
                &path,
                json!({"type":"session_meta","payload":{"id":id,"cwd":"F:/dev/bezel"}}),
            );
            append(
                &path,
                json!({"timestamp":timestamp(event_at),"type":"event_msg","payload":{"type":"task_started"}}),
            );
            append(
                &path,
                json!({"timestamp":timestamp(event_at),"type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":35,"window_minutes":300,"resets_at":current+3600}}}}),
            );
            File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_times(
                    std::fs::FileTimes::new()
                        .set_modified(UNIX_EPOCH + Duration::from_secs(file_at)),
                )
                .unwrap();
        }
        let mut reader = TranscriptReader::default();
        let sessions = reader.scan(&codex, &temp.path().join("claude"), None);
        assert_eq!(sessions[0].id, "open");
        assert_eq!(sessions[0].updated_at, current);
        assert_eq!(sessions[0].quota_updated_at, current);
        assert_eq!(sessions[0].state, AgentState::Working);
        let sessions = reader.scan(&codex, &temp.path().join("claude"), None);
        assert_eq!(
            sessions[0].updated_at, current,
            "a cached read preserves event time"
        );
        assert!(event_time(&json!({"timestamp":"invalid"})).is_none());
    }

    #[test]
    #[ignore = "reads the current user's local agent transcripts"]
    fn local_agent_discovery_probe() {
        let home = PathBuf::from(std::env::var_os("USERPROFILE").unwrap());
        let codex = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".codex"));
        let claude = std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".claude"));
        let sessions = TranscriptReader::default().scan(&codex, &claude, usage_dir().as_deref());
        assert!(!sessions.is_empty(), "no local transcripts found");
        for agent in [AgentKind::Codex, AgentKind::Claude] {
            let matching: Vec<_> = sessions.iter().filter(|s| s.agent == agent).collect();
            println!(
                "{}: {} sessions, {} messages, {} quota windows",
                agent.name(),
                matching.len(),
                matching.iter().filter(|s| !s.message.is_empty()).count(),
                matching
                    .iter()
                    .map(|s| usize::from(s.five_hour.is_some()) + usize::from(s.weekly.is_some()))
                    .sum::<usize>()
            );
        }
    }

    #[test]
    fn transcripts_follow_appends_and_skip_partial_records_and_reasoning() {
        let temp = tempfile::tempdir().unwrap();
        let codex = temp.path().join("codex");
        let claude = temp.path().join("claude");
        let folder = codex.join("sessions/2026/11/16");
        fs::create_dir_all(&folder).unwrap();
        let path = folder.join("rollout.jsonl");
        append(
            &path,
            json!({"type":"session_meta","payload":{"id":"codex-1","cwd":"F:\\dev\\bezel"}}),
        );
        append(
            &path,
            json!({"type":"event_msg","payload":{"type":"task_started"}}),
        );
        append(
            &path,
            json!({"type":"response_item","payload":{"type":"reasoning","summary":[{"text":"private"}]}}),
        );
        append(
            &path,
            json!({"type":"event_msg","payload":{"type":"agent_message","message":"Checking tests","phase":"commentary"}}),
        );
        append(
            &path,
            json!({"type":"event_msg","payload":{"type":"token_count","rate_limits":{"primary":{"used_percent":35,"window_minutes":300,"resets_at":now()+3600},"secondary":{"used_percent":62,"window_minutes":10080,"resets_at":now()+86400}}}}),
        );
        let mut reader = TranscriptReader::default();
        let sessions = reader.scan(&codex, &claude, None);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].project, "bezel");
        assert_eq!(sessions[0].message, "Checking tests");
        assert_eq!(sessions[0].state, AgentState::Working);
        assert_eq!(sessions[0].weekly.as_ref().unwrap().used_percent, 62.);
        let mut file = File::options().append(true).open(&path).unwrap();
        write!(
            file,
            "{{\"type\":\"event_msg\",\"payload\":{{\"type\":\"task_complete\""
        )
        .unwrap();
        drop(file);
        assert_eq!(
            reader.scan(&codex, &claude, None)[0].state,
            AgentState::Working
        );
        let mut file = File::options().append(true).open(&path).unwrap();
        writeln!(file, "}}}}").unwrap();
        drop(file);
        assert_eq!(
            reader.scan(&codex, &claude, None)[0].state,
            AgentState::Completed
        );
        // A truncated/replaced transcript must not reuse old messages or usage.
        fs::write(&path, b"{\"type\":\"session_meta\",\"payload\":{\"id\":\"replacement\",\"cwd\":\"F:/other\"}}\n").unwrap();
        let sessions = reader.scan(&codex, &claude, None);
        assert_eq!(sessions[0].id, "replacement");
        assert!(sessions[0].message.is_empty());
        assert!(sessions[0].five_hour.is_none());
    }

    #[test]
    fn claude_transcripts_use_visible_text_and_statusline_quotas() {
        let temp = tempfile::tempdir().unwrap();
        let claude = temp.path().join("claude");
        let folder = claude.join("projects/f--dev-bezel");
        let usage = temp.path().join("usage");
        fs::create_dir_all(&folder).unwrap();
        fs::create_dir_all(&usage).unwrap();
        let path = folder.join("claude-1.jsonl");
        append(
            &path,
            json!({"type":"assistant","sessionId":"claude-1","cwd":"F:/dev/bezel","message":{"content":[{"type":"thinking","thinking":"private"},{"type":"text","text":"Running tests"},{"type":"tool_use","name":"Bash"}]}}),
        );
        fs::write(
            usage.join("claude-1.json"),
            json!({"rate_limits":{"five_hour":{"used_percentage":23.5,"resets_at":now()+3600}}})
                .to_string(),
        )
        .unwrap();
        let mut reader = TranscriptReader::default();
        let sessions = reader.scan(&temp.path().join("codex"), &claude, Some(&usage));
        assert_eq!(sessions[0].agent, AgentKind::Claude);
        assert_eq!(sessions[0].message, "Running tests");
        assert_eq!(sessions[0].activity, "Bash");
        assert_eq!(sessions[0].five_hour.as_ref().unwrap().used_percent, 23.5);
        append(
            &path,
            json!({"type":"assistant","isSidechain":true,"sessionId":"child","message":{"content":[{"type":"text","text":"child"}]}}),
        );
        append(
            &path,
            json!({"type":"assistant","sessionId":"claude-1","message":{"content":[{"type":"text","text":"Done"}],"stop_reason":"end_turn"}}),
        );
        let sessions = reader.scan(&temp.path().join("codex"), &claude, Some(&usage));
        assert_eq!(sessions[0].id, "claude-1");
        assert_eq!(sessions[0].state, AgentState::Completed);
        assert_eq!(sessions[0].message, "Done");
        assert!(sessions[0].activity.is_empty());
    }

    #[test]
    fn statusline_setup_preserves_command_settings_and_backup_and_is_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("claude");
        let output = temp.path().join("usage");
        fs::create_dir_all(&root).unwrap();
        let original = json!({"model":"opus","statusLine":{"type":"command","command":"findstr session_id","padding":2},"permissions":{"allow":["Read"]}}).to_string();
        fs::write(root.join("settings.json"), &original).unwrap();
        install_statusline(&root, &output).unwrap();
        install_statusline(&root, &output).unwrap();
        let settings: Value =
            serde_json::from_slice(&fs::read(root.join("settings.json")).unwrap()).unwrap();
        assert_eq!(settings["model"], "opus");
        assert_eq!(settings["statusLine"]["padding"], 2);
        assert_eq!(settings["permissions"]["allow"][0], "Read");
        let backups: Vec<_> = fs::read_dir(&root)
            .unwrap()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "bak"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read_to_string(backups[0].path()).unwrap(), original);
        assert!(
            settings["statusLine"]["command"]
                .as_str()
                .unwrap()
                .ends_with("-PassThrough | (findstr session_id)")
        );
        // Execute the real bridge twice, including the replacement path on Windows.
        for used in [25, 42] {
            use std::os::windows::process::CommandExt;
            use std::process::{Command, Stdio};
            let mut child = Command::new(std::env::var_os("ComSpec").unwrap())
                .creation_flags(0x08000000)
                .args(["/d", "/s", "/c"])
                .raw_arg(settings["statusLine"]["command"].as_str().unwrap())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let payload = json!({"session_id":"session-1","transcript_path":"must-not-be-exported","rate_limits":{"five_hour":{"used_percentage":used,"resets_at":now()+3600}}});
            child
                .stdin
                .take()
                .unwrap()
                .write_all(payload.to_string().as_bytes())
                .unwrap();
            let result = child.wait_with_output().unwrap();
            assert!(result.status.success());
            assert!(String::from_utf8_lossy(&result.stdout).contains("must-not-be-exported"));
            let cached: Value =
                serde_json::from_slice(&fs::read(output.join("session-1.json")).unwrap()).unwrap();
            assert_eq!(cached["rate_limits"]["five_hour"]["used_percentage"], used);
            assert!(cached.get("transcript_path").is_none());
        }
    }
}
