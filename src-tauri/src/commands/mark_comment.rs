use decimal::{dec, Decimal, RoundingMode};
use std::str::FromStr;

use crate::{
    debug,
    domain::{
        dressage_test::{DressageTest, Exercise},
        scoresheet::{ScoredMark, Scoresheet},
    },
    sockets::{manager::ManagedSocket, message_types::application::Payload},
    state::ManagedApplicationState,
    templates::scoresheet::{attempt_input, attempt_input_with_score, get_attempt_buttons},
};

use super::replace_director::{
    emit_page, emit_page_outer, emit_page_prerendered, PageLocation, ReplaceDirector,
    ResponseDirector,
};

#[tauri::command]
pub async fn input_mark(
    state: tauri::State<'_, ManagedApplicationState>,
    socket: tauri::State<'_, ManagedSocket>,
    handle: tauri::AppHandle,
    value: &str,
    index: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("Index should be parsable");
    let movement = state
        .read_async(move |app_state| {
            get_current_movement(app_state.get_test().expect("No test"), index)
        })
        .await
        .map_err(|_| String::new())?;
    // parse value
    let num = match Decimal::from_str(value).as_mut() {
        Ok(num) => 'parse_dec: {
            // in order to allow decimals, we need to return
            // this here and do not do any other processing
            if value.ends_with('.') {
                return Ok(value.to_string());
            }
            *num = num.to_precision(2); // <- Need to make sure there are more than 1 decimal
                                        // so that the answer is rounded properly
            debug!("{:?} {:?}", movement, num);
            if *num < movement.min {
                break 'parse_dec None;
            }
            while *num > movement.max {
                *num /= movement.max;
            }

            debug!("{num}, {} = {}", movement.max, *num % movement.step);
            if *num % movement.step != dec![0.0] {
                break 'parse_dec None;
            }

            // calculate
            let trend = state
                .read_async(|app_state| {
                    let scoresheet = app_state.scoresheet();
                    scoresheet.map_or(dec!(0.0), |x| {
                        x.trend(app_state.get_test().expect("There must be test"))
                    })
                })
                .await
                .ok();
            let trend = crate::templates::scoresheet::header_trend(trend, Some(0), true);
            let trend = hypertext::Renderable::render(&trend);
            emit_page_prerendered(&handle, &PageLocation::HeaderTrend, trend.clone());
            emit_page_prerendered(&handle, &PageLocation::TotalScore, trend);
            Some(num.clone())
        }
        Err(_) => None,
    };
    // otherwise, change mark to nothing and reset to user
    let (sheet_id, comment) = state
        .write_async(move |app_state| {
            let remark = {
                let sheet = app_state.scoresheet_mut();
                let score = get_current_scored_exercise_mut(sheet, index);
                score.mark = num;
                score.remark.clone()
            };
            let sheet = app_state.scoresheet_mut();
            (sheet.expect("Must have a scoresheet").id.ulid(), remark)
        })
        .await
        .map_err(|_| String::new())?;
    socket
        .send(Payload::mark(sheet_id, index, num, comment))
        .await;
    num.map(|x| x.to_string()).ok_or(String::new())
}

#[tauri::command]
pub async fn input_comment(
    state: tauri::State<'_, ManagedApplicationState>,
    value: String,
    index: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("Index should be parsable");
    state
        .write_async(move |app_state| {
            let scored_exercise =
                get_current_scored_exercise_mut(app_state.scoresheet_mut(), index);

            // otherwise, change mark to nothing and reset to user
            scored_exercise.remark = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        })
        .await
        .ok();
    Ok(String::new())
}

fn get_current_movement(test: &DressageTest, index: u16) -> Exercise {
    test.movements
        .iter()
        .find(|x| x.number as u16 == index)
        .expect("No movement. Maybe this shouldn't be an expect")
        .clone()
}

fn get_current_scored_exercise_mut(
    scoresheet: Option<&mut Scoresheet>,
    index: u16,
) -> &mut ScoredMark {
    match scoresheet {
        Some(sheet) => {
            let search_index = sheet.scores.iter().position(|s| s.number == index);
            if let Some(i) = search_index {
                sheet.scores.get_mut(i)
            } else {
                sheet.scores.push(ScoredMark::new(index));
                sheet.scores.last_mut()
            }
            .expect("We just either checked or added this")
        }
        None => panic!("No scoresheet. Maybe this shouldn't be an expect"),
    }
}

