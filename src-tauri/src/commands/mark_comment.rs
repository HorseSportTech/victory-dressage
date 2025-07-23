use tauri::Emitter;

use crate::{
    domain::{
        competition::Competition,
        dressage_test::{DressageTest, Exercise},
        scoresheet::{ScoredMark, Scoresheet},
    },
    state::ManagedApplicationState,
    templates::scoresheet::{
        attempt_input, attempt_input_with_score, get_attempt_buttons, get_main_mark_input,
    },
};

use super::{
    replace_director::{
        emit_page, emit_page_outer, emit_page_prerendered, ReplaceDirector, ResponseDirector,
    },
    PAGE_UPDATE,
};

#[tauri::command]
pub async fn input_mark(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
    index: &str,
) -> Result<String, String> {
    let index = index.parse::<u16>().expect("Index should be parsable");
    let movement = {
        let mut app_state = state.read().map_err(|_| String::new())?;
        get_current_movement(app_state.get_test().expect("No test"), index)
    };
    // parse value
    if value.ends_with(".") {
        if let Ok(num) = value[0..value.len() - 1].parse::<f64>() {
            return Ok(value.to_string());
        }
    } else if let Ok(mut num) = value.parse::<f64>() {
        println!("{:?} {}", movement, num);
        if num >= movement.min as f64 {
            while num > movement.max as f64 {
                num /= movement.max as f64;
            }
            println!(
                "{}",
                f64::round(num * 10.0) % f64::round(movement.step as f64 * 10.0)
            );
            if f64::round(num * 10.0) % f64::round(movement.step as f64 * 10.0) == 0.0 {
                {
                    let mut app_state = state.write().map_err(|_| String::new())?;
                    get_current_scored_exercise(app_state.scoresheet_mut(), index).mk = Some(num);
                }
                // send to message que

                // calculate
                let trend = {
                    let app_state = state.read().map_err(|_| format!("{num}"))?;
                    let scoresheet = app_state.scoresheet();
                    scoresheet.map_or(0.0, |x| {
                        x.trend(app_state.get_test().expect("There must be test"))
                    })
                };
                let trend = crate::templates::scoresheet::header_trend(Some(trend), Some(0), true);
                let trend = hypertext::Renderable::render(&trend);
                emit_page_prerendered(&handle, "#header-trend", trend.clone());
                emit_page_prerendered(&handle, "#total-score", trend);

                //return to user
                return Ok(format!("{num}"));
            }
        }
    }
    // otherwise, change mark to nothing and reset to user
    {
        let mut app_state = state.write().map_err(|_| String::new())?;
        get_current_scored_exercise(app_state.scoresheet_mut(), index).mk = None;
    }
    return Ok(String::new());
}

#[tauri::command]
pub async fn input_comment(
    state: tauri::State<'_, ManagedApplicationState>,
    value: &str,
    index: &str,
) -> Result<String, String> {
    let mut app_state = state
        .write()
        .or_else(|_| {
            state.clear_poison();
            state.write()
        })
        .map_err(|_| String::new())?;
    let index = index.parse::<u16>().expect("Index should be parsable");
    let scored_exercise = get_current_scored_exercise(app_state.scoresheet_mut(), index);

    // otherwise, change mark to nothing and reset to user
    scored_exercise.rk = if value == "" {
        None
    } else {
        Some(value.to_string())
    };
    return Ok(String::new());
}

fn get_current_movement(test: &DressageTest, index: u16) -> Exercise {
    test.movements
        .iter()
        .find(|x| x.number as u16 == index)
        .expect("No movement. Maybe this shouldn't be an expect")
        .clone()
}

