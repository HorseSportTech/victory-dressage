use hypertext::Renderable;
use socket_manager::SocketManager;
use tauri::Manager;

use crate::commands::replace_director::{
    emit_page_outer, PageLocation, ReplaceDirector, ResponseDirector,
};
use crate::sockets::message_types::{application, common, server};
use crate::state::ManagedApplicationState;
use crate::templates::scoresheet::{
    get_confirm_or_signature, get_main_mark_input, zip_exercise_and_marks,
};

#[tauri::command]
pub async fn confirm_marks<'x>(
    state: tauri::State<'x, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let sheet_id = state
        .write_async(|app_state| {
            let ref mut starter = app_state
                .starter
                .as_mut()
                .ok_or_else(|| ReplaceDirector::none())?;

            match starter.scoresheets.first_mut() {
                Some(x) => {
                    x.locked = true;
                    Ok(x.id.ulid())
                }
                None => Err(ReplaceDirector::none()),
            }
        })
        .await??;

    let app_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let manager = app_handle.try_state::<SocketManager<application::Payload>>();
        if let Some(ref manager) = manager {
            manager.send(application::Payload::Competition(
                application::CompetitionMessage::Lock(common::Lock {
                    locked: true,
                    sheet_id,
                    scores: None,
                }),
            ));
        };
    });
    let (scores, signature) = state
        .read_async(|app_state| {
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
            Ok((
                zip_exercise_and_marks(test.movements.clone(), scoresheet.scores.clone()),
                judge.judge.signature.clone(),
            ))
        })
        .await??;
    for (exercise, mark) in scores.into_iter() {
        emit_page_outer(
            &handle,
            &PageLocation::Any(format!(
                "tr[data-index='{}'] .exercise-input[data-input-role='mark']",
                exercise.number
            )),
            get_main_mark_input(&mark, &exercise, true),
        );
    }
    return Ok(ReplaceDirector::with_target(
        &PageLocation::ConfirmMarks,
        get_confirm_or_signature(true, signature).render(),
    ));
}
