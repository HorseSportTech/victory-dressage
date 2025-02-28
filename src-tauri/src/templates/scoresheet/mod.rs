pub mod warnings;
pub mod start_list_bar;
pub mod warnings_bar;

use hypertext::{rsx_move, Renderable};
use hypertext::{rsx, RenderIterator, GlobalAttributes};
use start_list_bar::start_list_bar;
use super::html_elements;
use super::TxAttributes;

use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::domain::dressage_test::Exercise;
use crate::domain::ground_jury_member::GroundJuryMember;
use crate::domain::scoresheet::{ScoredMark, Scoresheet};
use crate::domain::starter::StarterResult;
use crate::state::ManagedApplicationState;

use super::error::screen_error;


pub async fn scoresheet(
	state: tauri::State<'_, ManagedApplicationState>
) -> ResponseDirector {
	let app_state = state.read().map_err(|_| screen_error("Unexpected poisoned lock"))?;
	let competition = app_state.competition.as_ref()
		.ok_or_else(||screen_error("Competition Not Found"))?;

	let show = app_state.show.as_ref()
		.ok_or_else(||screen_error("Show Not Found"))?;

	let starter = &app_state.starter.as_ref()
		.ok_or_else(||screen_error("Starter not found"))?;

	let scoresheet = &starter.scoresheets.first()
		.ok_or_else(||screen_error("Scoresheet not found for this competitor"))?;


	let athlete_name = format!("{} {}", starter.competitor.first_name, starter.competitor.last_name);
	let horse_name = starter.competitor.horse_name.to_string();
	let judge = competition.jury.first()
		.ok_or_else(||screen_error("Judge not found"))?;

	let test = competition.tests.first()
		.ok_or_else(||screen_error("Testsheet not found"))?;

	let test_name = competition.tests.first().as_ref()
		.and_then(|x| Some(x.name.as_str()))
		.unwrap_or("Default test");

    Ok(ReplaceDirector::page(rsx! {
		<main
			class="scoresheet"
			style="block-size: 100vh; inline-size: 100vw; display:grid; grid: 'header' 5em 'body' 1fr;">
				<header
					style="grid-area: header; display:flex; flex:row; border-block-end: 0.2rem solid var(--theme);
						box-shadow: 0 0.2rem 0.2rem #0003; background:var(--foreground);
						position:relative"
				>
					<section
						style="flex: 0 1 100%; padding: 0.5rem 0 0.5rem 0.5rem;display:flex;
							flex-direction:column;justify-content:center"
					>
						<div id="athlete-name">{athlete_name}</div>
						<div id="horse-name">
							<span style="color:var(--theme);font-weight:bold">{&starter.competitor.comp_no}</span>
							<span style="padding-inline:0.5rem">|</span>
							<span>{horse_name}</span>
						</div>
						<div id="test-name" style="font-size:var(--text-info);">{test_name}</div>
					</section>
					<section
						style="flex: 0 1 100%; display:flex; justify-content: center"
					>
						<div id="clock">Clock</div>
						<style onload="const clock = document.getElementById('clock');function setClock () {const date = new Date();
						clock.innerHTML = `${date.getHours()}:${date.getMinutes()?.toString().padStart(2, '0')}<span style='color:darkgrey'>:${date.getSeconds()?.toString().padStart(2, '0')}</span>`;}
						setClock();setInterval(setClock, 500);"></style>
					</section>
					<section
						style="flex: 0 1 100%; display:flex; justify-content: end; align-items:center;
						padding-inline-end:1rem"
					>
						<div style="text-align:end; margin-inline-end: 1rem;">

							<div id="header-trend">{ if !judge.judge.prefs.hide_trend {
								Some(header_trend(scoresheet.score, scoresheet.rank, false))
							 } else {None} }</div>

							<div style="font-size:0.6rem;">{ format!("{} {}", judge.judge.first_name, judge.judge.last_name) }</div>
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
				<section id="page"
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
						{scoresheet_rows(&test.movements, &scoresheet, &judge)}
						<tr style="margin-top: 1rem">
							<td
								colspan="2"
								style="font-size:0.7rem; text-align: right; border:none; padding-inline:var(--padding);
									font-style: italic;"
							>
								"You can edit the comments even after confirming marks"
							</td>
							<td
								colspan="2"
								style="vertical-align: center; text-align: start; border:none"
                                id="confirm-marks"
							>
								<button 
									style="background:var(--theme); color:white; border-radius:0.25rem;
										border:1px solid color-mix(in srgb, var(--theme) 92%, black);
										font-size:var(--text-info); padding:var(--padding);
										margin-block: 0.5rem 1rem"
                                    tx-command="confirm_marks"
								>
								Confirm Marks
								</button>
							</td>
							<td style="border:none; vertical-align:center; text-align:center">TODO: Signature</td>
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
			</section>
			{start_list_bar(&show, &competition.starters, &judge, &starter.id)}
			{warnings_bar::warnings_bar(&test, &starter, &scoresheet)}
			<aside id="alerts-and-warnings" style="top:6rem; left:2rem; position:fixed;">
				<dialog open style="border-radius:var(--corner-size); border:0.1rem solid var(--theme); background:#fffd; inline-size:17rem">
					<div style="font-weight:bold; color:var(--theme); font-size:var(--text-input);">"Notifications"</div>
					{warnings::get_warnings(&scoresheet)}
				</dialog>
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

pub fn header_trend<'a>(score: Option<f64>, rank: Option<u16>, provisional: bool) -> hypertext::Raw<String> {
	let score = format_score(score);
	let rank = format!("Rank {:.0}", rank.unwrap_or_default().clone());

	hypertext::Raw(rsx_move!{
		<div
			style=format!(
				"margin-right:-0.9rem; font-size:1.5rem;{}",
				if provisional {"color:hsl(0,100%,33%)"} else {""}
			)
		>{ score }<span style="font-size:1rem; transform-origin:left; scale:.75 1; color:hsl(0,0,90%)">%</span>
		</div>
		<div style=format!("font-size:var(--text-info); {}", if provisional {"color:hsl(0,100%,33%)"} else {""})
			>{format!("{}", if provisional {"Provisional".to_string()} else {rank})}
		</div>
	}.render().into_inner())
}

pub fn scoresheet_rows<'b,'c>(
	movements: &'c Vec<Exercise>,
	scoresheet: &'b Scoresheet,
	judge: &GroundJuryMember,
) -> impl for<'a> FnOnce(&'a mut std::string::String) + use<'b, 'c> {
	let comment_last = judge.judge.prefs.comment_last;
	movements.into_iter().map(move |x| {
		let marked_exercise = scoresheet.scores.iter().find(|s|s.nr as u8 == x.number)
			.cloned()
			.unwrap_or_else(|| ScoredMark::new(x.number as u16));
		rsx_move!{
	<tr style="block-size:1px; font-size:var(--text-info)">
		<td class="exercise-number">{x.number}.</td>
		<td class="exercise-exercise" style="padding:0; block-size:inherit;">
			<table style="block-size: 100%; inline-size:100%;border:none; table-layout: fixed;">
			<colgroup>
				<col style="width:3.5rem"/>
				<col style="width:auto"/>
			</colgroup>
			{x.lines.iter().map(|l| rsx!{
			<tr>
				<td style="border:none;text-align:center">{&l.letter}</td>
				<td style="border:none">{&l.description}</td>
			</tr>
			}).render_all()}
			</table>
		</td>
		<td class="exercise-input" style="height:inherit; box-sizing: border-box;padding:0">
			<input
				type="text"
				class="exercise-mark-input"
				data-index=x.number
				style="block-size:100%; inline-size:100%; border:none; outline: none; box-sizing: border-box;
						text-align: center; font-size:var(--text-input); border-width:0; margin:0"
				size="2"
				value=marked_exercise.mk.clone().and_then(|mk|Some(f64::round(mk*(x.max/x.step) as f64)/(x.max/x.step) as f64))
				onkeydown=format!("if (event.key == 'Enter') {{
					event.preventDefault();
					document.querySelector(`.exercise-remark-input[data-index=\'{}\']`)?.focus();
				}};", if comment_last.clone() {x.number} else {x.number + 1})
				oninput=format!("window.invoke('input_mark', {{value:this.value, index:this.dataset.index}}).then((e) => {{this.value = e}})")
			>
		</td>
		<td
			class="exercise-coefficient"
			style="text-align:center; vertical-align: center;"
		>{if x.coefficient != 1.0 {x.coefficient.to_string()} else {"".to_string()}}</td>
		<td class="exercise-remark" style="block-size:inherit; padding: 0; box-sizing: border-box;">
			<textarea
				style="appearance: none; -webkit-appearance:none; margin: 0 0; height:3.5rem;
						outline: none; border: none;
						min-block-size: 100%; inline-size: 100%; box-sizing: border-box;
						display:block; resize: none;
						font-size:var(--text-input);
						padding:var(--padding); font-family:writing"
				class="exercise-remark-input"
				data-index=x.number
				oninput="if (this.clientHeight < this.scrollHeight) this.style.minHeight = this.scrollHeight+'px';
					window.invoke('input_comment', {value:this.value, index:this.dataset.index});"
				onkeydown=format!("if (event.key == 'Enter') {{
					event.preventDefault();
					document.querySelector(`.exercise-mark-input[data-index=\'{}\']`)?.focus();
				}}", if comment_last.clone() {x.number + 1} else {x.number})
			>{marked_exercise.rk.as_ref()}</textarea>
		</td>
	</tr>
	}}).render_all()
}


