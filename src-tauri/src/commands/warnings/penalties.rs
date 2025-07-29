use crate::commands::replace_director::PageLocation;
use crate::domain::position::Position::{self, C};
use crate::domain::scoresheet::Scoresheet;
use crate::state::ApplicationState;
use crate::{
    commands::{
        alert_manager::{AlertManager, AlertType},
        replace_director::{emit_page, ReplaceDirector, ResponseDirector},
    },
    state::ManagedApplicationState,
    templates::{
        error::screen_error,
        scoresheet::{artistic_row, errors_row, technical_row, warnings::get_warnings},
    },
};
use hypertext::Renderable;
const TARGET_NAME: &PageLocation = &PageLocation::AlertsAndWarnings;
const PENALTIES: &PageLocation = &PageLocation::PenaltiesErrors;
const TECHNICAL: &PageLocation = &PageLocation::PenaltiesTechnical;
const ARTISTIC: &PageLocation = &PageLocation::PenaltiesArtistic;

// ERRORS
#[tauri::command]
pub fn plus_error(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let errors = state.write(|app_state| {
        let position = get_position(&mut *app_state);
        let scoresheet = get_scoresheet(&mut *app_state)?;

        scoresheet.errors += 1;
        alert_manager.toggle(AlertType::ErrorOfCourse(scoresheet.errors), &position);
        Ok(scoresheet.errors)
    })??;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        PENALTIES,
        errors_row(true, errors).render(),
    ))
}
#[tauri::command]
pub fn sub_error(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let errors = state.write(|app_state| {
        let position = get_position(&mut *app_state);
        let scoresheet = get_scoresheet(&mut *app_state)?;

        alert_manager.toggle(AlertType::ErrorOfCourse(scoresheet.errors), &position);
        scoresheet.errors = scoresheet.errors.saturating_sub(1);
        Ok(scoresheet.errors)
    })??;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        PENALTIES,
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
    let tech_penalties = state.write(|app_state| {
        let position = get_position(&mut *app_state);
        let scoresheet = get_scoresheet(&mut *app_state)?;

        scoresheet.tech_penalties += 1;
        alert_manager.toggle(
            AlertType::TechnicalPenalty(scoresheet.tech_penalties),
            &position,
        );
        Ok(scoresheet.tech_penalties)
    })??;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        TECHNICAL,
        technical_row(true, tech_penalties).render(),
    ))
}
#[tauri::command]
pub fn sub_technical(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let tech_penalties = state.write(|app_state| {
        let position = get_position(&mut *app_state);
        let scoresheet = get_scoresheet(&mut *app_state)?;

        alert_manager.toggle(
            AlertType::TechnicalPenalty(scoresheet.tech_penalties),
            &position,
        );
        scoresheet.tech_penalties = scoresheet.tech_penalties.saturating_sub(1);
        Ok(scoresheet.tech_penalties)
    })??;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        TECHNICAL,
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
    let art_penalties = state.write(|app_state| {
        let position = get_position(&mut *app_state);
        let scoresheet = get_scoresheet(&mut *app_state)?;

        scoresheet.art_penalties += 1;
        alert_manager.toggle(
            AlertType::ArtisticPenalty(scoresheet.art_penalties),
            &position,
        );
        Ok(scoresheet.art_penalties)
    })??;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        ARTISTIC,
        artistic_row(true, art_penalties).render(),
    ))
}
#[tauri::command]
pub fn sub_artistic(
    app: tauri::AppHandle,
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let art_penalties = state.write(|app_state| {
        let position = get_position(&mut *app_state);
        let scoresheet = get_scoresheet(&mut *app_state)?;
        alert_manager.toggle(
            AlertType::ArtisticPenalty(scoresheet.art_penalties),
            &position,
        );
        scoresheet.errors = scoresheet.art_penalties.saturating_sub(1);
        Ok(scoresheet.art_penalties)
    })??;
    emit_page(&app, TARGET_NAME, get_warnings(alert_manager));
    Ok(ReplaceDirector::with_target(
        ARTISTIC,
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
