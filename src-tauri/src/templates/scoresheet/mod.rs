pub mod start_list_bar;
pub mod warnings;
pub mod warnings_bar;

use super::html_elements;
use super::TxAttributes;
use hypertext::{rsx, rsx_move, GlobalAttributes, Lazy, Renderable};
use hypertext::{rsx_static, Raw};

use crate::commands::alert_manager::AlertManager;
use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::domain::competition::Competition;
use crate::domain::dressage_test::{Exercise, TestSheetType};
use crate::domain::ground_jury_member::GroundJuryMember;
use crate::domain::ground_jury_member::JuryAuthority;
use crate::domain::scoresheet::{ScoredMark, Scoresheet};
use crate::domain::starter::StarterResult;
use crate::state::ManagedApplicationState;

use super::error::screen_error;

pub async fn scoresheet(
    state: tauri::State<'_, ManagedApplicationState>,
    alert_manager: tauri::State<'_, AlertManager>,
) -> ResponseDirector {
    let app_state = state
        .read()
        .map_err(|_| screen_error("Unexpected poisoned lock"))?;
    let competition = app_state
        .competition
        .as_ref()
        .ok_or_else(|| screen_error("Competition Not Found"))?;

    let show = app_state
        .show
        .as_ref()
        .ok_or_else(|| screen_error("Show Not Found"))?;

    let starter = &app_state
        .starter
        .as_ref()
        .ok_or_else(|| screen_error("Starter not found"))?;

    let scoresheet = starter
        .scoresheets
        .clone()
        .into_iter()
        .next()
        .ok_or_else(|| screen_error("Scoresheet not found for this competitor"))?;

    let athlete_name = format!(
        "{} {}",
        starter.competitor.first_name, starter.competitor.last_name
    );
    let horse_name = starter.competitor.horse_name.to_string();
    let judge = competition
        .jury
        .first()
        .ok_or_else(|| screen_error("Judge not found"))?;

    let test = competition
        .tests
        .clone()
        .into_iter()
        .next()
        .ok_or_else(|| screen_error("Testsheet not found"))?;

    let test_name = competition
        .tests
        .first()
        .as_ref()
        .and_then(|x| Some(x.name.as_str()))
        .unwrap_or("Default test");

    let scoresheet_row_html = scoresheet_rows(test.movements.clone(), scoresheet.clone(), &judge);
    let warnings = warnings::get_warnings(alert_manager);

    Ok(ReplaceDirector::page(rsx! {
	<main
		class="scoresheet"
		id="scoresheet"
		style="display:grid; grid: 'header' 5em 'body' 1fr;"
		data-exercise-comment-last=judge.judge.prefs.comment_last
	>
		<header
			style="grid-area: header; display:flex; flex:row; border-block-end: 0.2rem solid var(--theme);
				box-shadow: 0 0.2rem 0.2rem #0003; background:var(--foreground);
				position:relative"
		>
			<section
				style="flex: 0 1 100%; padding: 0.5rem 0 0.5rem 0.5rem;display:flex;
					flex-direction:column;justify-content:center"
			>
				<h1 id="athlete-name">{&athlete_name}</h1>
				<h2 id="horse-name">
					<span style="color:var(--theme);font-weight:bold">{&starter.competitor.comp_no}</span>
					<span style="padding-inline:0.5rem">|</span>
					<span>{&horse_name}</span>
				</h2>
				<h3 id="test-name" style="font-size:var(--text-info);">{test_name}</h3>
			</section>
			<section
				style="position:fixed; display:block; inset 0 0 auto 0; text-align:center"
			>
				<div id="clock">Clock</div>
				<style onload="const clock = document.getElementById('clock');function setClock () {const date = new Date();
				clock.innerHTML = `${date.getHours()}:${date.getMinutes()?.toString().padStart(2, '0')}<span style='color:darkgrey'>:${date.getSeconds()?.toString().padStart(2, '0')}</span>`;}
				setClock();setInterval(setClock, 500);"></style>
			</section>
			<section id="timing-category">
				{get_timing_section(&competition, &judge)}
			</section>
			<section
				style="flex: 0 1 100%; display:flex; justify-content: end; align-items:center;
				padding-inline-end:1rem"
			>
				<div style="text-align:end; margin-inline-end: 1rem;">
					<output id="header-trend">{ if !judge.judge.prefs.hide_trend {
						Some(header_trend(scoresheet.score, scoresheet.rank, false))
						} else {None} }</output>

					<h3 style="font-size:0.6rem;">{ format!("{} {}", judge.judge.first_name, judge.judge.last_name) }</h3>
				</div>
				<div style="--color: lightgrey;--size: 3rem;width:var(--size);height:var(--size);display:flex;justify-content:center;
					align-items:center;font-size:calc(var(--size) / 1.5);font-weight:bold;border-radius:calc(var(--size) / 6);color:var(--color);
					border:3px solid var(--color);"
				>{ judge.position.to_string() }</div>
			</section>
			<svg
				viewBox="0 0 100 1"
				style="position: absolute; bottom: -1px; width: 100vw"
			>
				<path
					d="M0,0L60+0.5+100-0V1H0z"
					fill="color-mix(in srgb, transparent 80%, var(--theme))"
				></path>
			</svg>
		</header>
		<main
			style="overflow-y: scroll; overflow-x: clip; grid-area:body; background: var(--background)"
		>
			<form id="page"
				style="margin:0.6rem; margin-inline-end:0.2rem; background: var(--foreground);min-height: calc(100% - calc(2*1rem));
					padding: .7rem; box-sizing: border-box;"
			>
				<table style="width: 100%; border-collapse:collapse; table-layout:fixed">
					<colgroup>
						<col style="width: 1rem"/>
						<col style="width: 45%"/>
						<col style="inline-size: 3.1rem"/>
						<col style="width: 1.2rem"/>
						<col style="width:auto"/>
					</colgroup>
					<tr style="max-height: 2rem; font-weight: 500 !important; font-size:var(--text-info); height:1px;">
						<th colspan="2" style="width:clamp(14rem, 50%, 18rem)">Test</th>
						<th>Mark</th>
						<th style="height:inherit; font-weight:500">
							<svg viewBox="0 0 10 40" style="width:auto;height:2.8rem"
									preserveAspectRatio="xMidYMid meet">
								<g transform="rotate(-90) translate(-40 0) scale(0.75 1)">
									<text
										x="0"
										y="9"
										font-size="10px"
										transform-origin="center"
										font-weight="bold"
										text-anchor="center"
									>Coefficient</text>
								</g>
							</svg>
						</th>
						<th>Remark</th>
					</tr>
					{&scoresheet_row_html}
					<tr style="margin-top: 1rem">
						<td
							colspan="2"
							style="font-size:0.7rem; text-align: right; border:none; padding:1rem var(--padding);
								font-style: italic;align-content:start;"
						>
							"You can edit the comments even after confirming marks"
						</td>
						<td
							colspan="3"
							style="vertical-align: center; text-align: start; border:none"
							id="confirm-marks"
						>
                        {get_confirm_or_signature(&scoresheet, judge)}
						</td>
					</tr>
				</table>

				<div class="final-boxes" style="inline-size: 100%; display:grid; grid: repeat(3, auto) / 15% 11rem 1fr">
					<div style="font-size:var(--text-info); font-weight:bold; grid-column: 3 / 4">"Summary remarks"</div>
					<div
						style="border: 1px solid black; align-items:center;
							display:flex;padding:var(--padding); font-size:var(--text-info);"
					>Deductions</div>
					<div
						style="border: 1px solid black; border-width: 1px 1px 1px 0;
							align-items:center; display:flex; justify-content:end;padding:var(--padding);
							font-size:var(--text-info);"
					>TODO: -0%</div>
					<div
						style="border: 1px solid black; border-width: 1px 1px 1px 0;
							grid-row: 2 / 4; grid-column: 3 / 4;"
					>
						<textarea
							id="final-remark"
							style="appearance: none; -webkit-appearance:none; margin: 0 0; padding:0;
								outline: none; border: none;
								min-block-size: 100%; inline-size: 100%; box-sizing: border-box;
								display:block; resize: none;
								font-size: 1rem;
								padding:var(--padding)"
							rows="4"
							onkeyup="if (this.scrollHeight > this.clientHeight) this.style.minHeight = `${this.scrollHeight}px`"
						>{scoresheet.summary.as_ref()}</textarea>
					</div>
					<div
						style="border: 1px solid black; border-width: 0 1px 1px 1px;
							align-items:center; display:flex; padding:var(--padding);
							font-size:var(--text-info);"
					>Your score</div>
					<div
						id="total-score"
						style="border: 1px solid black; border-width: 0 1px 1px 0; font-size:var(--text-jumbo);
							font-weight:bold; align-items:center; display:flex; justify-content: end;
							padding:var(--padding)"
					>{if !judge.judge.prefs.hide_trend {
						Some(format_score(scoresheet.score))
					} else {None}
					}</div>
				</div>
			</form>
			<footer>
				<h2>"Judges’ notes "<span>"(These are only visible to you)"</span></h2>
				<textarea id="private-notes" rows="3">{scoresheet.notes.as_ref()}</textarea>
			</footer>
			{start_list_bar::start_list_bar(&show, &competition.starters, &judge, &starter.id)}
			{warnings_bar::warnings_bar(&test, &starter, &scoresheet)}
			<aside id="alerts-and-warnings" style="top:6rem; left:2rem; position:fixed;">
                {&warnings}
			</aside>
		</main>
	</main>
	}.render()))
}