pub fn errors_row(has_errors: bool, errors: u8)  -> impl for<'a> FnOnce(&'a mut std::string::String,) {
	rsx_move!{
		{if has_errors{Some(rsx!{
		<label
			style="flex:1 0 100%"
		>"Errors of course"</label>
		{if errors > 0 {rsx!{
			<button
				style="flex:0 1 auto"
				tx-command="sub_error"
			>"<"</button>
		}} else {rsx!{
			<button
				style="flex:0 1 auto"
				tx-command="sub_error"
				disabled
			>"<"</button>
		}}}
		
		<input
			style="flex:1 0 auto"
			type="number"
			value=errors
			disabled
		>
		<button
			style="flex:0 1 auto"
			tx-command="plus_error"
		>">"</button>
	})} else {None}}
}
}

pub fn technical_row(has_errors: bool, technical_penalties: u8) -> impl for<'a> FnOnce(&'a mut std::string::String) {
	rsx_move!{
		{if has_errors {Some(rsx!{
		<label
			style="flex:1 0 100%"
		>"Technical Penalties"</label>
		{if technical_penalties > 0 {rsx!{
			<button
				style="flex:0 1 auto"
				tx-command="sub_technical"
			>"<"</button>
		}} else {rsx!{
			<button
				style="flex:0 1 auto"
				tx-command="sub_technical"
				disabled
			>"<"</button>
		}}}
		
		<input
			style="flex:1 0 auto"
			type="number"
			value=technical_penalties
			disabled
		>
		<button
			style="flex:0 1 auto"
			tx-command="plus_technical"
		>">"</button>
	})}else{None}}
}
}

