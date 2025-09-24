use relm4::{Component, ComponentParts};

use crate::{greetd::ipc::login, model::model::*};

impl Component for LoginCommandModel {
    type CommandOutput = LoginState;
    type Input = LoginCommandInput;
    type Output = AppAction;
    type Init = ();
    type Root = ();
    type Widgets = ();

    fn init_root() -> Self::Root {
        ()
    }

    fn init(
        _: Self::Init,
        _: Self::Root,
        _: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        ComponentParts {
            model: LoginCommandModel {},
            widgets: (),
        }
    }

    fn update(&mut self, input: Self::Input, sender: relm4::ComponentSender<Self>, _: &Self::Root) {
        let input_clone = input.clone();
        sender.oneshot_command(async move {
            match login(&input_clone).await {
                Ok(state) => state,
                Err(_) => LoginState::UnknownError,
            }
        });
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: relm4::ComponentSender<Self>,
        _: &Self::Root,
    ) {
        sender.output(AppAction::ChangeLoginState(message)).unwrap();
    }
}
