use crate::model::model::{AppAction, AppModel, Config, LoginState, MainStackPage, Session, User};

use gtk::{gdk::Monitor, prelude::*};

pub fn setup_login_button_stack(
    login_button_stack: &gtk::Stack,
    too_many_attempts_label: &gtk::Label,
) {
    login_button_stack.add_named(&gtk::Label::new(Some("Login")), Some("login"));

    login_button_stack.add_named(&gtk::Label::new(Some("Please wait…")), Some("logging_in"));

    login_button_stack.add_named(
        &gtk::Label::new(Some("Couldn't create session")),
        Some("create_session_error"),
    );

    login_button_stack.add_named(
        &gtk::Label::new(Some("Wrong password")),
        Some("auth_failed"),
    );

    login_button_stack.add_named(
        &gtk::Label::new(Some("Couldn't start session")),
        Some("start_session_error"),
    );

    login_button_stack.add_named(
        &gtk::Label::new(Some("Starting session…")),
        Some("starting_session"),
    );

    login_button_stack.add_named(
        &gtk::Label::new(Some("Unknown error")),
        Some("unknown_error"),
    );

    login_button_stack.add_named(too_many_attempts_label, Some("too_many_attempts"));

    login_button_stack.set_visible_child_name("login");
}

pub fn login_state_page_name(state: &LoginState) -> String {
    match state {
        LoginState::Waiting => "login",
        LoginState::LoggingIn => "logging_in",
        LoginState::CreateSessionError => "create_session_error",
        LoginState::AuthFailed => "auth_failed",
        LoginState::StartSessionError => "start_session_error",
        LoginState::StartingSession => "starting_session",
        LoginState::UnknownError => "unknown_error",
        LoginState::TooManyAttemps(_) => "too_many_attempts",
    }
    .to_owned()
}

pub fn session_logo(icon_name: &str) -> gtk::Image {
    gtk::Image::builder()
        .halign(gtk::Align::Start)
        .icon_name(icon_name)
        .pixel_size(24)
        .build()
}

pub fn session_item(session_logo: &gtk::Image, label: &gtk::Label) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["session-item"])
        .hexpand(true)
        .build();

    container.append(session_logo);

    label.set_hexpand(true);
    container.append(label);

    container
}

pub fn user_avatar(username: &str) -> gtk::Box {
    gtk::Box::builder()
        .css_classes(["avatar", &("avatar-".to_owned() + username)])
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Start)
        .overflow(gtk::Overflow::Hidden)
        .width_request(40)
        .height_request(40)
        .build()
}

pub fn user_item(avatar: &gtk::Box, label: &gtk::Label) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["user-item"])
        .hexpand(true)
        .build();

    container.append(avatar);

    label.set_hexpand(true);
    container.append(label);

    container
}

fn pointer_cursor() -> gtk::gdk::Cursor {
    gtk::gdk::Cursor::from_name("pointer", None).unwrap()
}

pub fn setup_password_entry(entry: &gtk::PasswordEntry, sender: &relm4::ComponentSender<AppModel>) {
    let sender_clone_notify_text = sender.clone();
    entry.connect_text_notify(move |e| {
        sender_clone_notify_text.input(AppAction::UpdatePassword(e.text().to_string().to_owned()))
    });

    entry.connect_realize(|e| {
        e.grab_focus();
    });

    let sender_clone_activate = sender.clone();
    entry.connect_activate(move |_| {
        sender_clone_activate.input(AppAction::ChangeLoginState(LoginState::LoggingIn));
    });
}

fn choose_session_form(
    sessions: &[Session],
    sender: &relm4::ComponentSender<AppModel>,
) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["choose-session"])
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .spacing(12)
        .build();

    for session in sessions.to_owned() {
        let button = gtk::Button::builder()
            .css_classes(["pill"])
            .child(&session_item(
                &session_logo(&session.icon_name),
                &gtk::Label::new(Some(&session.name)),
            ))
            .cursor(&pointer_cursor())
            .build();

        let sender_clone = sender.clone();
        button.connect_clicked(move |_| {
            sender_clone.input(AppAction::UpdateSession(session.to_owned()));
        });

        container.append(&button);
    }

    container
}

fn choose_user_page(users: &[User], sender: &relm4::ComponentSender<AppModel>) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["choose-user"])
        .orientation(gtk::Orientation::Vertical)
        .valign(gtk::Align::Center)
        .spacing(12)
        .build();

    for user in users.to_owned() {
        let button = gtk::Button::builder()
            .css_classes(["pill"])
            .child(&user_item(
                &user_avatar(&user.name),
                &gtk::Label::new(Some(&user.name)),
            ))
            .cursor(&pointer_cursor())
            .build();

        let sender_clone = sender.clone();
        button.connect_clicked(move |_| {
            sender_clone.input(AppAction::UpdateUsername(user.name.to_owned()));
        });

        container.append(&button);
    }

    container
}

pub fn setup_main_stack(
    main_stack: &gtk::Stack,
    sessions: &[Session],
    users: &[User],
    sender: &relm4::ComponentSender<AppModel>,
) {
    main_stack.add_named(
        &choose_session_form(sessions, sender),
        Some("choose-session"),
    );

    main_stack.add_named(&choose_user_page(users, sender), Some("choose-user"));

    main_stack.set_visible_child_name("login");
}

pub fn main_stack_page_name(page: &MainStackPage) -> String {
    match page {
        MainStackPage::Login => "login",
        MainStackPage::ChooseUser => "choose-user",
        MainStackPage::ChooseSession => "choose-session",
    }
    .to_owned()
}

pub fn setup_background_drawing_area(
    drawing_area: &gtk::DrawingArea,
    config: &Config,
    monitor: &Monitor,
) {
    let screen_width = monitor.geometry().width() as f64;
    let screen_height = monitor.geometry().height() as f64;
    let wallpaper_path = config.assets_dir.to_owned() + "/background.png";

    drawing_area.set_draw_func(move |_, cr, width, height| {
        let file = gio::File::for_path(&wallpaper_path);
        let mut stream = file.read(None::<&gio::Cancellable>).unwrap().into_read();
        let surface = cairo::ImageSurface::create_from_png(&mut stream).unwrap();
        let x_scale = surface.width() as f64 / screen_width;
        let y_scale = surface.height() as f64 / screen_height;
        surface.set_device_scale(x_scale, y_scale);
        let x = (screen_width - width as f64) / -2f64;
        let y = (screen_height - height as f64) / -2f64;
        cr.set_source_surface(surface, x, y).unwrap();
        cr.paint().unwrap();
    });
}

pub fn root_window() -> gtk::Window {
    gtk::Window::builder()
        .title("swagreet")
        .name("main")
        .css_classes(["Main"])
        .decorated(false)
        .hexpand(true)
        .vexpand(true)
        .visible(false)
        .maximized(true)
        .fullscreened(true)
        .build()
}

pub fn dummy_window() -> gtk::Window {
    gtk::Window::builder()
        .hexpand(true)
        .vexpand(true)
        .decorated(false)
        .visible(false)
        .maximized(true)
        .fullscreened(true)
        .build()
}