pub fn artistic_row(has_errors: bool, artistic_penalties: u8) -> impl for<'a> FnOnce(&'a mut std::string::String) {
	rsx_move!{{if has_errors {Some(rsx!{
		<label
			style="flex:1 0 100%"
		>"Artistic Penalties"</label>
		{if artistic_penalties > 0 {rsx!{
			<button
				style="flex:0 1 auto"
				tx-command="sub_artistic"
			>"<"</button>
		}} else {rsx!{
			<button
				style="flex:0 1 auto"
				tx-command="sub_artistic"
				disabled
			>"<"</button>
		}}}
		
		<input
			style="flex:1 0 auto"
			type="number"
			value=artistic_penalties
			disabled
		>
		<button
			style="flex:0 1 auto"
			tx-command="plus_artistic"
		>">"</button>
	})}else{None}}
}
}

pub fn status_selection<'b>(status: &'b StarterResult) 
-> impl for<'a> FnOnce(&'a mut std::string::String,) + use<'b> {
	rsx_move!{<select
		tx-command="change_competitor_status"
		tx-trigger="change"
	>
		<optgroup label="Normal">
			{if let StarterResult::InProgress(_) = status{
				rsx!{<option value="InProgress" selected>"Normal"</option>}
			} else {rsx!{<option value="InProgress">"Normal"</option>}}}
		</optgroup>
		<optgroup label="Did not finish">
			{if let StarterResult::Retired = status{
				rsx!{<option value="Retired" selected>"Retired"</option>}
			} else {rsx!{<option value="Retired">"Retired"</option>}}}
			{if let StarterResult::Eliminated(_) = status{
				rsx!{<option value="Eliminated" selected>"Eliminated"</option>}
			} else {rsx!{<option value="Eliminated">"Eliminated"</option>}}}
			{if let StarterResult::Withdrawn = status{
				rsx!{<option value="Withdrawn" selected>"Withdrawn"</option>}
			} else {rsx!{<option value="Withdrawn">"Withdrawn"</option>}}}
			{if let StarterResult::NoShow = status{
				rsx!{<option value="NoShow" selected>"No Show"</option>}
			} else {rsx!{<option value="NoShow">"No Show"</option>}}}
		</optgroup>
	</select>}
}