fn format_score(score: Option<f64>) -> String {
    match score {
        Some(s) => format!("{s:.3}"),
        None => String::new(),
    }
}

pub fn header_trend<'a>(
    score: Option<f64>,
    rank: Option<u16>,
    provisional: bool,
) -> hypertext::Raw<String> {
    let score = format_score(score);
    let rank = match provisional {
        true => "Provisional".to_string(),
        false => format!("Rank {:.0}", rank.unwrap_or_default().clone()),
    };

    hypertext::Raw(rsx_move!{
		<div
			style=format!(
				"margin-right:-0.9rem; font-size:1.5rem;{}",
				if provisional {"color:hsl(0,100%,33%)"} else {""}
			)
		>{ &score }<span style="font-size:1rem; transform-origin:left; scale:.75 1; color:hsl(0,0,90%)">%</span>
		</div>
		<div style=format!("font-size:var(--text-info); {}", if provisional {"color:hsl(0,100%,33%)"} else {""})
			>{format!("{}", rank)}
		</div>
	}.render().into_inner())
}

pub fn scoresheet_rows<'b, 'c>(
    movements: Vec<Exercise>,
    mut scoresheet: Scoresheet,
    _judge: &GroundJuryMember,
) -> Lazy<impl Fn(&mut String) + 'b> {
    let marked_exercises = zip_exercise_and_marks(movements, scoresheet.scores.drain(..).collect());

    rsx_move! {
        @for (x, marked_exercise) in marked_exercises.iter() {
        <tr class="movement-row-type" data-row-type=x.category.to_string()>
            <td colspan="4">{x.category.to_string()}" Movements"</td>
        </tr>
        <tr style="block-size:1px; font-size:var(--text-info)" data-row-type=x.category.to_string()>
            <td class="exercise-number">{x.number}.</td>
            <td class="exercise-exercise" style="padding:0; block-size:inherit;">
                <table style="block-size: 100%; inline-size:100%;border:none; table-layout: fixed;">
                <colgroup>
                    <col style="width:3.5rem"/>
                    <col style="width:auto"/>
                </colgroup>
                @for l in x.lines.iter() {
                    <tr>
                        <td style="border:none;text-align:center">{&l.letter}</td>
                        <td style="border:none">{&l.description}</td>
                    </tr>
                }
                </table>
            </td>
            <td class="exercise-mark" style="height:inherit; box-sizing: border-box;padding:0">
                <input
                    type="text"
                    class="exercise-input"
                    data-index=x.number
                    size="2"
                    data-input-role="mark"
                    value=marked_exercise.clone().map_or(String::new(), |sm|sm.mk
                        .map_or(String::new(), |mk|format!("{:.1}", f64::round(mk*(x.max/x.step) as f64)/(x.max/x.step) as f64)))
                    oninput=format!("window.invoke('input_mark', {{value:this.value, index:this.dataset.index}}).then((e) => {{this.value = e}})")
                >
            </td>
            <td
                class="exercise-coefficient"
                style="text-align:center; vertical-align: center;"
            >{if x.coefficient != 1.0 {x.coefficient.to_string()} else {"".to_string()}}</td>
            <td class="exercise-remark" style="block-size:inherit; padding: 0; box-sizing: border-box;">
                <textarea
                    class="exercise-input"
                    data-index=x.number
                    data-input-role="remark"
                    oninput="if (this.clientHeight < this.scrollHeight) this.style.minHeight = this.scrollHeight+'px';
                        window.invoke('input_comment', {value:this.value, index:this.dataset.index});"
                >@if let Some(x) = marked_exercise.clone() {{x.rk.clone()}}</textarea>
            </td>
        </tr>
        }
    }
}

