use std::sync::{LockResult, PoisonError};

use tauri::Manager;

use super::message_types::{application, common, server};
use crate::commands::replace_director::PageLocation;
use crate::state::ManagedApplicationState;

impl server::Trend {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        use crate::commands::replace_director::emit_page_prerendered;
        use crate::templates::scoresheet::header_trend;

        let state = handle.state::<ManagedApplicationState>();
        state.read(|app_state| {
            let header = header_trend(Some(self.score), Some(self.rank), false);
            let trend = hypertext::Renderable::render(&header);
            emit_page_prerendered(&handle, &PageLocation::HeaderTrend, trend.clone());
            emit_page_prerendered(&handle, &PageLocation::TotalScore, trend);
        });

        Ok(())
    }
}

impl server::Reset {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        Ok(())
    }
}

impl server::AlterStarter {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
        Ok(())
    }
}

impl common::Lock {
    pub fn handle(self, handle: tauri::AppHandle) -> HandlerResult {
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

type HandlerResult = Result<(), HandlerError>;
#[derive(thiserror::Error, Debug)]
pub(in crate::sockets) enum HandlerError {
    #[error("The trend handler return an error")]
    Trend,
    #[error(transparent)]
    Fatal(FatalHandlerError),
}
#[derive(thiserror::Error, Debug)]
enum FatalHandlerError {
    #[error("The state which was being accessed is missing")]
    StateMissing,
    #[error(transparent)]
    StatePosioned(#[from] PoisonError<ManagedApplicationState>),
}
