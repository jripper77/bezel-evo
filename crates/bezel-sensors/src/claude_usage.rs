//! Read account usage through the signed-in Claude executable's control protocol.
//! No prompt is submitted. Credentials remain inside Claude's own client.
use bezel_core::domain::agent::AgentQuota;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::os::windows::process::CommandExt;
use std::{
    fs,
    io::{self, BufRead, BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Usage {
    pub at: u64,
    home: String,
    five_hour: Option<Window>,
    weekly: Option<Window>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Window {
    used: f64,
    reset: u64,
}

impl Usage {
    pub fn five_hour(&self) -> Option<AgentQuota> {
        self.five_hour.as_ref().map(|q| AgentQuota {
            used_percent: q.used,
            resets_at: q.reset,
        })
    }
    pub fn weekly(&self) -> Option<AgentQuota> {
        self.weekly.as_ref().map(|q| AgentQuota {
            used_percent: q.used,
            resets_at: q.reset,
        })
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn config_home() -> io::Result<PathBuf> {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".claude")))
        .ok_or_else(|| io::Error::other("Claude configuration directory unavailable"))
}
fn normalized(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}
fn cache_path() -> io::Result<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|p| PathBuf::from(p).join("Bezel Evo/agents/claude-sdk-usage.json"))
        .ok_or_else(|| io::Error::other("LOCALAPPDATA unavailable"))
}

pub(crate) fn cached(home: &Path) -> Option<Usage> {
    read_cache(&cache_path().ok()?, home, now())
}

fn read_cache(path: &Path, home: &Path, at: u64) -> Option<Usage> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() > 16384 {
        return None;
    }
    let mut usage: Usage = serde_json::from_slice(&bytes).ok()?;
    if usage.home != normalized(home) || at.saturating_sub(usage.at) >= 900 {
        return None;
    }
    usage.five_hour = usage
        .five_hour
        .filter(|q| q.reset > at && q.used.is_finite() && (0.0..=100.).contains(&q.used));
    usage.weekly = usage
        .weekly
        .filter(|q| q.reset > at && q.used.is_finite() && (0.0..=100.).contains(&q.used));
    (usage.five_hour.is_some() || usage.weekly.is_some()).then_some(usage)
}

#[derive(Serialize, Deserialize)]
struct Attempt {
    at: u64,
    home: String,
}

// Shared by Studio, Light and all screens: one attempt per minute, including
// failures. The lock is released automatically even if the caller exits early.
fn claim_attempt(path: &Path, home: &Path, at: u64) -> io::Result<Option<fs::File>> {
    use std::fs::TryLockError;
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Ok(None),
        Err(TryLockError::Error(error)) => return Err(error),
    }
    let mut bytes = Vec::new();
    (&mut file).take(4096).read_to_end(&mut bytes)?;
    if let Ok(previous) = serde_json::from_slice::<Attempt>(&bytes)
        && previous.home == normalized(home)
        && at >= previous.at
        && at - previous.at < 60
    {
        return Ok(None);
    }
    file.set_len(0)?;
    file.rewind()?;
    serde_json::to_writer(
        &mut file,
        &Attempt {
            at,
            home: normalized(home),
        },
    )?;
    file.sync_all()?;
    Ok(Some(file))
}

