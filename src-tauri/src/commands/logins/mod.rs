use hypertext::{html_elements, rsx_move, GlobalAttributes, Lazy, Renderable};
use tauri::http::StatusCode;
use tauri_plugin_store::StoreExt;

use crate::{
    commands::replace_director::{PageLocation, ResponseDirector},
    debug,
    domain::{
        judge::Judge,
        user::{IntitialUser, User},
        SurrealId,
    },
    state::{
        application_page::ApplicationPage,
        users::{decode_token, InitialTokenUser, TokenUser, Tokens, UserRoleTag},
        ManagedApplicationState, UserType,
    },
    templates::error::screen_error,
    STATE,
};

use super::{
    fetch::{fetch, Method},
    replace_director::ReplaceDirector,
};

#[derive(serde::Deserialize, Clone, Debug)]
struct JudgeResponse {
    token: String,
    user: User,
    judge: Judge,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct InitialJudgeResponse {
    token: String,
    user: IntitialUser,
    judge: Judge,
}

impl TryInto<JudgeResponse> for InitialJudgeResponse {
    type Error = String;

    fn try_into(self) -> Result<JudgeResponse, Self::Error> {
        match decode_token(&self.token) {
            Ok(c) => Ok(JudgeResponse {
                user: User {
                    id: SurrealId::make("user", c.claims.user_id.to_string().as_str()),
                    username: self.user.username,
                    email: self.user.email,
                    refresh_token: self.user.refresh_token,
                },
                token: self.token,
                judge: self.judge,
            }),
            Err(_) => Err("Not Authorized".to_string()),
        }
    }
}

#[tauri::command]
pub async fn login_judge(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    let judge_response: JudgeResponse = fetch(
        Method::Post,
        concat!(env!("API_URL"), "authenticate_as_judge"),
        &state,
    )
    .body(format!("{{\"id\":\"{id}\"}}"))
    .send()
    .await
    .map_err(|err| {
        debug!("{err:?}");
        screen_error("Error loading data")
    })?
    .json::<InitialJudgeResponse>()
    .await
    .map_err(|_| screen_error("Error parsing judge data"))?
    .try_into()
    .map_err(|_| screen_error("Error parsing judge data"))?;

    let token = judge_response.token.clone();
    let judge = UserType::Judge(
        judge_response.judge,
        TokenUser {
            user: judge_response.user.clone(),
            token: judge_response.token,
        },
    );

    state
        .write_async(move |app_state| {
            if let Some(refresh_token) = judge_response.user.refresh_token {
                app_state.set_tokens(Tokens {
                    token,
                    refresh_token,
                });
            };
            debug!("{judge:?}");
            app_state.user = judge;
        })
        .await?;
    match super::navigation::page_x_welcome(state.clone(), handle.clone()).await {
        Ok(page) => {
            ApplicationPage::Welcome.set_location(&handle)?;
            Ok(page)
        }
        err => err,
    }
}

#[tauri::command]
pub async fn login_user(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    email: Option<String>,
    password: Option<String>,
) -> ResponseDirector {
    let email = match email {
        Some(email) if !email.is_empty() => email,
        _ => return Err(error_email("You must supply an email")),
    };
    let password = match password {
        Some(password) if !password.is_empty() => password,
        _ => return Err(error_pass("You must supply a password")),
    };
    let initial_user = fetch(Method::Post, concat!(env!("API_URL"), "login"), &state)
        .body(format!(
            "{{\"email\":\"{email}\", \"password\": \"{password}\"}}",
        ))
        .send()
        .await
        .map_err(|_| error_gen("Error making request to login"))?
        .error_for_status()
        .map_err(|e| match e.status() {
            Some(StatusCode::UNAUTHORIZED) => error_pass("Incorrect password"),
            Some(StatusCode::FORBIDDEN) => error_gen("You do not have permission"),
            Some(StatusCode::REQUEST_TIMEOUT) => error_gen("Server error: Check connection"),
            Some(StatusCode::NOT_FOUND) => error_email("Incorrect email"),
            None | Some(_) => error_gen(e.to_string().as_str()),
        })?
        .json::<InitialTokenUser>()
        .await
        .map_err(|_| screen_error("Error parsing login data"))?;

    let Ok(user) = TryInto::<TokenUser>::try_into(initial_user) else {
        return Err(error_gen("You do not have permission"));
    };

    match user.clone().get_role_for_user() {
        UserRoleTag::Judge => return login_judge(state.clone(), handle, user.user.id.id()).await,
        UserRoleTag::NotAuthorised => return Err(error_gen("You do not have permission")),
        UserRoleTag::Admin => (),
    }
    state
        .write_async(|app_state| {
            if let Some(ref refresh_token) = user.user.refresh_token {
                app_state.set_tokens(Tokens {
                    token: user.token.to_string(),
                    refresh_token: refresh_token.to_string(),
                });
            }
            app_state.user = UserType::Admin(TokenUser {
                user: user.user,
                token: user.token,
            });
        })
        .await?;
    super::navigation::page_x_judge_login(state.clone(), handle).await
}

fn error_pass(string: &str) -> ReplaceDirector {
    ReplaceDirector::with_target(&PageLocation::PasswordLabel, error_box(string).render())
}

fn error_email(string: &str) -> ReplaceDirector {
    ReplaceDirector::with_target(&PageLocation::EmailLabel, error_box(string).render())
}
fn error_gen(string: &str) -> ReplaceDirector {
    ReplaceDirector::with_target(&PageLocation::LoginButton, error_box(string).render())
}
fn error_box<'a>(string: &'a str) -> Lazy<impl Fn(&mut String) + use<'a>> {
    rsx_move! {<div style="inline-size:100%; background:red; color:white; border-radius:var(--corner-size);
    padding-inline:0.3rem; box-sizing:border-box; font-weight:bold;">{string}</div>}
}
