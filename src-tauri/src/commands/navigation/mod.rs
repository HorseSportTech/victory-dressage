use super::{super::state::application_page::ApplicationPage, alert_manager::AlertManager};
use crate::{
    commands::replace_director::ResponseDirector,
    domain::{show::Shows, starter::Starter},
    state::ManagedApplicationState,
    templates::error::screen_error,
    traits::{Entity, Storable},
};

#[tauri::command]
pub async fn page_x_judge_login(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    super::super::templates::choose_judge::choose_judge(state, handle).await
}

#[tauri::command]
pub async fn page_x_welcome(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    // TODO: update state and store show list
    super::super::templates::welcome::welcome(state, handle).await
}

#[tauri::command]
pub async fn page_x_preferences(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    match super::super::templates::preferences::get_preferences(state.clone(), handle).await {
        Ok(page) => {
            state.write(|x| x.page = ApplicationPage::Preferences);
            Ok(page)
        }
        Err(err) => Err(err),
    }
}
#[tauri::command]
pub async fn page_x_settings(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    match super::super::templates::settings::get_settings(state.clone(), handle).await {
        Ok(page) => {
            state.write(|x| x.page = ApplicationPage::Settings);
            Ok(page)
        }
        Err(err) => Err(err),
    }
}
#[tauri::command]
pub async fn page_x_results(
    state: tauri::State<'_, ManagedApplicationState>,
    _handle: tauri::AppHandle,
) -> ResponseDirector {
    match super::super::templates::result::result(state.clone()).await {
        Ok(page) => {
            state.write(|x| x.page = ApplicationPage::FinalResult)?;
            Ok(page)
        }
        Err(err) => Err(err),
    }
}

#[tauri::command]
pub async fn page_x_competition_list(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    let id2 = id.clone();
    let id3 = id.clone();
    if state
        .read_async(move |x| x.show.as_ref().is_none_or(|x| x.get_id() != id2))
        .await?
    {
        let shows = Shows::get(&handle, "shows")
            .map_err(|_| screen_error("Cannot find shows to navigate"))?;

        state
            .write_async(move |app_state| {
                app_state.show = shows.0.into_iter().find(|x| x.get_id() == id3);
            })
            .await?
    }
    super::super::templates::competition_list::competition_list(state, handle, id).await
}

#[tauri::command]
pub async fn page_x_scoresheet(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    _handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    let competition = state
        .read_async(move |x| {
            x.show
                .as_ref()
                .and_then(|x| x.competitions.iter().find(|c| c.id.id() == id))
                .map(|x| x.clone())
        })
        .await?
        .ok_or_else(|| screen_error("Competition not found for scoresheet"))?;

    let starter: Option<&Starter> = competition
        .starters
        .iter()
        .find(|x| !x.status.is_finished());
    let starter: Option<Starter> = starter.map_or_else(
        || competition.starters.first().cloned(),
        |x| Some(x.clone()),
    );

    if let Some(s) = starter {
        state
            .write_async(move |app| {
                app.competition = Some(competition.clone());
                app.page = ApplicationPage::Scoresheet(s.id.clone());
                app.starter = Some(s);
            })
            .await?;

        crate::templates::scoresheet::scoresheet(state, alert_manager).await
    } else {
        return Err(screen_error("No starters in this competition"));
    }
}

#[tauri::command]
pub async fn page_x_current(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let application_page = state.read_async(|x| x.page.clone()).await?;

    match application_page {
        ApplicationPage::Login => crate::templates::login::login(state, handle).await,
        ApplicationPage::LoginJudge => {
            crate::templates::choose_judge::choose_judge(state, handle).await
        }
        ApplicationPage::Welcome => crate::templates::welcome::welcome(state, handle).await,
        ApplicationPage::CompetitionList => {
            crate::templates::competition_list::competition_list(
                state,
                handle,
                String::from("TODO"),
            )
            .await
        }
        ApplicationPage::Scoresheet(_) => {
            crate::templates::scoresheet::scoresheet(state, alert_manager).await
        }
        ApplicationPage::Settings => crate::templates::settings::get_settings(state, handle).await,
        ApplicationPage::Preferences => {
            crate::templates::preferences::get_preferences(state, handle).await
        }
        ApplicationPage::FinalResult => todo!(),
        ApplicationPage::Error => Err(screen_error("Unspecified Error")),
    }
}