#[tauri::command]
pub async fn input_attempt(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
    index: &str,
    attempt: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("Index should be parsable");

    let (min, max, step) = state
        .read_async(move |app_state| {
            let movement = get_current_movement(app_state.get_test().expect("No test"), index);
            (movement.min, movement.max, movement.step)
        })
        .await
        .map_err(|_| "Could not get movement triple".to_string())?;
    // parse value
    if value.ends_with(".") {
        let trimmed = &value[0..value.len() - 1];
        if trimmed.parse::<f64>().is_ok() {
            return Ok(value.to_string());
        }
    } else if let Ok(mut num) = <Decimal as std::str::FromStr>::from_str(value) {
        if num >= min {
            while num > max {
                num /= max;
            }
            if num % step == dec![0.0] {
                if value.len() >= 3 {
                    _ = confirm_attempt(state.clone(), handle, value, &index.to_string(), attempt)
                        .await;
                }
                //return to user
                return Ok(format!("{num}"));
            }
        }
    }
    // otherwise, change mark to nothing and reset to user
    Ok(String::new())
}
#[tauri::command]
pub async fn confirm_attempt(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
    index: &str,
    attempt: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("Index should be parsable");
    let attempt = attempt.parse::<usize>().expect("Index should be parsable");
    let (movement, mut scored_exercise) = state
        .write_async(move |app_state| {
            let movement = get_current_movement(app_state.get_test().expect("No test"), index);
            let scored_exercise =
                get_current_scored_exercise_mut(app_state.scoresheet_mut(), index);
            (movement.clone(), scored_exercise.clone())
        })
        .await
        .map_err(|_| String::new())?;
    // parse value
    if value.ends_with(".") {
        let trimmed = &value[0..value.len() - 1];
        if trimmed.parse::<f64>().is_ok() {
            return Ok(value.to_string());
        }
    } else if value.is_empty() && attempt < scored_exercise.attempts.len() {
        scored_exercise.attempts.swap_remove(attempt);
    } else if let Ok(mut num) = <Decimal as std::str::FromStr>::from_str(value) {
        if num >= movement.min {
            while num > movement.max {
                num /= movement.max;
            }
            if num % movement.step == dec!(0.0) {
                if attempt < scored_exercise.attempts.len() {
                    scored_exercise.attempts[attempt] = num;
                } else {
                    scored_exercise.attempts.push(num);
                }
            } else {
                return Err(String::new());
            }
        } else {
            return Err(String::new());
        }
    }

    // TODO: This should respect the correct "fair rounding" rounding
    // techniques.
    // 1. Take all marks which cannot be evenly averaged.
    // 2. sort them by highest to lowest coefficient, then
    // 3. sort them by movement number
    // 4. alternately round up and down, starting with round up.
    // 5. remove any marks which have been manually set, and don't update these marks.
    //      This is important so that the judge doesn't have the marks dancing around
    //      after already been rounded just because they manually set a mark.
    // 6. Post updated marks back to the front-end.
    let step_scale = movement.step.scale();
    let preproccessed_mark =
        Decimal::average(&scored_exercise.attempts, step_scale, RoundingMode::Up);
    scored_exercise.mark = preproccessed_mark;

    // calculate
    let export_mark = scored_exercise
        .mark
        .map_or_else(String::new, |x| x.to_string());
    emit_page(
        &handle,
        &PageLocation::Any(format!("tr[data-index='{index}'] .attempt-track")),
        get_attempt_buttons(&scored_exercise),
    );
    emit_page_outer(
        &handle,
        &PageLocation::Any(format!(
            "tr[data-index='{index}'] input.exercise-input[data-input-role='attempt']",
        )),
        attempt_input(index as u8, scored_exercise.attempts.len()),
    );
    emit_page_prerendered(
        &handle,
        &PageLocation::Any(format!(
            "tr[data-index='{index}'] input.exercise-input[data-input-role='mark']",
        )),
        hypertext::Rendered(export_mark),
    );
    //trend
    let trend = state
        .read_async(|app_state| {
            let scoresheet = app_state
                .scoresheet()
                .cloned()
                .expect("There to be a scoresheet");
            scoresheet.trend(app_state.get_test().expect("No test"))
        })
        .await
        .ok();
    let trend = crate::templates::scoresheet::header_trend(trend, Some(0), true);
    let trend = hypertext::Renderable::render(&trend);
    emit_page_prerendered(&handle, &PageLocation::HeaderTrend, trend.clone());
    emit_page_prerendered(&handle, &PageLocation::TotalScore, trend);

    Ok(String::new())
}

#[tauri::command]
pub async fn edit_attempt(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    attempt: usize,
    index: u16,
) -> ResponseDirector {
    let attempt_score = state
        .write_async(move |x| {
            let scored_exercise = get_current_scored_exercise_mut(x.scoresheet_mut(), index);
            scored_exercise.attempts.get(attempt).cloned()
        })
        .await?
        .map_or_else(|| Err(ReplaceDirector::none()), Ok)?;
    emit_page_outer(
        &handle,
        &PageLocation::Any(row_from_idx(
            index,
            " input.exercise-input[data-input-role='attempt']",
        )),
        attempt_input_with_score(index as u8, attempt, Some(attempt_score)),
    );
    Ok(ReplaceDirector::with_target_outer(
        &PageLocation::Any(row_from_idx(index, "input[data-input-mode='attempt']")),
        hypertext::Rendered(attempt_score.to_string()),
    ))
}
fn row_from_idx(index: u16, specifier: &str) -> String {
    format!("tr[data-index='{index}'] {specifier}")
}
