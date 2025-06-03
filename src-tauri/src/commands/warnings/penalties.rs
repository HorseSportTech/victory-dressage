use crate::domain::position::Position::{self, C};
use crate::domain::scoresheet::Scoresheet;
use crate::state::ApplicationState;
use crate::{
    commands::{
        alert_manager::{self, AlertManager, AlertType},
        replace_director::{emit_page, ReplaceDirector, ResponseDirector},
        PAGE_UPDATE,
    },
    state::ManagedApplicationState,
    templates::{
        error::screen_error,
        scoresheet::{artistic_row, errors_row, technical_row, warnings::get_warnings},
    },
};
use hypertext::Renderable;
use tauri::Emitter as _;

// ERRORS
#[tauri::command]
pub fn plus_error(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let position = get_position(&mut *app_state);
    let scoresheet = get_scoresheet(&mut *app_state)?;

    scoresheet.errors += 1;
    let errors = scoresheet.errors;
    alert_manager.toggle(AlertType::ErrorOfCourse(scoresheet.errors), &position);
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        "#penalties-errors",
        errors_row(true, errors).render(),
    ))
}
#[tauri::command]
pub fn sub_error(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let position = get_position(&mut *app_state);
    let scoresheet = get_scoresheet(&mut *app_state)?;

    alert_manager.toggle(AlertType::ErrorOfCourse(scoresheet.errors), &position);
    scoresheet.errors = scoresheet.errors.saturating_sub(1);
    let errors = scoresheet.errors;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        "#penalties-errors",
        errors_row(true, errors).render(),
    ))
}

// TECHNICAL PENALTIES
#[tauri::command]
pub fn plus_technical(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let position = get_position(&mut *app_state);
    let scoresheet = get_scoresheet(&mut *app_state)?;

    scoresheet.tech_penalties += 1;
    let errors = scoresheet.tech_penalties;
    alert_manager.toggle(
        AlertType::TechnicalPenalty(scoresheet.tech_penalties),
        &position,
    );
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        "#penalties-technical",
        technical_row(true, errors).render(),
    ))
}
#[tauri::command]
pub fn sub_technical(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let position = get_position(&mut *app_state);
    let scoresheet = get_scoresheet(&mut *app_state)?;

    alert_manager.toggle(
        AlertType::TechnicalPenalty(scoresheet.tech_penalties),
        &position,
    );
    scoresheet.tech_penalties = scoresheet.tech_penalties.saturating_sub(1);
    let tech_penalties = scoresheet.tech_penalties;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        "#penalties-technical",
        technical_row(true, tech_penalties).render(),
    ))
}

// ARTISTIC PENALTIES
#[tauri::command]
pub fn plus_artistic(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let position = get_position(&mut *app_state);
    let scoresheet = get_scoresheet(&mut *app_state)?;

    scoresheet.art_penalties += 1;
    let art_penalties = scoresheet.art_penalties;
    alert_manager.toggle(
        AlertType::ArtisticPenalty(scoresheet.art_penalties),
        &position,
    );
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        "#penalties-artistic",
        artistic_row(true, art_penalties).render(),
    ))
}
#[tauri::command]
pub fn sub_artistic(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let position = get_position(&mut *app_state);
    let scoresheet = get_scoresheet(&mut *app_state)?;
    alert_manager.toggle(
        AlertType::ArtisticPenalty(scoresheet.art_penalties),
        &position,
    );
    scoresheet.errors = scoresheet.art_penalties.saturating_sub(1);
    let art_penalties = scoresheet.art_penalties;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        "#penalties-artistic",
        artistic_row(true, art_penalties).render(),
    ))
}
fn get_position(app_state: &mut ApplicationState) -> Position {
    app_state
        .get_jury_member()
        .map_or(C, |x| x.position.clone())
}
fn get_scoresheet(app_state: &mut ApplicationState) -> Result<&mut Scoresheet, ReplaceDirector> {
    app_state
        .scoresheet_mut()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))
}
const TARGET_NAME: &'static str = "#alerts-and-warnings";
