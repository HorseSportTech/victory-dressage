use decimal::{dec, Decimal, RoundingMode};
use std::str::FromStr;
use tauri::Manager;

use crate::{
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

enum MarkState {
    Unprocessable,
    OutOfBounds,
    Incomplete(Decimal),
    Complete(Decimal),
}
#[tauri::command]
pub fn input_mark(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: String,
    index: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("Index should be parsable");
    state
        .read(move |app_state| app_state.score_debounces.cancel(index))
        .map_err(|_| "Err".to_string())?;
    let final_mark: MarkState = 'bounds: {
        if value.is_empty() {
            break 'bounds MarkState::Unprocessable;
        }
        let bytes = value.as_bytes();
        for char in bytes.iter() {
            match char {
                b'.' | b'-' | b'0'..=b'9' => (),
                _ => break 'bounds MarkState::Unprocessable,
            }
        }
        let movement = state
            .read(move |app_state| {
                get_current_movement(app_state.get_test().expect("No test"), index)
            })
            .map_err(|_| "Err".to_string())?;

        let Ok(mut mark) = Decimal::from_str(&value) else {
            break 'bounds MarkState::Unprocessable;
        };

        // check that mark conforms to the movement requirements
        if mark < movement.min || mark.scale() > movement.step.scale() {
            break 'bounds MarkState::OutOfBounds;
        }
        let corrected_decimal_place = mark >= movement.max; // if it's equal, it also cannot have any
                                                            // more added
        while mark > movement.max {
            mark = mark.safe_divide(dec!(10.0), 3).map_err(|_| String::new())?;
        }
        if mark % movement.step != dec!(0.0) {
            break 'bounds MarkState::OutOfBounds;
        }
        if corrected_decimal_place {
            MarkState::Complete(mark.to_precision(movement.step.scale()))
        } else {
            MarkState::Incomplete(mark.to_precision(movement.step.scale()))
        }
    };
    match final_mark {
        MarkState::Complete(mark) | MarkState::Incomplete(mark) => {
            let _ = state.write(move |app_state| {
                let sheet = app_state
                    .scoresheet_mut()
                    .expect("Should be able to get scoresheet. Maybe shouldn't be an expect");
                let score = get_current_scored_exercise_mut(sheet, index);
                score.mark = Some(mark);
            });
            calculate_trend_and_emit(&handle);
        }
        _ => (),
    }

    // send via socket if complete, or queue in the debounce to send shortly
    // if not complete. If Parse error, do nothing.
    // Then return the mark if complete, the input value if incomplete, and
    // a blank string if a parse error
    let handle = handle.clone();
    Ok(match final_mark {
        MarkState::Complete(mark) => {
            tauri::async_runtime::spawn(async move {
                parse_and_send_mark(handle, Some(mark), index).await;
            });
            mark.to_string()
        }
        MarkState::Incomplete(mark) => {
            _ = state.write(move |app_state| {
                app_state.score_debounces.debounce(
                    index,
                    std::time::Duration::from_millis(800),
                    move || {
                        tauri::async_runtime::spawn(async move {
                            parse_and_send_mark(handle, Some(mark), index).await;
                        });
                    },
                )
            });
            value
        }
        _ => String::new(),
    })
}

#[tauri::command]
pub async fn assent_mark(
    handle: tauri::AppHandle,
    value: &str,
    index: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("to get index");
    let mark = Decimal::from_str(value).map_err(|_| String::new())?;

    let state = handle.state::<ManagedApplicationState>();
    let movement = state
        .read(move |app_state| get_current_movement(app_state.get_test().expect("No test"), index))
        .map_err(|_| String::new())?;

    if mark >= movement.min && mark <= movement.max && mark % movement.step == dec!(0.0) {
        let _ = state.read(move |a| a.score_debounces.execute_immediately(index));
        return Ok(mark.to_string());
    }
    Ok(String::new())
}
fn calculate_trend_and_emit(handle: &tauri::AppHandle) {
    let state = handle.state::<ManagedApplicationState>();
    let trend = state
        .read(|app_state| {
            let scoresheet = app_state.scoresheet();
            scoresheet.map_or(dec!(0.0), |x| {
                let test = app_state.get_test().expect("There must be a test");
                x.calculate_trend(test)
            })
        })
        .ok();
    let trend = crate::templates::scoresheet::header_trend(trend, Some(0), true);
    let trend = hypertext::Renderable::render(&trend);
    emit_page_prerendered(&handle, &PageLocation::HeaderTrend, trend.clone());
    emit_page_prerendered(&handle, &PageLocation::TotalScore, trend);
}

pub async fn parse_and_send_mark(
    handle: tauri::AppHandle,
    mark: Option<Decimal>,
    index: u16,
) -> Option<()> {
    let socket = handle.state::<ManagedSocket>();
    let state = handle.state::<ManagedApplicationState>();

    calculate_trend_and_emit(&handle);
    let (sheet_id, comment) = state
        .write_async(move |app_state| {
            let sheet = app_state
                .scoresheet_mut()
                .expect("Should be able to get scoresheet. Maybe shouldn't be an expect");
            let remark = {
                let score = get_current_scored_exercise_mut(sheet, index);
                score.mark = mark;
                score.remark.clone()
            };
            (sheet.id.ulid(), remark)
        })
        .await
        .ok()?;
    emit_page_prerendered(
        &handle,
        &PageLocation::Any(format!("tr [data-input-role='mark'][data-index='{index}']")),
        hypertext::Rendered(mark.map_or(String::new(), |x| x.to_string())),
    );

    let _ = socket
        .send(Payload::mark(sheet_id, index, mark, comment))
        .await;
    Some(())
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
            let scored_exercise = get_current_scored_exercise_mut(
                app_state
                    .scoresheet_mut()
                    .expect("Should have the scoresheet. Maybe this shouldn't be expect"),
                index,
            );

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

fn get_current_scored_exercise_mut(sheet: &mut Scoresheet, index: u16) -> &mut ScoredMark {
    let search_index = sheet.scores.iter().position(|s| s.number == index);
    if let Some(i) = search_index {
        sheet.scores.get_mut(i)
    } else {
        sheet.scores.push(ScoredMark::new(index));
        sheet.scores.last_mut()
    }
    .expect("We just either checked or added this")
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
            let scored_exercise = get_current_scored_exercise_mut(
                app_state
                    .scoresheet_mut()
                    .expect("Scoresheet should exist. Maybe shouldn't be expect"),
                index,
            );
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
            scoresheet.calculate_trend(app_state.get_test().expect("No test"))
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
            let scored_exercise = get_current_scored_exercise_mut(
                x.scoresheet_mut()
                    .expect("Should get scoresheet. Maybe shouldn't be expect"),
                index,
            );
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
