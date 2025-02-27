use hypertext::{html_elements, rsx, GlobalAttributes, Renderable};

use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::sockets::messages::SocketMessage;
use crate::state::ManagedApplicationState;
use crate::templates::TxAttributes;

pub async fn confirm_marks(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|err| Err(ReplaceDirector::none()))?;

    if let Some(ref mut starter) = app_state.starter {
        if let Some(ref mut scoresheet) = starter.scoresheets.first() {
            scoresheet.locked = true;
        };
    };
    tauri::async_runtime::spawn(async move {
        SocketMessage::generate(
            handle,
            CompetitionDTO::ScoresheetLock {
                locked: true,
                sid: starter.id,
            },
        )
        .await
    });
    // TODO: better handle the locking of marks and the signature here
    Ok(ReplaceDirector::with_target(
        "#confirm-marks",
        rsx! {
            <style onload="lockMarks"></style>
            <div style="color:var(--color-theme);font-weight:bold">"Marks confirmed!"</div>
        }
        .render(),
    ))
}
