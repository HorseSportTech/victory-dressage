use hypertext::Renderable;
use socket_manager::SocketManager;
use tauri::Manager;

use crate::commands::replace_director::{emit_page_outer, ReplaceDirector, ResponseDirector};
use crate::sockets::message_types::{AppSocketMessage, CompetitionMessage};
use crate::state::ManagedApplicationState;
use crate::templates::scoresheet::{
    get_confirm_or_signature, get_main_mark_input, zip_exercise_and_marks,
};

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

        let app_handle = handle.clone();
        tauri::async_runtime::spawn(async move {
            let manager = app_handle.try_state::<SocketManager<AppSocketMessage>>();
            if let Some(ref manager) = manager {
                manager.send(AppSocketMessage::Competition(CompetitionMessage::Lock {
                    locked: true,
                    sheet_id: ulid::Ulid::from_string(&scoresheet.id.id())
                        .expect("This should be a ulid"),
                    scores: None,
                }));
            };
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
    let scoresheet = app_state.scoresheet().ok_or_else(ReplaceDirector::none)?;
    let test = competition
        .tests
        .first()
        .ok_or_else(ReplaceDirector::none)?;
    let scores = zip_exercise_and_marks(test.movements.clone(), scoresheet.scores.clone());
    for (exercise, mark) in scores.into_iter() {
        emit_page_outer(
            &handle,
            &format!(
                "tr[data-index='{}'] .exercise-input[data-input-role='mark']",
                exercise.number
            ),
            get_main_mark_input(&mark, &exercise, &scoresheet),
        );
    }
    return Ok(ReplaceDirector::with_target(
        "#confirm-marks",
        get_confirm_or_signature(&scoresheet, judge).render(),
    ));
}
