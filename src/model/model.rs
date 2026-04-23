use std::fs;
use std::path::Path;

use gio::glib::SourceId;
use relm4::Controller;
use serde::Deserialize;

pub const ACCOUNTSERVICE_ICONS_PATH: &str = "/var/lib/AccountsService/icons";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginState {
    Waiting,
    LoggingIn,
    CreateSessionError,
    AuthFailed,
    StartSessionError,
    UnknownError,
    StartingSession,
    TooManyAttemps(u16),
}

#[derive(Debug)]
pub enum MainStackPage {
    Login,
    ChooseUser,
    ChooseSession,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    pub name: String,
    pub icon_name: String,
    pub command: Vec<String>,
    pub env: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoginCommandInput {
    pub username: String,
    pub password: String,
    pub session: Session,
}

#[derive(Debug)]
pub enum AppAction {
    ChangeLoginState(LoginState),
    ChangeMainStackPage(MainStackPage),
    UpdateSession(Session),
    UpdateUsername(String),
    UpdatePassword(String),
    LoginButtonResetTimerFinish,
}

#[derive(Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub avatar_path: String
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub assets_dir: String,
    pub monitor_order: Vec<String>,
    pub maximum_attempts: u16,
    pub maximum_attempts_timeout_seconds: u16,
    pub users: Option<Vec<User>>,
    pub ignored_sessions: Option<Vec<String>>,
    pub default_session: Option<String>,
}

pub struct LoginCommandModel {}

pub struct AppModel {
    pub login_state: LoginState,
    pub main_stack_page: MainStackPage,
    pub session: Session,
    pub username: String,
    pub password: String,
    pub attempts: u16,
    pub reset_login_button_timer_source_id: Option<SourceId>,
    pub config: Config,
    pub login_command: Controller<LoginCommandModel>,
}

fn parse_desktop_file(path: &Path) -> Option<Session> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let id = path.file_stem()?.to_str()?.to_string();

    let mut in_desktop_entry = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        } else if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }

        if in_desktop_entry {
            if line.starts_with("Name=") && name.is_none() {
                name = Some(line[5..].to_string());
            } else if line.starts_with("Exec=") && exec.is_none() {
                exec = Some(line[5..].to_string());
            }
        }
    }

    if let (Some(name), Some(exec)) = (name, exec) {
        // Simple cleanup of Exec: remove % placeholders
        let exec_clean = exec.split('%').next()?.trim();
        let command = exec_clean.split_whitespace().map(|s| s.to_string()).collect();
        Some(Session {
            name,
            icon_name: format!("{}-logo", id),
            command,
            env: vec!["XDG_SESSION_TYPE=wayland".to_string()],
        })
    } else {
        None
    }
}

pub fn get_wayland_sessions(ignored_sessions: Option<&Vec<String>>) -> Vec<Session> {
    let mut sessions = Vec::new();
    let path = Path::new("/usr/share/wayland-sessions");
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                if let Some(session) = parse_desktop_file(&path) {
                    if let Some(ignored) = ignored_sessions {
                        if ignored.contains(&session.name) {
                            continue;
                        }
                    }
                    sessions.push(session);
                }
            }
        }
    }
    sessions.sort_by(|a, b| a.name.cmp(&b.name));
    sessions
}
