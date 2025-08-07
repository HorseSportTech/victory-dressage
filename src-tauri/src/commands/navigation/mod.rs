use std::str::FromStr;

use tauri::Manager;

use super::{
    super::state::application_page::ApplicationPage,
    alert_manager::AlertManager,
    replace_director::{
        emit_page, emit_page_prerendered, emit_page_with_director, PageLocation, ReplaceDirector,
    },
};
use crate::{
    commands::replace_director::ResponseDirector,
    debug,
    domain::{
        show::{Show, Shows},
        starter::Starter,
    },
    sockets::{manager::ManagedSocket, message_types::application},
    state::{store::Storable, ManagedApplicationState},
    templates::{self, error::screen_error},
    traits::{Entity, Fetchable},
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
            state.write(|x| x.page = ApplicationPage::Preferences)?;
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
            state.write(|x| x.page = ApplicationPage::Settings)?;
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
    match templates::result::result(state.clone()).await {
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
    let id4 = id.clone();
    let show_does_not_exist = state
        .read_async(move |x| x.show.as_ref().is_none_or(|x| x.get_id() != id2))
        .await?;
    if show_does_not_exist {
        let shows = Shows::retrieve(&handle)
            .ok_or_else(|| screen_error("Cannot find shows to navigate"))?;

        state
            .write_async(move |app_state| {
                app_state.show = shows.get_show_by_str_id(&id3).cloned();
            })
            .await?;
    }
    let handle2 = handle.clone();
    tauri::async_runtime::spawn(async move {
        let state = handle2.state::<ManagedApplicationState>();
        if let Ok(show) = Show::select(&state, &id4).await {
            let list = templates::competition_list::render_list(show.competitions);
            emit_page(&handle2, &PageLocation::CompetitionList, list);
        };
        Ok::<(), ReplaceDirector>(())
    });
    templates::competition_list::competition_list(state, handle, id).await
}

#[tauri::command]
pub async fn page_x_scoresheet(
    state: tauri::State<'_, ManagedApplicationState>,
    soc_man: tauri::State<'_, ManagedSocket>,
    alert_manager: tauri::State<'_, AlertManager>,
    _handle: tauri::AppHandle,
    id: String,
) -> ResponseDirector {
    if let Err(err) = soc_man
        .send(application::Payload::Subscribe {
            competition_id: ulid::Ulid::from_str(&id).map_err(|_| {
                screen_error("Competition ID was not in expected format. Should be ULID")
            })?,
        })
        .await
    {
        debug!(red, "{err:?}");
    }
    let competition = state
        .read_async(move |x| {
            x.show
                .as_ref()
                .and_then(|x| x.competitions.iter().find(|c| c.id.id() == id))
                .cloned()
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
                app.competition_id = Some(competition.id.clone());
                app.page = ApplicationPage::Scoresheet(s.id.clone());
                app.starter_id = Some(s.id.clone());
            })
            .await?;

        crate::templates::scoresheet::scoresheet(state, alert_manager).await
    } else {
        Err(screen_error("No starters in this competition"))
    }
}

#[tauri::command]
pub async fn page_x_current(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    use templates::*;
    let application_page = state.read_async(|x| x.page.clone()).await?;

    match application_page {
        ApplicationPage::Login => login::login(state, handle).await,
        ApplicationPage::LoginJudge => choose_judge::choose_judge(state, handle).await,
        ApplicationPage::Welcome => welcome::welcome(state, handle).await,
        ApplicationPage::CompetitionList => {
            competition_list::competition_list(state, handle, String::from("TODO")).await
        }
        ApplicationPage::Scoresheet(_) => scoresheet::scoresheet(state, alert_manager).await,
        ApplicationPage::Settings => settings::get_settings(state, handle).await,
        ApplicationPage::Preferences => preferences::get_preferences(state, handle).await,
        ApplicationPage::FinalResult => result::result(state).await,
        ApplicationPage::Error => Err(screen_error("Unspecified Error")),
    }
}
