use crate::{
    state::{ApplicationState, ManagedApplicationState},
    templates::error::screen_error,
};

use super::replace_director::ResponseDirector;

#[tauri::command]
pub async fn log_out(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    state
        .write_async(|x| *x = ApplicationState::new())
        .await
        .map_err(|_| screen_error("Cannot log out session"))?;
    crate::templates::login::login(state, handle).await
}