fn zip_exercise_and_marks(
    mut exercises: Vec<Exercise>,
    mut marks: Vec<ScoredMark>,
) -> Vec<(Exercise, Option<ScoredMark>)> {
    exercises.sort_by_key(|e| e.number);
    marks.sort_by_key(|e| e.nr);
    let mut marks_iter = marks.into_iter();
    exercises
        .into_iter()
        .map(|x| {
            let mark = marks_iter.next();
            if mark.as_ref().is_some_and(|m| m.nr == x.number as u16) {
                return (x, mark);
            }
            return (x, None);
        })
        .collect()
}

pub fn errors_row(has_errors: bool, errors: u8) -> Lazy<impl Fn(&mut String)> {
    let errors_disabled = errors == 0;
    rsx_move! {
        @if has_errors{
            <label>"Errors of course"</label>
            @match errors_disabled {
                true => {<button tx-command="sub_error" disabled>"-"</button>},
                false => {<button tx-command="sub_error" >"-"</button>}
            }

            <input type="number" value=errors disabled>
            <button tx-command="plus_error">"+"</button>
        }
    }
}

pub fn technical_row(has_errors: bool, technical_penalties: u8) -> Lazy<impl Fn(&mut String)> {
    rsx_move! {
        @if has_errors {
            <label>"Technical Penalties"</label>
            @match technical_penalties > 0 {
                true => <button tx-command="sub_technical">"-"</button>
                false => <button tx-command="sub_technical" disabled>"-"</button>
            }
            <input type="number" value=technical_penalties disabled>
            <button tx-command="plus_technical">"+"</button>
        }
    }
}

