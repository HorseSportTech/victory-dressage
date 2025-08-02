use crate::logging::Logger;
use crate::state::ManagedApplicationState;
use crate::templates::error::screen_error;
use crate::templates::settings::button_freestyle_mode;
use hypertext::rsx;
use hypertext::Renderable;
use tauri_plugin_store::StoreExt;

use crate::{
    sockets::manager::STORED_MESSAGES,
    state::{application_page::ApplicationPage, ApplicationState},
    templates::settings::clear_data_button,
    STORE_URI,
};

use super::replace_director::{PageLocation, ReplaceDirector, ResponseDirector};

#[tauri::command]
pub async fn toggle_freestyle_mode(
    state: tauri::State<'_, ManagedApplicationState>,
) -> ResponseDirector {
    let use_auto_freestyle = state
        .write_async(|app_state| {
            app_state.auto_freestyle = !app_state.auto_freestyle;
            app_state.auto_freestyle
        })
        .await
        .map_err(|_| screen_error("Cannot access judge preferences due to poisoned lock"))?;
    Ok(ReplaceDirector::with_target(
        &PageLocation::FreestyleModeBtn,
        button_freestyle_mode(use_auto_freestyle).render(),
    ))
}

#[tauri::command]
pub fn clear_data(
    app_state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    app_state
        .write(|app_state| {
            *app_state = ApplicationState {
                permanent_id: app_state.permanent_id,
                user: app_state.user.clone(),
                token_expires: app_state.token_expires,
                show: None,
                competition_id: None,
                starter_id: None,
                page: ApplicationPage::Settings,
                battery: app_state.battery.clone(),
                auto_freestyle: Default::default(),
                app_handle: app_state.app_handle.clone(),
            };
        })
        .map_err(|_| {
            ReplaceDirector::with_target(
                &PageLocation::ClearDataButton,
                rsx! {
                    {clear_data_button(false)}
                    "Error occured!"
                }
                .render(),
            )
        })?;
    let store = handle.store(STORE_URI).expect("To get the store");
    _ = store.delete(STORED_MESSAGES);
    Ok(ReplaceDirector::with_target(
        &PageLocation::ClearDataButton,
        clear_data_button(true).render(),
    ))
}

#[tauri::command]
pub fn download_file(handle: tauri::AppHandle) -> ResponseDirector {
    let logs = Logger::new(&handle).get_logs();
    println!("{logs}");
    Ok(ReplaceDirector::none())
}
