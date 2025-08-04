use std::sync::PoisonError;

use socket_manager::message::Message;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use super::message_types::server::Payload;
use super::message_types::{common, server};
use crate::commands::replace_director::{PageLocation, ReplaceDirector};
use crate::domain::starter::StarterResult;
use crate::state::{ApplicationState, ManagedApplicationState};
use crate::{debug, STATE, STORE_URI};

impl server::Trend {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        use crate::commands::replace_director::emit_page_prerendered;
        use crate::templates::scoresheet::header_trend;

        let state = handle.state::<ManagedApplicationState>();
        state
            .write(|app_state| {
                if let Some(starter) = app_state.starter_mut() {
                    starter.impose_trend(&self);
                }
            })
            .map_err(|_| FatalHandlerError::StateMissing)?;
        let header = header_trend(Some(self.score), Some(self.rank), false);
        let trend = hypertext::Renderable::render(&header);
        emit_page_prerendered(&handle, &PageLocation::HeaderTrend, trend.clone());
        emit_page_prerendered(&handle, &PageLocation::TotalScore, trend);
        Ok(())
    }
}

impl server::Reset {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        const THIRTY_SECONDS: chrono::Duration = chrono::Duration::seconds(30);
        if self.timestamp > chrono::Utc::now() - THIRTY_SECONDS {
            let state = handle.state::<ManagedApplicationState>();
            let _ = state.write(|app_state| {
                if let Some(starter) = app_state.starter_from_sheet_ulid_mut(&self.sheet_id) {
                    if let Some(scoresheet) = starter.scoresheets.first_mut() {
                        scoresheet.scores = vec![];
                        scoresheet.score = None;
                        starter.score = None;
                        starter.status = StarterResult::Upcoming;
                        scoresheet.errors = 0;
                        scoresheet.tech_penalties = 0;
                        scoresheet.art_penalties = 0;
                    }
                }
            });
            // TODO: Clear all
        }
        Ok(())
    }
}

impl server::AlterStarter {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        Ok(())
    }
}

impl server::Lock {
    pub(in crate::sockets) fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        let state = handle.state::<ManagedApplicationState>();
        state
            .write(|app_state| {
                if let Some(ref mut starter) = app_state.starter_mut() {
                    if starter.matches_sheet_ulid(&self.sheet_id) {
                        starter.impose_lock(&self);

                        // TODO: Emit to screen
                    }
                }
            })
            .map_err(FatalHandlerError::from)?;
        auto_state_saver::<ApplicationState>(&handle, STATE, |app_state| {
            if let Some(ref mut starter) = app_state.starter_mut() {
                if starter.matches_sheet_ulid(&self.sheet_id) {
                    starter.impose_lock(&self);
                }
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl common::Signal {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        Ok(())
    }
}

impl common::Status {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        Ok(())
    }
}
pub fn handle_ack(ulid: ulid::Ulid, handle: &tauri::AppHandle) {
    if let Ok(store) = handle.store(STORE_URI) {
        if let Some(payload) = store.get(STATE) {
            let res = serde_json::from_value::<Vec<Message<Payload>>>(payload);
            let filtered_messages = res.map_or_else(
                |_| Default::default(),
                |messages| {
                    messages
                        .into_iter()
                        .filter(|x| x.id != ulid)
                        .collect::<Vec<_>>()
                },
            );
            if let Err(err) = serde_json::to_value(filtered_messages).map(|v| store.set(STATE, v)) {
                debug!("{err:?}");
            } else {
                debug!(dim, "Ack {ulid}");
            }
        }
    }
}
pub fn handle_application_state(a: Payload) {
    debug!(green, "App State {a:?}");
}

fn auto_state_saver<R>(
    handle: &tauri::AppHandle,
    key: &str,
    f: impl FnOnce(&mut R) -> Result<(), HandlerError>,
) -> Result<(), HandlerError>
where
    R: serde::de::DeserializeOwned + serde::ser::Serialize,
{
    let store = handle.store(STORE_URI).map_err(FatalHandlerError::from)?;
    let mut stored_state =
        serde_json::from_value::<R>(store.get(key).ok_or(HandlerError::MissingStoreValue)?)
            .map_err(HandlerError::SerdeJson)?;
    let ret = f(&mut stored_state);
    if ret.is_ok() {
        let res = serde_json::to_value(stored_state).map_err(HandlerError::SerdeJson)?;
        store.set(key, res);
    }
    ret
}

type HandlerResult = Result<(), HandlerError>;
#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("The trend handler return an error")]
    Trend,
    #[error(transparent)]
    Fatal(#[from] FatalHandlerError),
    #[error("Store is missing a value which was expected")]
    MissingStoreValue,
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}
#[derive(thiserror::Error, Debug)]
pub enum FatalHandlerError {
    #[error("The state which was being accessed is missing")]
    StateMissing,
    #[error("Poisoned state")]
    StatePosioned(#[from] PoisonError<ManagedApplicationState>),
    #[error("A poisoned lock stopped the program")]
    PoisonedLock,
    #[error(transparent)]
    TauriStore(#[from] tauri_plugin_store::Error),
}
impl From<ReplaceDirector> for FatalHandlerError {
    fn from(_: ReplaceDirector) -> FatalHandlerError {
        FatalHandlerError::PoisonedLock
    }
}
