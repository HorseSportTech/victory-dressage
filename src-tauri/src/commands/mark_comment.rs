use tauri::Emitter;

use crate::{domain::{competition::Competition, dressage_test::{DressageTest, Exercise}, scoresheet::{ScoredMark, Scoresheet}}, state::ManagedApplicationState};

use super::{replace_director::ReplaceDirector, PAGE_UPDATE};

#[tauri::command]
pub async fn input_mark(
	state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
	value: &str,
	index: &str,
) -> Result<String, String> {
	let mut app_state = state.write()
		.map_err(|_| String::new())?;
	let index = index.parse::<u16>().expect("Index should be parsable");
	let movement = get_current_movement(app_state.competition.as_ref(), index);
	let scored_exercise = get_current_scored_exercise(app_state.scoresheet(), index);
	// parse value
    if let Ok(mut num) = value.parse::<f64>() {
		if num >= movement.min as f64 {
			while num > movement.max as f64 {
				num /= movement.max as f64;
			};
			if num % movement.step as f64 == 0.0 {
				scored_exercise.mk = Some(num);
				// send to message que
				
				// calculate

					let scoresheet = app_state.scoresheet().cloned();
					drop(app_state);
					let app_state = state.read().map_err(|_|format!("{num}"))?;
					let trend = calculate_trend(
						scoresheet.as_ref(),
						get_current_test(app_state.competition.as_ref()),
					);
					handle.emit(
						PAGE_UPDATE,
						ReplaceDirector::with_target(
							"#header-trend",
							hypertext::Rendered(crate::templates::scoresheet::header_trend(Some(trend), Some(0), true).0)
						)
					).ok();
					drop(app_state);
				
				//return to user
				return Ok(format!("{num}"))
			}
		}
	}
	// otherwise, change mark to nothing and reset to user
	scored_exercise.mk = None;
	return Ok(String::new());
}

#[tauri::command]
pub async fn input_comment(
	state: tauri::State<'_, ManagedApplicationState>,
	value: &str,
	index: &str,
) -> Result<String, String> {
	let mut app_state = state.write()
		.or_else(|_| {
			state.clear_poison();
			state.write()
		})
		.map_err(|_| String::new())?;
	let index = index.parse::<u16>().expect("Index should be parsable");
	let scored_exercise = get_current_scored_exercise(app_state.scoresheet(), index);
    
	// otherwise, change mark to nothing and reset to user
	scored_exercise.rk = if value == "" {
		None
	} else {
		Some(value.to_string())
	};
	return Ok(String::new());
}


fn get_current_movement(
	competition: Option<&Competition>,
	index: u16,
) -> Exercise {
    let test = get_current_test(competition);
	test.movements.iter()
		.find(|x| x.number as u16 == index)
		.expect("No movement. Maybe this shouldn't be an expect")
		.clone()
}
fn get_current_test(competition: Option<&Competition>) -> &DressageTest {
	competition.as_ref()
		.and_then(|x| x.tests.first())
		.expect("No Test. Maybe this shouldn't be an expect")
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
			}.expect("We just either checked or added this")
		}
		None => panic!("No scoresheet. Maybe this shouldn't be an expect")
	}
}

fn calculate_trend(
	scoresheet: Option<&Scoresheet>,
	testsheet: &DressageTest,
) -> f64 {
	let Some(scoresheet) = scoresheet else {return 0.0};
	let mut total = 0.0;
	let mut max_total = 0.0;
	for movement in testsheet.movements.iter() {
		let Some(exercise) = scoresheet.scores.iter()
			.find(|x| x.nr == movement.number as u16) else {continue};
		total += exercise.mk.unwrap_or(0.0) * movement.coefficient as f64;
		max_total += movement.max * movement.coefficient;
	}
	return total / max_total as f64 * 100.0
}

