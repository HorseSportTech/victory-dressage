use crate::{
    commands::replace_director::ResponseDirector,
    state::{application_page::ApplicationPage, ManagedApplicationState},
};

use super::alert_manager::AlertManager;

#[tauri::command]
pub async fn recover(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let application_page = state.read_async(|app_state| app_state.page.clone()).await?;
    match application_page {
        ApplicationPage::Error
        | ApplicationPage::Login
        | ApplicationPage::LoginJudge
        | ApplicationPage::Welcome => crate::templates::login::login(state, handle).await,

        ApplicationPage::CompetitionList
        | ApplicationPage::Settings
        | ApplicationPage::Preferences => crate::templates::welcome::welcome(state, handle).await,

        ApplicationPage::Scoresheet(_) | ApplicationPage::FinalResult => {
            crate::templates::scoresheet::scoresheet(state, alert_manager).await
        }
    }
}