pub fn artistic_row(has_errors: bool, artistic_penalties: u8) -> Lazy<impl Fn(&mut String)> {
    rsx_move! {
        @if has_errors {
            <label>"Artistic Penalties"</label>
            @match artistic_penalties > 0 {
                true => {<button tx-command="sub_artistic">"-"</button>},
                false => {<button tx-command="sub_artistic" disabled>"-"</button>},
            }
            <input type="number" value=artistic_penalties disabled>
            <button tx-command="plus_artistic">"+"</button>
        }
    }
}

pub fn status_selection<'b>(status: StarterResult) -> Lazy<impl Fn(&mut String) + 'b> {
    use StarterResult::*;
    rsx_move! {
        <select
            id="status-selector"
            tx-command="change_competitor_status"
            tx-trigger="change"
        >
        <optgroup label="Normal">
            @match status {
                InProgress(_)|Upcoming => {
                    <option value="InProgress" selected>"Normal"</option>
                }
                _ => { <option value="InProgress">"Normal"</option> }
            }
        </optgroup>
        <optgroup label="Did not finish">
            @match status {
                Retired => {<option value="Retired" selected>"Retired"</option>}
                _ => { <option value="Retired">"Retired"</option> }
            }
            @match status {
                Eliminated(_) => {<option value="Eliminated" selected>"Eliminated"</option>}
                _ => { <option value="Eliminated">"Eliminated"</option> }
            }
            @match status {
                Withdrawn => {<option value="Withdrawn" selected>"Withdrawn"</option>}
                _ => { <option value="Withdrawn">"Withdrawn"</option> }
            }
            @match status {
                NoShow => {<option value="NoShow" selected>"No Show"</option>}
                _ => { <option value="NoShow">"No Show"</option> }
            }
        </optgroup>
        </select>
        @if let Eliminated(ref value) = status {
        <select>
            <option value=value>Option</option>
        </select>
        }
    }
}