fn update_cache(path: &Path, response: &Value, home: &Path, at: u64) -> io::Result<()> {
    if !response["rate_limits_available"].as_bool().unwrap_or(false) {
        return Err(io::Error::other("Claude account usage unavailable"));
    }
    let limits = &response["rate_limits"];
    let mut usage = Usage {
        at,
        home: normalized(home),
        five_hour: window(&limits["five_hour"]).filter(|q| q.reset > at),
        weekly: window(&limits["seven_day"]).filter(|q| q.reset > at),
    };
    if usage.five_hour.is_none() && usage.weekly.is_none() {
        return Err(io::Error::other("Claude returned no account usage windows"));
    }
    if let Some(previous) = read_cache(path, home, at) {
        let retained = (usage.five_hour.is_none() && previous.five_hour.is_some())
            || (usage.weekly.is_none() && previous.weekly.is_some());
        usage.five_hour = usage.five_hour.or(previous.five_hour);
        usage.weekly = usage.weekly.or(previous.weekly);
        // A single observation timestamp must not make a retained window fresh.
        if retained {
            usage.at = previous.at;
        }
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Missing usage cache directory"))?;
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer(&mut file, &usage)?;
    file.as_file().sync_all()?;
    file.persist(path).map_err(|e| e.error)?;
    Ok(())
}

fn executable() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("CLAUDE_CODE_EXECUTABLE")
        .map(PathBuf::from)
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")))
    {
        return Ok(path);
    }
    let home = PathBuf::from(
        std::env::var_os("USERPROFILE")
            .ok_or_else(|| io::Error::other("USERPROFILE unavailable"))?,
    );
    let mut candidates = Vec::new();
    for editor in [".vscode", ".vscode-insiders"] {
        if let Ok(entries) = fs::read_dir(home.join(editor).join("extensions")) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if let Some(version) = name.strip_prefix("anthropic.claude-code-") {
                    let version: Vec<u32> = version
                        .split('-')
                        .next()
                        .unwrap_or_default()
                        .split('.')
                        .filter_map(|part| part.parse().ok())
                        .collect();
                    let path = entry.path().join("resources/native-binary/claude.exe");
                    if path.is_file() {
                        candidates.push((version, path));
                    }
                }
            }
        }
    }
    if let Some((_, path)) = candidates.into_iter().max_by(|a, b| a.0.cmp(&b.0)) {
        return Ok(path);
    }
    let local = home.join(".local/bin/claude.exe");
    if local.is_file() {
        return Ok(local);
    }
    Err(io::Error::other("Claude native executable unavailable"))
}

struct Client {
    child: Child,
    responses: Option<Receiver<Value>>,
    reader: Option<JoinHandle<()>>,
    _workspace: tempfile::TempDir,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.child.stdin.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.responses.take();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

impl Client {
    fn start() -> io::Result<Self> {
        let workspace = tempfile::Builder::new()
            .prefix("bezel-claude-usage-")
            .tempdir()?;
        let child = Command::new(executable()?)
            .creation_flags(0x08000000)
            .args([
                "--print",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--verbose",
                "--no-session-persistence",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
                "--tools",
                "",
                "--setting-sources",
                "",
                "--disable-slash-commands",
            ])
            .current_dir(workspace.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut client = Self {
            child,
            responses: None,
            reader: None,
            _workspace: workspace,
        };
        let stdout = client
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Claude stdout unavailable"))?;
        let (sender, receiver) = mpsc::sync_channel(16);
        client.responses = Some(receiver);
        client.reader = Some(
            std::thread::Builder::new()
                .name("claude-usage-output".into())
                .spawn(move || {
                    use std::io::Read;
                    let mut reader = BufReader::new(stdout);
                    loop {
                        let mut line = Vec::new();
                        let read = reader
                            .by_ref()
                            .take(1024 * 1024)
                            .read_until(b'\n', &mut line);
                        if !matches!(read, Ok(n) if n > 0) {
                            break;
                        }
                        if let Ok(value) = serde_json::from_slice::<Value>(&line)
                            && sender.send(value).is_err()
                        {
                            break;
                        }
                    }
                })?,
        );
        Ok(client)
    }

    fn request(&mut self, id: &str, request: Value, stop: &AtomicBool) -> io::Result<Value> {
        let input = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::other("Claude stdin unavailable"))?;
        serde_json::to_writer(
            &mut *input,
            &json!({"type":"control_request", "request_id":id, "request":request}),
        )?;
        input.write_all(b"\n")?;
        input.flush()?;
        let responses = self
            .responses
            .as_ref()
            .ok_or_else(|| io::Error::other("Claude response channel unavailable"))?;
        let deadline = Instant::now() + Duration::from_secs(20);
        while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
            match responses.recv_timeout(Duration::from_millis(100)) {
                Ok(value)
                    if value["type"] == "control_response"
                        && value["response"]["request_id"] == id =>
                {
                    if value["response"]["subtype"] != "success" {
                        return Err(io::Error::other("Claude usage control request unavailable"));
                    }
                    return Ok(value["response"]["response"].clone());
                }
                Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::other("Claude usage client exited"));
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Claude usage request timed out or cancelled",
        ))
    }
}

fn window(value: &Value) -> Option<Window> {
    let used = value["utilization"].as_f64()?;
    if !used.is_finite() || !(0.0..=100.0).contains(&used) {
        return None;
    }
    let reset = chrono::DateTime::parse_from_rfc3339(value["resets_at"].as_str()?)
        .ok()?
        .timestamp();
    Some(Window {
        used,
        reset: u64::try_from(reset).ok()?,
    })
}

