use hypertext::Renderable;
use serde::Deserialize;

use crate::{
    commands::replace_director::{PageLocation, ReplaceDirector, ResponseDirector},
    domain::starter::StarterResult,
    state::ManagedApplicationState,
    templates::{error::screen_error, scoresheet::status_selection},
};

#[tauri::command]
pub fn change_competitor_status(
    state: tauri::State<'_, ManagedApplicationState>,
    value: WrappedStatus,
) -> ResponseDirector {
    let WrappedStatus(value) = value;
    let status = state.write(|app_state| {
        let mut starter = app_state
            .starter
            .clone()
            .ok_or_else(|| screen_error("Could not increase error due to poisoned lock"))?;

        starter.status = value.clone();
        Ok(value)
    })??;
    Ok(ReplaceDirector::with_target(
        &PageLocation::StatusSelector,
        status_selection(status).render(),
    ))
}
#[derive(serde::Serialize)]
#[serde(transparent)]
pub struct WrappedStatus(StarterResult);
impl<'de> Deserialize<'de> for WrappedStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        Ok(match str.as_str() {
            "Eliminated" => WrappedStatus(StarterResult::Eliminated(String::from(""))),
            "Withdrawn" => WrappedStatus(StarterResult::Withdrawn),
            "NoShow" => WrappedStatus(StarterResult::NoShow),
            "Retired" => WrappedStatus(StarterResult::Retired),
            "Placed" => WrappedStatus(StarterResult::Placed(0)),
            "NotPlaced" => WrappedStatus(StarterResult::NotPlaced(0)),
            _ => WrappedStatus(StarterResult::InProgress(0)),
        })
    }
}
