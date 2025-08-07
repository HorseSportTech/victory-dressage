use hypertext::{html_elements, rsx, Renderable};
use tauri::Manager;

use crate::commands::replace_director::{
    emit_page, emit_page_outer, PageLocation, ReplaceDirector, ResponseDirector,
};
use crate::sockets::manager::ManagedSocket;
use crate::sockets::message_types::{application, common};
use crate::state::ManagedApplicationState;
use crate::templates::scoresheet::{
    get_confirm_or_signature, get_main_mark_input, missing_movements_dialog, zip_exercise_and_marks,
};

#[tauri::command]
pub async fn confirm_marks<'x>(
    state: tauri::State<'x, ManagedApplicationState>,
    handle: tauri::AppHandle,
) -> ResponseDirector {
    let sheet_id = state
        .write_async(|app_state| {
            let movements = app_state
                .get_test()
                .ok_or_else(ReplaceDirector::none)?
                .movements
                .clone();
            let starter = app_state.starter_mut().ok_or_else(ReplaceDirector::none)?;

            match starter.scoresheets.first_mut() {
                Some(scoresheet) => {
                    let mut unscored_movements = vec![];
                    movements.iter().for_each(|movement| {
                        let has_mark_scored = scoresheet
                            .scores
                            .iter()
                            .find(|score| score.number == movement.number as u16)
                            .is_some_and(|score| score.mark.is_some());
                        if !has_mark_scored {
                            unscored_movements.push(movement.number);
                        };
                    });
                    if unscored_movements.is_empty() {
                        scoresheet.locked = true;
                    } else {
                        let movements = unscored_movements
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>();
                        return Err(ReplaceDirector::with_target(
                            &PageLocation::MissingScoreAside,
                            missing_movements_dialog(movements).render(),
                        ));
                    }
                    Ok(scoresheet.id.ulid())
                }
                None => Err(ReplaceDirector::none()),
            }
        })
        .await??;

    let app_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let manager = app_handle.try_state::<ManagedSocket>();
        if let Some(ref manager) = manager {
            _ = manager
                .send(application::Payload::Competition(
                    application::CompetitionMessage::Lock(common::Lock {
                        locked: true,
                        sheet_id,
                        scores: None,
                    }),
                ))
                .await
                .map_err(|_| ReplaceDirector::none());
        };
    });
    let (scores, signature) = state
        .read_async(|app_state| {
            let competition = app_state.competition().ok_or_else(ReplaceDirector::none)?;
            let judge = competition.jury.first().ok_or_else(ReplaceDirector::none)?;
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
    Ok(ReplaceDirector::with_target(
        &PageLocation::ConfirmMarks,
        get_confirm_or_signature(true, signature).render(),
    ))
}
