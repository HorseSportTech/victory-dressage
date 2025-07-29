use hypertext::{html_elements, rsx, GlobalAttributes, Renderable};
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
        application_page::ApplicationPage, decode_token, InitialTokenUser, ManagedApplicationState,
        TokenUser, UserRoleTag, UserType,
    },
    templates::error::screen_error,
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
    let res = fetch(
        Method::Post,
        &format!(concat!(env!("API_URL"), "authenticate_as_judge")),
        state.clone(),
    )
    .body(format!("{{\"id\":\"{id}\"}}"))
    .send()
    .await;
    let res = res.map_err(|err| {
        debug!("{err:?}");
        screen_error("Error loading data")
    })?;
    let initial_judge_response = res.json::<InitialJudgeResponse>().await;
    let Ok(judge_response) = initial_judge_response
        .map_err(|err| err.to_string())
        .and_then(|x| TryInto::<JudgeResponse>::try_into(x))
    else {
        return Err(screen_error("Error parsing judge data"));
    };

    let judge = UserType::Judge(
        judge_response.judge,
        TokenUser {
            user: judge_response.user,
            token: judge_response.token,
        },
    );

    state
        .write_async(move |app_state| {
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
        Some(email) if email != "" => email,
        _ => return error_email("You must supply an email"),
    };
    let password = match password {
        Some(password) if password != "" => password,
        _ => return error_pass("You must supply a password"),
    };
    let Ok(res) = fetch(
        Method::Post,
        concat!(env!("API_URL"), "login"),
        state.clone(),
    )
    .body(format!(
        "{{\"email\":\"{}\", \"password\": \"{}\"}}",
        email, password
    ))
    .send()
    .await
    else {
        return error_gen("Error making request to login");
    };

    let res = match res.error_for_status() {
        Ok(res) => res,
        Err(e) => {
            return match e.status() {
                Some(StatusCode::UNAUTHORIZED) => error_pass("Incorrect password"),
                Some(StatusCode::FORBIDDEN) => error_gen("You do not have permission"),
                Some(StatusCode::REQUEST_TIMEOUT) => error_gen("Server error: Check connection"),
                Some(StatusCode::NOT_FOUND) => error_email("Incorrect email"),
                None | Some(_) => error_gen(e.to_string().as_str()),
            }
        }
    };

    let user_res = res
        .text()
        .await
        .map_err(|_| screen_error("Could not read response when trying to login"))?;

    let initial_user = match serde_json::from_str::<InitialTokenUser>(&user_res) {
        Ok(u) => u,
        Err(err) => {
            eprintln!("{user_res:?} {err:?}");
            return Err(screen_error("Error parsing login data"));
        }
    };
    let Ok(user) = TryInto::<TokenUser>::try_into(initial_user) else {
        return error_gen("You do not have permission");
    };

    match user.clone().get_role_for_user() {
        UserRoleTag::Judge => return login_judge(state.clone(), handle, user.user.id.id()).await,
        UserRoleTag::NotAuthorised => return error_gen("You do not have permission"),
        UserRoleTag::Admin => (),
    }
    state
        .write_async(|app_state| {
            app_state.user = UserType::Admin(TokenUser {
                user: user.user,
                token: user.token,
            });
        })
        .await?;
    let handle2 = handle.clone();
    state
        .read_async(move |app_state| {
            handle2
                .store(crate::STORE_URI)
                .expect("To be able to access store")
                .set("state", serde_json::to_value((*app_state).clone()).ok());
        })
        .await?;
    super::navigation::page_x_judge_login(state.clone(), handle).await
}

fn error_pass(string: &str) -> ResponseDirector {
    Ok(ReplaceDirector::with_target(
        &PageLocation::PasswordLabel,
		rsx!{<div style="inline-size:100%; background:red; color:white; border-radius:var(--corner-size);
					padding-inline:0.3rem; box-sizing:border-box; font-weight:bold;">{string}</div>}.render()
	))
}

fn error_email(string: &str) -> ResponseDirector {
    Ok(ReplaceDirector::with_target(
        &PageLocation::EmailLabel,
		rsx!{<div style="inline-size:100%; background:red; color:white; border-radius:var(--corner-size);
					padding-inline:0.3rem; box-sizing:border-box; font-weight:bold;">{string}</div>}.render()
	))
}
fn error_gen(string: &str) -> ResponseDirector {
    Ok(ReplaceDirector::with_target(
        &PageLocation::LoginButton,
		rsx!{<div style="inline-size:100%; background:red; color:white; border-radius:var(--corner-size);
					padding-inline:0.3rem; box-sizing:border-box; font-weight:bold;">{string}</div>}.render()
	))
}
