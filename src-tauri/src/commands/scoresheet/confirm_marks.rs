use hypertext::Renderable;

use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::sockets::message_types::{AppSocketMessage, CompetitionMessage};
use crate::sockets::messages::SocketMessage;
use crate::state::ManagedApplicationState;
use crate::templates::scoresheet::get_confirm_or_signature;

#[tauri::command]
pub async fn confirm_marks<'x>(
    state: tauri::State<'x, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    {
        let mut app_state = match state.write() {
            Ok(a) => a,
            Err(_) => {
                state.clear_poison();
                state.write().map_err(|_err| ReplaceDirector::none())?
            }
        };

        let ref mut starter = app_state
            .starter
            .as_mut()
            .ok_or_else(|| ReplaceDirector::none())?;

        let scoresheet = match starter.scoresheets.first_mut() {
            Some(x) => {
                x.locked = true;
                x.clone()
            }
            None => return Err(ReplaceDirector::none()),
        };

        tauri::async_runtime::spawn(async move {
            SocketMessage::generate(
                handle,
                AppSocketMessage::Competition(CompetitionMessage::Lock {
                    locked: true,
                    sheet_id: ulid::Ulid::from_string(&scoresheet.id.id())
                        .expect("This should be a ulid"),
                    scores: None,
                }),
            )
            .await
        });
    }
    let ref app_state = match state.read() {
        Ok(a) => a,
        Err(_) => {
            state.clear_poison();
            state.read().map_err(|_err| ReplaceDirector::none())?
        }
    };
    let competition = app_state
        .competition
        .as_ref()
        .ok_or_else(|| ReplaceDirector::none())?;
    let judge = competition
        .jury
        .first()
        .ok_or_else(|| ReplaceDirector::none())?;
    let starter = app_state
        .starter
        .as_ref()
        .ok_or_else(ReplaceDirector::none)?;
    let scoresheet = starter
        .scoresheets
        .first()
        .ok_or_else(ReplaceDirector::none)?;
    return Ok(ReplaceDirector::with_target(
        "#confirm-marks",
        get_confirm_or_signature(scoresheet, judge).render(),
    ));
}
