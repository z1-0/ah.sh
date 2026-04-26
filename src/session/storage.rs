use std::cmp::Ordering;
use std::io::ErrorKind;
use std::path::Path;
use std::time::SystemTime;

use anyhow::{Context, Result};
use fs_err as fs;
use tracing_attributes::instrument;

use crate::path;
use crate::path::cache::sessions::{FLAKE_FILE, HISTORY_FILE, METADATA_FILE};
use crate::provider::get_flake_contents;
use crate::session::types::{HISTORY_LIMIT, Session, SessionKey};
use crate::util::atomic_write;

fn read_history(session_dir: &Path) -> Result<Vec<String>> {
    let history_path = session_dir.join(HISTORY_FILE);
    match fs::read_to_string(&history_path) {
        Ok(content) => {
            let history: Vec<String> = serde_json::from_str(&content)
                .with_context(|| format!("failed to parse history file: {:?}", history_path))?;
            Ok(history)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

fn get_sessions_with_mtime() -> Result<Vec<(Session, SystemTime)>> {
    let session_dir = path::cache::sessions::get_dir();
    let entries = match fs::read_dir(&session_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let sessions: Vec<(Session, SystemTime)> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let session_id = entry.file_name().to_string_lossy().into_owned();
            let session = try_session_by_id(&session_id).ok()?;
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            Some((session, mtime))
        })
        .collect();
    Ok(sessions)
}

#[instrument(skip_all)]
pub fn list_sessions() -> Result<Vec<Session>> {
    let mut sessions = get_sessions_with_mtime()?;

    sessions.sort_by(|(a, a_mtime), (b, b_mtime)| match b_mtime.cmp(a_mtime) {
        Ordering::Equal => a.id.cmp(&b.id),
        other => other,
    });

    Ok(sessions.into_iter().map(|(session, _)| session).collect())
}

#[instrument(skip_all)]
pub fn find_session_by_history() -> Result<Vec<Session>> {
    let cwd = path::get_cwd()?;
    let target_path = cwd.to_string_lossy().into_owned();
    let session_base_dir = path::cache::sessions::get_dir();

    let entries = match fs::read_dir(&session_base_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut matching_ids: Vec<String> = Vec::new();
    let entries: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();

    for entry in entries {
        let history = read_history(&entry.path())?;
        if history.iter().any(|e| e == &target_path) {
            matching_ids.push(entry.file_name().to_string_lossy().into_owned());
        }
    }

    let mut sessions: Vec<Session> = Vec::new();
    for id in matching_ids {
        if let Ok(s) = try_session_by_id(&id) {
            sessions.push(s);
        }
    }
    Ok(sessions)
}

#[instrument(skip_all, fields(session_id = %session.id))]
pub fn save_session(session: &Session) -> Result<()> {
    let session_dir = session.get_dir();
    fs::create_dir_all(&session_dir)?;

    let flake_contents = get_flake_contents(session.provider)(&session.languages)?;
    let flake_path = session_dir.join(FLAKE_FILE);
    atomic_write(&flake_path, &flake_contents)
        .with_context(|| format!("failed to write flake.nix: {:?}", flake_path))?;

    let meta_path = session_dir.join(METADATA_FILE);
    let content = serde_json::to_string_pretty(&session)?;
    atomic_write(&meta_path, &content)
        .with_context(|| format!("failed to write metadata file: {:?}", meta_path))?;
    Ok(())
}

#[instrument(skip_all, fields(session_id = %session_id))]
pub fn remove_session(session_id: &str) -> Result<bool> {
    let session_path = path::cache::sessions::get_dir().join(session_id);
    match fs::remove_dir_all(&session_path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

#[instrument(skip_all)]
pub fn clear_sessions() -> Result<usize> {
    let session_dir = path::cache::sessions::get_dir();
    let mut removed = 0usize;

    let entries = match fs::read_dir(&session_dir) {
        Ok(e) => e,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(e.into()),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        match fs::remove_dir_all(&path) {
            Ok(()) => removed += 1,
            Err(e) if e.kind() == ErrorKind::NotADirectory => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(removed)
}

#[instrument(skip_all, fields(session_id = %session.id))]
pub fn update_history(session: &Session) -> Result<()> {
    let session_dir = session.get_dir();
    let mut history = read_history(&session_dir)?;

    let cwd = path::get_cwd()?;
    let cwd_str = cwd.to_string_lossy().into_owned();
    history.retain(|entry| *entry != cwd_str);
    history.insert(0, cwd_str);
    history.truncate(HISTORY_LIMIT);

    let history_path = session_dir.join(HISTORY_FILE);
    let content = serde_json::to_string_pretty(&history)?;
    atomic_write(&history_path, &content)
        .with_context(|| format!("failed to write history file: {:?}", history_path))?;

    Ok(())
}

#[instrument(skip_all, fields(session_id = %session_id))]
pub(crate) fn try_session_by_id(session_id: &str) -> Result<Session> {
    let session_path = path::cache::sessions::get_dir().join(session_id);
    let meta_path = session_path.join(METADATA_FILE);
    let content = fs::read_to_string(&meta_path)?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse metadata.json at {:?}", meta_path))
}

#[instrument(skip_all, fields(idx = %idx))]
pub(crate) fn try_session_by_index(idx: usize) -> Result<Session> {
    if idx == 0 {
        anyhow::bail!("session index starts from 1, not 0");
    }
    let sessions = list_sessions()?;
    sessions
        .get(idx - 1)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("session index {} out of range (1-{})", idx, sessions.len()))
}

#[instrument(skip_all, err, fields(key = %key))]
pub(crate) fn try_session_by_key(key: &SessionKey) -> Result<Session> {
    match key {
        SessionKey::Id(id) => try_session_by_id(id),
        SessionKey::Index(idx) => try_session_by_index(*idx),
    }
}

#[instrument(skip_all, fields(key = %key))]
pub fn find_session_by_key(key: &SessionKey) -> Result<Session> {
    try_session_by_key(key)
}
