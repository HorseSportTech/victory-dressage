use crate::{
    commands::replace_director::ResponseDirector,
    state::{ManagedApplicationState, UserType},
};

use super::{log_out, replace_director::ReplaceDirector};

#[tauri::command]
pub async fn update_comment_first(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
) -> ResponseDirector {
    let bool_value = value == "true";
    match state
        .write_async(move |app_state| match app_state.user {
            UserType::Judge(ref mut judge, _) => {
                judge.prefs.comment_last = bool_value;
                Ok(())
            }
            _ => Err(()),
        })
        .await
    {
        Ok(_) => Ok(ReplaceDirector::none()),
        Err(_) => log_out::log_out(state.clone(), handle).await,
    }
}

#[tauri::command]
pub async fn update_show_trend(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
) -> ResponseDirector {
    let bool_value = value == "true";
    match state
        .write_async(move |app_state| match app_state.user {
            UserType::Judge(ref mut judge, _) => {
                judge.prefs.hide_trend = bool_value;
                Ok(())
            }
            _ => Err(()),
        })
        .await
    {
        Ok(_) => Ok(ReplaceDirector::none()),
        Err(_) => log_out::log_out(state.clone(), handle).await,
    }
}

#[tauri::command]
pub async fn update_auto_sign(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
) -> ResponseDirector {
    let bool_value = value == "true";
    match state
        .write_async(move |app_state| match app_state.user {
            UserType::Judge(ref mut judge, _) => {
                judge.prefs.manually_sign = bool_value;
                Ok(())
            }
            _ => Err(()),
        })
        .await
    {
        Ok(_) => Ok(ReplaceDirector::none()),
        Err(_) => log_out::log_out(state.clone(), handle).await,
    }
}
