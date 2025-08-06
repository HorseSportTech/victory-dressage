use tauri::Manager;

use crate::{
    commands::replace_director::{ReplaceDirector, ResponseDirector},
    domain::SurrealId,
};

use super::ManagedApplicationState;

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ApplicationPage {
    Login,
    LoginJudge,
    Welcome,
    CompetitionList,
    Scoresheet(SurrealId),
    Settings,
    Preferences,
    FinalResult,
    Error,
}
impl ApplicationPage {
    pub fn set_location(self, handle: &tauri::AppHandle) -> ResponseDirector {
        let state = handle.state::<ManagedApplicationState>();
        state.write(|app_state| app_state.page = self)?;

        Ok(ReplaceDirector::none())
    }
}