pub(crate) fn refresh(stop: &AtomicBool) -> io::Result<()> {
    if stop.load(Ordering::Relaxed) {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "Claude usage read cancelled",
        ));
    }
    let home = config_home()?;
    let path = cache_path()?;
    let at = now();
    if read_cache(&path, &home, at).is_some_and(|usage| at.saturating_sub(usage.at) < 60) {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("Missing usage cache directory"))?;
    fs::create_dir_all(parent)?;
    let Some(_attempt) = claim_attempt(&path.with_extension("refresh-lock"), &home, at)? else {
        return if read_cache(&path, &home, at).is_some() {
            Ok(())
        } else {
            Err(io::Error::other(
                "Claude usage refresh already requested; retry in a minute",
            ))
        };
    };
    let mut client = Client::start()?;
    client.request("bezel-init", json!({"subtype":"initialize"}), stop)?;
    let response = client.request(
        "bezel-usage",
        json!({"subtype":"get_usage", "skip_behaviors":true}),
        stop,
    )?;
    update_cache(&path, &response, &home, now())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_and_partial_responses_preserve_valid_cached_windows_without_refreshing_their_age() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("usage.json");
        let home = temp.path().join("claude");
        let response = json!({"rate_limits_available":true,"rate_limits":{
            "five_hour":{"utilization":30,"resets_at":"2030-01-01T00:00:00Z"},
            "seven_day":{"utilization":60,"resets_at":"2030-01-07T00:00:00Z"}
        }});
        update_cache(&path, &response, &home, 100).unwrap();
        let original = fs::read(&path).unwrap();
        assert!(
            update_cache(
                &path,
                &json!({"rate_limits_available":true,"rate_limits":null}),
                &home,
                120
            )
            .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), original);
        let mut partial = response.clone();
        partial["rate_limits"]["seven_day"] = Value::Null;
        partial["rate_limits"]["five_hour"]["utilization"] = json!(45);
        update_cache(&path, &partial, &home, 130).unwrap();
        let cached = read_cache(&path, &home, 400).unwrap();
        assert_eq!(cached.at, 100);
        assert_eq!(cached.five_hour().unwrap().used_percent, 45.);
        assert_eq!(cached.weekly().unwrap().used_percent, 60.);
        assert!(read_cache(&path, &home, 1000).is_none());
        assert!(read_cache(&path, &temp.path().join("another-account"), 400).is_none());
        let mut expired = response;
        expired["rate_limits"]["five_hour"]["resets_at"] = json!("2000-01-01T00:00:00Z");
        expired["rate_limits"]["seven_day"]["resets_at"] = json!("2000-01-01T00:00:00Z");
        assert!(update_cache(&path, &expired, &home, 1794840000).is_err());
    }

    #[test]
    fn refresh_attempts_are_shared_and_back_off_after_failures() {
        let temp = tempfile::TempDir::new().unwrap();
        let path = temp.path().join("refresh.lock");
        let home = temp.path().join("claude");
        let first = claim_attempt(&path, &home, 100).unwrap().unwrap();
        assert!(
            claim_attempt(&path, &home, 161).unwrap().is_none(),
            "a running query owns the lock"
        );
        drop(first);
        assert!(
            claim_attempt(&path, &home, 159).unwrap().is_none(),
            "failed queries still count toward cadence"
        );
        assert!(claim_attempt(&path, &home, 160).unwrap().is_some());
    }

    #[test]
    fn cancelled_reads_do_not_start_an_external_client() {
        assert_eq!(
            refresh(&AtomicBool::new(true)).unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
    }
    #[test]
    fn sdk_windows_use_percentage_and_rfc3339_reset() {
        let value = json!({"utilization":16,"resets_at":"2026-11-16T03:40:00.331256+00:00"});
        let quota = window(&value).unwrap();
        assert_eq!(quota.used, 16.);
        assert_eq!(quota.reset, 1794800400);
        assert!(window(&json!({"utilization":101,"resets_at":"2026-11-16T03:40:00Z"})).is_none());
        assert!(window(&Value::Null).is_none());
    }
    #[test]
    #[ignore = "queries the signed-in Claude account through its own executable"]
    fn live_claude_usage_probe() {
        refresh(&AtomicBool::new(false)).unwrap();
        let usage = cached(&config_home().unwrap()).unwrap();
        assert!(usage.five_hour().is_some() || usage.weekly().is_some());
    }
}
