use crate::{
    commands::replace_director::ResponseDirector,
    state::{application_page::ApplicationPage, ManagedApplicationState},
    templates,
    traits::Entity,
};

use super::alert_manager::AlertManager;

#[tauri::command]
pub async fn recover(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let (application_page, competition_id) = state
        .read_async(|app_state| {
            (
                app_state.page.clone(),
                app_state.show.as_ref().map(|x| x.get_id().clone()),
            )
        })
        .await?;

    use ApplicationPage::*;
    match application_page {
        Error | Login | LoginJudge | Welcome => templates::login::login(state, handle).await,

        CompetitionList | Settings | Preferences => {
            templates::welcome::welcome(state, handle).await
        }

        Scoresheet(_) if competition_id.is_some() => {
            templates::competition_list::competition_list(
                state,
                handle,
                competition_id.expect("Exists"),
            )
            .await
        }
        Scoresheet(_) | FinalResult => {
            templates::scoresheet::scoresheet(state, alert_manager).await
        }
    }
}
