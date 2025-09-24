use std::{fs::read_to_string, path::Path};

use getopts::Options;
use gio::prelude::*;
use gtk::Application;

use crate::model::model::*;

mod greetd;
mod model;
mod ui;
mod widgets;

const APP_ID: &str = "st.contraptioni.swagreet";

fn get_account_service_users() -> Vec<User> {
    gio::File::for_path(ACCOUNTSERVICE_ICONS_PATH)
        .enumerate_children("", gio::FileQueryInfoFlags::NONE, None::<&gio::Cancellable>)
        .expect("Couldn't read users from AccountService")
        .into_iter()
        .map(|f| f.unwrap().name().to_str().unwrap().to_owned())
        .map(|u| User {
            avatar_path: format!("{path}/{name}", path = ACCOUNTSERVICE_ICONS_PATH, name = u),
            name: u,
        })
        .collect()
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config", "set config file name", "NAME");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let mut config_path = "/etc/greetd/swagreet/config.toml".to_owned();
    if matches.opt_present("c") {
        config_path = matches.opt_str("c").unwrap().to_owned();
        println!("{}", &config_path);
    }

    let config_raw = read_to_string(Path::new(&config_path)).expect("Config file not found!");

    let config: Config = toml::from_str(&config_raw).expect("Invalid config file!");
    let users = config
        .users
        .to_owned()
        .unwrap_or(get_account_service_users());
    let app = Application::builder().application_id(APP_ID).build();

    relm4::RelmApp::from_app(app)
        .with_args(Vec::new())
        .visible_on_activate(false)
        .run::<AppModel>((config, users));
}
