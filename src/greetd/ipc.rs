use greetd_ipc::{codec::TokioCodec, *};
use std::env;
use tokio::net::UnixStream;

use crate::model::model::*;

pub async fn login(input: &LoginCommandInput) -> Result<LoginState, Box<dyn std::error::Error>> {
    let LoginCommandInput {
        username,
        password,
        session,
    } = input;

    let Session {
        name: _,
        icon_name: _,
        command,
        env,
    } = session;

    let greetd_sock = env::var("GREETD_SOCK")?;
    let mut stream = UnixStream::connect(greetd_sock).await?;

    let mut next_request = Request::CreateSession {
        username: username.to_owned(),
    };

    let mut starting = false;

    loop {
        next_request.write_to(&mut stream).await?;

        match Response::read_from(&mut stream).await? {
            Response::AuthMessage {
                auth_message: _,
                auth_message_type,
            } => {
                let response = match auth_message_type {
                    AuthMessageType::Visible | AuthMessageType::Secret => Some(password.to_owned()),
                    AuthMessageType::Info | AuthMessageType::Error => None,
                };

                next_request = Request::PostAuthMessageResponse { response };
            }
            Response::Success => {
                if starting {
                    return Ok(LoginState::StartingSession);
                } else {
                    starting = true;
                    next_request = Request::StartSession {
                        env: env.to_owned(),
                        cmd: command.to_owned(),
                    }
                }
            }
            Response::Error {
                error_type,
                description,
            } => {
                Request::CancelSession.write_to(&mut stream).await?;
                eprintln!("Error: {description}");
                return match error_type {
                    ErrorType::AuthError => Ok(LoginState::AuthFailed),
                    ErrorType::Error => match starting {
                        true => Ok(LoginState::StartSessionError),
                        false => Ok(LoginState::CreateSessionError),
                    },
                };
            }
        }
    }
}