fn get_current_scored_exercise<'a>(
    scoresheet: Option<&'a mut Scoresheet>,
    index: u16,
) -> &'a mut ScoredMark {
    match scoresheet {
        Some(sheet) => {
            let search_index = sheet.scores.iter().position(|s| s.nr == index);
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

    let (min, max, step) = {
        let mut app_state = state.write().map_err(|_| String::new())?;
        let movement = get_current_movement(app_state.get_test().expect("No test"), index);
        (movement.min, movement.max, movement.step)
    };
    // parse value
    if value.ends_with(".") {
        if let Ok(num) = value[0..value.len() - 1].parse::<f64>() {
            return Ok(value.to_string());
        }
    } else if let Ok(mut num) = value.parse::<f64>() {
        if num >= min as f64 {
            while num > max as f64 {
                num /= max as f64;
            }
            if num % step as f64 == 0.0 {
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
    return Ok(String::new());
}
#[tauri::command]
pub async fn confirm_attempt(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    value: &str,
    index: &str,
    attempt: &str,
) -> Result<String, String> {
    let mut app_state = state.write().map_err(|_| String::new())?;
    let index = index.parse::<u16>().expect("Index should be parsable");
    let attempt = attempt.parse::<usize>().expect("Index should be parsable");
    let movement = get_current_movement(app_state.get_test().expect("No test"), index);
    let scored_exercise = get_current_scored_exercise(app_state.scoresheet_mut(), index);
    // parse value
    if value.ends_with(".") {
        if let Ok(num) = value[0..value.len() - 1].parse::<f64>() {
            return Ok(value.to_string());
        }
    } else if value == "" && attempt < scored_exercise.at.len() {
        scored_exercise.at.swap_remove(attempt);
    } else if let Ok(mut num) = value.parse::<f64>() {
        if num >= movement.min as f64 {
            while num > movement.max as f64 {
                num /= movement.max as f64;
            }
            if num % movement.step as f64 == 0.0 {
                if attempt < scored_exercise.at.len() {
                    scored_exercise.at[attempt] = num;
                } else {
                    scored_exercise.at.push(num);
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
    scored_exercise.mk = if scored_exercise.at.len() > 0 {
        Some(
            f64::round(
                (scored_exercise.at.iter().fold(0., |mut sum, num| {
                    sum += num;
                    sum
                }) / scored_exercise.at.len() as f64)
                    * (1.0 / movement.step) as f64,
            ) / (1.0 / movement.step) as f64,
        )
    } else {
        None
    };
    // calculate

    let export_mark = scored_exercise
        .mk
        .map_or_else(String::new, |x| format!("{x:.1}"));
    emit_page(
        &handle,
        &format!("tr[data-index='{}'] .attempt-track", index),
        get_attempt_buttons(scored_exercise),
    );
    emit_page_outer(
        &handle,
        &format!(
            "tr[data-index='{}'] input.exercise-input[data-input-role='attempt']",
            index
        ),
        attempt_input(index as u8, scored_exercise.at.len()),
    );
    emit_page_prerendered(
        &handle,
        &format!(
            "tr[data-index='{}'] input.exercise-input[data-input-role='mark']",
            index
        ),
        hypertext::Rendered(export_mark),
    );
    //trend
    drop(app_state);
    let app_state = state.read().map_err(|_| String::new())?;
    let scoresheet = app_state
        .scoresheet()
        .cloned()
        .expect("There to be a scoresheet");
    let trend = scoresheet.trend(app_state.get_test().expect("No test"));
    let trend = crate::templates::scoresheet::header_trend(Some(trend), Some(0), true);
    let trend = hypertext::Renderable::render(&trend);
    emit_page_prerendered(&handle, "#header-trend", trend.clone());
    emit_page_prerendered(&handle, "#total-score", trend);
    drop(app_state);

    return Ok(String::new());
}

#[tauri::command]
pub async fn edit_attempt(
    state: tauri::State<'_, ManagedApplicationState>,
    handle: tauri::AppHandle,
    attempt: usize,
    index: u16,
) -> ResponseDirector {
    let mut app_state = state.write().map_err(|_| ReplaceDirector::none())?;
    // parse value
    let scored_exercise = get_current_scored_exercise(app_state.scoresheet_mut(), index);
    let Some(attempt_score) = scored_exercise.at.get(attempt) else {
        return Err(ReplaceDirector::none());
    };
    emit_page_outer(
        &handle,
        &format!(
            "tr[data-index='{}'] input.exercise-input[data-input-role='attempt']",
            index
        ),
        attempt_input_with_score(index as u8, attempt, Some(*attempt_score)),
    );
    return Ok(ReplaceDirector::with_target_outer(
        &format!(
            "tr[data-index='{}'] input[data-input-mode='attempt']",
            index
        ),
        hypertext::Rendered(format!("{attempt_score}")),
    ));
}
