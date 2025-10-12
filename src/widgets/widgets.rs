use crate::model::{
    self,
    model::{AppAction, AppModel, Config, LoginState, MainStackPage, Session, User},
};

use gtk::{gdk::Monitor, prelude::*};
use std::process::Command;

pub fn login_button_stack(too_many_attempts_label: &gtk::Label) -> gtk::Stack {
    let login_button_stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(125)
        .build();

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

    login_button_stack
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

pub fn login_button(
    login_button_stack: &gtk::Stack,
    sender: &relm4::ComponentSender<AppModel>,
) -> gtk::Button {
    let button = gtk::Button::builder()
        .css_classes(["login-button", "pill"])
        .sensitive(true) // canLogin
        .cursor(&pointer_cursor())
        .child(login_button_stack)
        .build();

    let sender_clone = sender.clone();
    button.connect_clicked(move |_| {
        sender_clone.input(AppAction::ChangeLoginState(
            model::model::LoginState::LoggingIn,
        ));
    });

    button
}

fn pointer_cursor() -> gtk::gdk::Cursor {
    gtk::gdk::Cursor::from_name("pointer", None).unwrap()
}

fn vertical_spacer() -> gtk::Box {
    gtk::Box::builder().vexpand(true).build()
}

fn separator() -> gtk::Box {
    gtk::Box::builder()
        .css_name("separator")
        .halign(gtk::Align::Center)
        .build()
}

pub fn session_button(
    session_item: &gtk::Box,
    sender: &relm4::ComponentSender<AppModel>,
) -> gtk::Button {
    let button = gtk::Button::builder()
        .css_classes(["pill"])
        .sensitive(true) // canLogin
        .cursor(&pointer_cursor())
        .child(session_item)
        .build();

    let sender_clone = sender.clone();
    button.connect_clicked(move |_| {
        sender_clone.input(AppAction::ChangeMainStackPage(MainStackPage::ChooseSession));
    });

    button
}

pub fn user_button(user_item: &gtk::Box, sender: &relm4::ComponentSender<AppModel>) -> gtk::Button {
    let button = gtk::Button::builder()
        .css_classes(["pill"])
        .sensitive(true) // canLogin
        .cursor(&pointer_cursor())
        .child(user_item)
        .build();

    let sender_clone = sender.clone();
    button.connect_clicked(move |_| {
        sender_clone.input(AppAction::ChangeMainStackPage(MainStackPage::ChooseUser));
    });

    button
}

pub fn password_entry(sender: &relm4::ComponentSender<AppModel>) -> gtk::PasswordEntry {
    let entry = gtk::PasswordEntry::builder()
        .css_classes(["pill"])
        .sensitive(true) // canLogin
        .placeholder_text("Password")
        .hexpand(true)
        .xalign(0.5)
        .build();

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

    entry
}

pub fn login_form(
    session_button: &gtk::Button,
    user_button: &gtk::Button,
    password_entry: &gtk::PasswordEntry,
    login_button: &gtk::Button,
) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["login-form"])
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .build();

    container.append(session_button);
    container.append(&separator());
    container.append(user_button);
    container.append(password_entry);
    container.append(&separator());
    container.append(login_button);

    container
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

fn choose_user_form(users: &[User], sender: &relm4::ComponentSender<AppModel>) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["choose-session"])
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

fn bottom_button(icon_name: &str, label: &str, cmd: &[String]) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_classes(["button-container"])
        .orientation(gtk::Orientation::Vertical)
        .build();

    let button = gtk::Button::builder()
        .cursor(&pointer_cursor())
        .icon_name(icon_name)
        .halign(gtk::Align::Center)
        .build();

    let cmd = cmd.to_owned();
    button.connect_clicked(move |_| {
        Command::new(&cmd[0])
            .args(&cmd[1..])
            .output()
            .expect("Failed to execute command");
    });

    container.append(&button);
    container.append(&gtk::Label::new(Some(label)));

    container
}

pub fn main_stack(
    sessions: &[Session],
    users: &[User],
    login_form: &gtk::Box,
    sender: &relm4::ComponentSender<AppModel>,
) -> gtk::Stack {
    let stack = gtk::Stack::builder()
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(125)
        .build();

    stack.add_named(
        &choose_session_form(sessions, sender),
        Some("choose-session"),
    );
    stack.add_named(login_form, Some("login"));
    stack.add_named(&choose_user_form(users, sender), Some("choose-user"));

    stack.set_visible_child_name("login");

    stack
}

pub fn main_stack_page_name(page: &MainStackPage) -> String {
    match page {
        MainStackPage::Login => "login",
        MainStackPage::ChooseUser => "choose-user",
        MainStackPage::ChooseSession => "choose-session",
    }
    .to_owned()
}

pub fn content_box(main_stack: &gtk::Stack) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_name("ContentBox")
        .orientation(gtk::Orientation::Vertical)
        .vexpand(true)
        .hexpand(true)
        .build();

    let logo_image = gtk::Image::builder()
        .css_classes(["logo"])
        .icon_name("main-logo")
        .pixel_size(72)
        .build();

    container.append(&logo_image);
    container.append(&vertical_spacer());
    container.append(main_stack);
    container.append(&vertical_spacer());

    let bottom_buttons_row = gtk::Box::builder()
        .css_classes(["bottom-buttons-row"])
        .spacing(24)
        .halign(gtk::Align::End)
        .hexpand(true)
        .build();

    let suspend_button = bottom_button(
        "system-suspend-symbolic",
        "Suspend",
        &["/bin/systemctl".to_owned(), "suspend".to_owned()],
    );

    bottom_buttons_row.append(&suspend_button);

    let restart_button = bottom_button(
        "system-reboot-symbolic",
        "Restart",
        &["/bin/reboot".to_owned()],
    );

    bottom_buttons_row.append(&restart_button);

    let shutdown_button = bottom_button(
        "system-shutdown-symbolic",
        "Shutdown",
        &["/bin/shutdown".to_owned(), "now".to_owned()],
    );

    bottom_buttons_row.append(&shutdown_button);

    container.append(&bottom_buttons_row);

    container
}

pub fn background_widget(config: &Config, monitor: &Monitor) -> gtk::Box {
    let container = gtk::Box::builder()
        .css_name("BoxBackgroundContainer")
        .layout_manager(&gtk::BinLayout::new())
        .overflow(gtk::Overflow::Hidden)
        .build();

    let drawing_area = gtk::DrawingArea::builder()
        .css_name("BoxBackground")
        .vexpand(true)
        .hexpand(true)
        .build();

    container.append(&drawing_area);

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

    let background_overlay = gtk::Box::builder()
        .css_name("BoxBackgroundOverlay")
        .vexpand(true)
        .hexpand(true)
        .build();

    container.append(&background_overlay);

    container
}

pub fn wrapper_box(background_widget: &gtk::Box, content_box: &gtk::Box) -> gtk::Box {
    let container = gtk::Box::builder()
        .layout_manager(&gtk::BinLayout::new())
        .width_request(500)
        .height_request(800)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();

    container.append(background_widget);
    container.append(content_box);

    container
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
