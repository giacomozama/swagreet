use std::fs::read_to_string;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use gio::glib::{self};
use gtk::{gdk, prelude::*};
use relm4::{Component, ComponentParts, SimpleComponent, prelude::*};

use crate::model::model::*;
use crate::widgets::widgets::*;
use crate::widgets::widgets::{dummy_window, login_state_page_name, main_stack_page_name};

fn load_css_and_icons(display: &gdk::Display, config: &Config, users: &[User]) {
    gtk::IconTheme::for_display(display)
        .add_search_path(&(config.assets_dir.to_owned() + "/icons"));

    let background_css = &format!(
        ":root {{ --bg-url: url('file://{path}/background.png'); }}\n",
        path = &config.assets_dir
    );

    let avatars_css = users
        .iter()
        .map(|User { name, avatar_path }| {
            format!(
                ".avatar-{name} {{ background-image: url(file://{path}); }}",
                path = avatar_path,
                name = name
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let css_source = read_to_string(Path::new(&(config.assets_dir.to_owned() + "/style.css")))
        .expect("CSS file not found!");

    let css = background_css.to_owned() + &avatars_css + &css_source;

    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);

    gtk::style_context_add_provider_for_display(
        display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn select_monitor(config: &Config, monitors: &[gdk::Monitor]) -> gdk::Monitor {
    let mut res = monitors.get(0).unwrap();
    let mut res_order = 999999;
    for monitor in monitors {
        let connector = monitor.connector().unwrap().to_string();
        let order = config
            .monitor_order
            .iter()
            .position(|r| *r == connector)
            .unwrap();
        if order < res_order {
            res = monitor;
            res_order = order;
        }
    }
    res.to_owned()
}

pub struct AppWidgets {
    main_stack: gtk::Stack,
    login_button_stack: gtk::Stack,
    password_entry: gtk::PasswordEntry,
    session_button: gtk::Button,
    session_button_logo: gtk::Image,
    session_button_label: gtk::Label,
    user_button: gtk::Button,
    user_button_avatar: gtk::Box,
    user_button_label: gtk::Label,
    login_button: gtk::Button,
    too_many_attempts_label: gtk::Label,
    suspend_button: gtk::Button,
    restart_button: gtk::Button,
    shutdown_button: gtk::Button,
}

impl SimpleComponent for AppModel {
    type Input = AppAction;
    type Output = ();
    type Init = (Config, Vec<User>);
    type Root = gtk::Window;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        root_window()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let display = gdk::Display::default().unwrap();
        let (config, users) = init;

        load_css_and_icons(&display, &config, &users);

        let monitors = display
            .monitors()
            .into_iter()
            .map(|m| m.unwrap().downcast::<gdk::Monitor>().unwrap())
            .collect::<Vec<gdk::Monitor>>();

        let main_monitor = select_monitor(&config, &monitors);
        root.fullscreen_on_monitor(&main_monitor);

        let initial_session = config.sessions.get(0).unwrap();

        let login_command = LoginCommandModel::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| msg);

        let model = AppModel {
            login_state: LoginState::Waiting,
            main_stack_page: MainStackPage::Login,
            session: initial_session.to_owned(),
            username: users.get(0).unwrap().name.to_owned(),
            password: "".to_owned(),
            attempts: 0,
            reset_login_button_timer_source_id: None,
            config: config.to_owned(),
            login_command,
        };

        let builder = gtk::Builder::from_file(config.assets_dir.to_owned() + "/layout.ui");

        for monitor in monitors {
            if monitor.connector() != main_monitor.connector() {
                let dummy_window = dummy_window();
                dummy_window.fullscreen_on_monitor(&monitor);
                dummy_window.present();
            }
        }

        let widgets = AppWidgets {
            main_stack: builder.object("main_stack").unwrap(),
            login_button_stack: builder.object("login_button_stack").unwrap(),
            password_entry: builder.object("password_entry").unwrap(),
            session_button: builder.object("session_button").unwrap(),
            session_button_logo: builder.object("session_button_logo").unwrap(),
            session_button_label: builder.object("session_button_label").unwrap(),
            user_button: builder.object("user_button").unwrap(),
            user_button_avatar: builder.object("user_button_avatar").unwrap(),
            user_button_label: builder.object("user_button_label").unwrap(),
            login_button: builder.object("login_button").unwrap(),
            too_many_attempts_label: builder.object("too_many_attempts_label").unwrap(),
            suspend_button: builder.object("suspend_button").unwrap(),
            restart_button: builder.object("restart_button").unwrap(),
            shutdown_button: builder.object("shutdown_button").unwrap(),
        };

        let wrapper_box = builder.object::<gtk::Widget>("wrapper_box").unwrap();

        let background_drawing_area = builder
            .object::<gtk::DrawingArea>("background_drawing_area")
            .unwrap();

        setup_background_drawing_area(&background_drawing_area, &config, &main_monitor);

        setup_main_stack(&widgets.main_stack, &config.sessions, &users, &sender);

        setup_login_button_stack(
            &widgets.login_button_stack,
            &widgets.too_many_attempts_label,
        );

        setup_password_entry(&widgets.password_entry, &sender);

        widgets.login_button.connect_clicked({
            let sender = sender.clone();
            move |_| {
                sender.input(AppAction::ChangeLoginState(LoginState::LoggingIn));
            }
        });

        widgets.session_button.connect_clicked({
            let sender = sender.clone();
            move |_| sender.input(AppAction::ChangeMainStackPage(MainStackPage::ChooseSession))
        });

        widgets.user_button.connect_clicked({
            let sender = sender.clone();
            move |_| sender.input(AppAction::ChangeMainStackPage(MainStackPage::ChooseUser))
        });

        widgets.suspend_button.connect_clicked(move |_| {
            Command::new("/bin/systemctl")
                .args(["suspend"])
                .output()
                .expect("Failed to execute command");
        });

        widgets.restart_button.connect_clicked(move |_| {
            Command::new("/bin/reboot")
                .output()
                .expect("Failed to execute command");
        });

        widgets.shutdown_button.connect_clicked(move |_| {
            Command::new("/bin/shutdown")
                .args(["now"])
                .output()
                .expect("Failed to execute command");
        });

        root.set_child(Some::<&gtk::Widget>(&wrapper_box));

        root.present();

        widgets
            .user_button_avatar
            .set_css_classes(&["avatar", &("avatar-".to_owned() + &model.username)]);

        widgets.user_button_label.set_label(&model.username);

        widgets
            .session_button_logo
            .set_icon_name(Some(&model.session.icon_name));

        widgets.session_button_label.set_label(&model.session.name);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: relm4::ComponentSender<Self>) {
        match message {
            AppAction::LoginButtonResetTimerFinish => {
                self.reset_login_button_timer_source_id = None;
                self.login_state = LoginState::Waiting;
            }
            AppAction::ChangeLoginState(login_state) => {
                let current_source_id =
                    std::mem::replace(&mut self.reset_login_button_timer_source_id, None);
                if let Some(source_id) = current_source_id {
                    source_id.remove();
                    self.reset_login_button_timer_source_id = None;
                }

                match login_state {
                    LoginState::LoggingIn => {
                        self.login_command
                            .sender()
                            .send(LoginCommandInput {
                                username: self.username.to_owned(),
                                password: self.password.to_owned(),
                                session: self.session.to_owned(),
                            })
                            .unwrap();
                        self.password = "".to_owned();
                        self.login_state = login_state;
                    }
                    LoginState::AuthFailed => {
                        if self.attempts + 1 == self.config.maximum_attempts {
                            self.attempts = 0;
                            self.login_state = LoginState::TooManyAttemps(
                                self.config.maximum_attempts_timeout_seconds,
                            );
                        } else {
                            self.attempts += 1;
                            let source_id = glib::timeout_add_seconds(3, move || {
                                sender.input(AppAction::LoginButtonResetTimerFinish);
                                glib::ControlFlow::Break
                            });
                            self.reset_login_button_timer_source_id = Some(source_id);
                            self.login_state = login_state;
                        }
                    }
                    LoginState::StartingSession => {
                        self.login_state = login_state;
                    }
                    _ => {
                        let source_id = glib::timeout_add_seconds(3, move || {
                            sender.input(AppAction::LoginButtonResetTimerFinish);
                            glib::ControlFlow::Break
                        });
                        self.reset_login_button_timer_source_id = Some(source_id);
                        self.login_state = login_state;
                    }
                }
            }
            AppAction::ChangeMainStackPage(main_stack_page) => {
                self.main_stack_page = main_stack_page;
            }
            AppAction::UpdateSession(session) => {
                self.session = session;
                self.main_stack_page = MainStackPage::Login;
            }
            AppAction::UpdateUsername(username) => {
                self.username = username;
                self.main_stack_page = MainStackPage::Login;
            }
            AppAction::UpdatePassword(password) => {
                self.password = password;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, sender: relm4::ComponentSender<Self>) {
        widgets
            .main_stack
            .set_visible_child_name(&main_stack_page_name(&self.main_stack_page));

        widgets
            .login_button_stack
            .set_visible_child_name(&login_state_page_name(&self.login_state));

        widgets
            .user_button_avatar
            .set_css_classes(&["avatar", &("avatar-".to_owned() + &self.username)]);

        widgets.user_button_label.set_label(&self.username);

        widgets
            .session_button_logo
            .set_icon_name(Some(&self.session.icon_name));

        widgets.session_button_label.set_label(&self.session.name);

        if self.password.is_empty() && !widgets.password_entry.text().is_empty() {
            widgets.password_entry.set_text("");
        }

        let can_login = match self.login_state {
            LoginState::LoggingIn | LoginState::StartingSession => false,
            LoginState::TooManyAttemps(seconds_left) => {
                widgets.too_many_attempts_label.set_label(&format!(
                    "Too many attempts. Please wait {secs}s.",
                    secs = seconds_left
                ));
                glib::source::timeout_add_seconds(1, move || {
                    if seconds_left == 1 {
                        sender.input(AppAction::ChangeLoginState(LoginState::Waiting));
                    } else {
                        sender.input(AppAction::ChangeLoginState(LoginState::TooManyAttemps(
                            seconds_left - 1,
                        )));
                    }
                    glib::ControlFlow::Break
                });
                false
            }
            _ => true,
        };

        widgets.login_button.set_sensitive(can_login);
        widgets.session_button.set_sensitive(can_login);
        widgets.user_button.set_sensitive(can_login);
        widgets.password_entry.set_sensitive(can_login);

        if self.login_state == LoginState::StartingSession {
            glib::source::timeout_add(Duration::from_millis(1500), || {
                std::process::exit(0);
            });
        }
    }
}