pub const BELL_BUTTON: Raw<&'static str> = rsx_static! {
    <button tx-command="ring_bell" class="bell-button">
        <svg viewBox="0 0 40 40" style="height:1rem; fill:currentColor">
            <path d="M20,0S30,0 32,15 35,20 40,35L40,37H20zM20,0S10,0 8,15 5,20 0,35L0,37H20zM17,37S20,46 24,37z"></path>
        </svg>
    </button>
};
pub fn get_timing_section<'a>(
    competition: &'a Competition,
    jury: &'a GroundJuryMember,
) -> Lazy<impl Fn(&mut String) + 'a> {
    let test = competition.get_test(jury);
    let [normal_countdown, music_countdown] = test.countdowns;
    return rsx_move! {
        @if let JuryAuthority::Chief | JuryAuthority::Shadow  = jury.authority {
            <div class="timing-button-row" id="countdown">
                {BELL_BUTTON}
                @if normal_countdown > 0 {
                <div id="normal-countdown">
                <button tx-command="start_normal_time">{normal_countdown}" sec"</button>
                </div>
                }
                @if music_countdown > 0 {
                <div id="music-countdown">
                <button tx-command="start_music_time">{music_countdown}" sec"</button>
                </div>
                }
            </div>
            @if test.test_type == TestSheetType::Freestyle {
                <div class="timing-button-row" id="test-time-countdown">
                    <button tx-command="start_test_time_limit">"Test time"</button>
                </div>
            }
        }
    };
}

pub fn get_confirm_or_signature<'a>(
    scoresheet: &'a Scoresheet,
    judge: &'a GroundJuryMember,
) -> Lazy<impl Fn(&mut String) + use<'a>> {
    rsx! {
        @if scoresheet.locked {
            <style onload="lockMarks"></style>
            <div style="display:flex">
                <div style="color:var(--theme);font-weight:bold">"Marks"<br/>"confirmed!"</div>
                <svg viewBox="0 0 200 100" style="display:flex;flex:1 1 auto"><path stroke="blue" fill="none" d=&judge.judge.signature></path></svg>
            </div>
        } @else {
            <button
                style="background:var(--theme); color:white; border-radius:0.25rem;
                    border:1px solid color-mix(in srgb, var(--theme) 92%, black);
                    font-size:var(--text-info); padding:var(--padding);
                    margin-block: 0.5rem 1rem"
                tx-command="confirm_marks"
                type="button"
            >
            "Confirm Marks"
            </button>
        }
    }
}
