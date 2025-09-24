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
    pub sessions: Vec<Session>,
    pub maximum_attempts: u16,
    pub maximum_attempts_timeout_seconds: u16,
    pub users: Option<Vec<User>>
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
