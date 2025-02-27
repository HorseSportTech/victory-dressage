use hypertext::Renderable;

use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    state::ManagedApplicationState,
    templates::{
        error::screen_error,
        scoresheet::{artistic_row, errors_row, technical_row},
    },
};

// ERRORS
#[tauri::command]
pub fn plus_error(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let scoresheet = app_state
        .scoresheet()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

    scoresheet.errors += 1;
    let errors = scoresheet.errors;
    Ok(ReplaceDirector::with_target(
        "#penalties-errors",
        errors_row(true, errors).render(),
    ))
}
#[tauri::command]
pub fn sub_error(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let scoresheet = app_state
        .scoresheet()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

    scoresheet.errors = scoresheet.errors.saturating_sub(1);
    let errors = scoresheet.errors;
    Ok(ReplaceDirector::with_target(
        "#penalties-errors",
        errors_row(true, errors).render(),
    ))
}

// TECHNICAL PENALTIES
#[tauri::command]
pub fn plus_technical(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let scoresheet = app_state
        .scoresheet()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

    scoresheet.tech_penalties += 1;
    let errors = scoresheet.tech_penalties;
    Ok(ReplaceDirector::with_target(
        "#penalties-technical",
        technical_row(true, errors).render(),
    ))
}
#[tauri::command]
pub fn sub_technical(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let scoresheet = app_state
        .scoresheet()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

    scoresheet.tech_penalties = scoresheet.tech_penalties.saturating_sub(1);
    let tech_penalties = scoresheet.tech_penalties;
    Ok(ReplaceDirector::with_target(
        "#penalties-technical",
        technical_row(true, tech_penalties).render(),
    ))
}

// ARTISTIC PENALTIES
#[tauri::command]
pub fn plus_artistic(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let scoresheet = app_state
        .scoresheet()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

    scoresheet.art_penalties += 1;
    let art_penalties = scoresheet.art_penalties;
    Ok(ReplaceDirector::with_target(
        "#penalties-artistic",
        artistic_row(true, art_penalties).render(),
    ))
}
#[tauri::command]
pub fn sub_artistic(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| screen_error("Could not increase error due to poisoned lock"))?;

    let scoresheet = app_state
        .scoresheet()
        .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

    scoresheet.errors = scoresheet.art_penalties.saturating_sub(1);
    let art_penalties = scoresheet.art_penalties;
    Ok(ReplaceDirector::with_target(
        "#penalties-artistic",
        artistic_row(true, art_penalties).render(),
    ))
}

