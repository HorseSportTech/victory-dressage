use hypertext::{rsx, rsx_move, GlobalAttributes};
use crate::commands::warnings::manager::PositionedWarning;
use crate::domain::position::Position;
use crate::templates::html_elements;
use crate::domain::scoresheet::Scoresheet;



pub fn get_warnings<'b>(
	scoresheet: &'b Scoresheet,
) -> impl for<'a> FnOnce(&'a mut std::string::String,) + use<'b> {
	rsx!{
		<div style="display:flex; flex-wrap:nowarp;">
			<div style="inline-size:max-content; flex:1 0 33%; align-content:center" class="flash">"Error 1"</div>
			<div style="display:flex; flex-direction:row; gap:0.2rem">{warnings_row(&scoresheet.warning_manager.blood)}</div>
		</div>
		<style>"#alerts-and-warnings { & .flash {font-weight:bold;animation: flashing 300ms alternate-reverse infinite ease;}}"</style>
	}
}


fn warnings_row<'b>(warnings: &'b PositionedWarning) -> impl for<'a> FnOnce(&'a mut std::string::String,) + use<'b> {
	rsx_move!{
		<div style="flex:1 0 auto;">{if !warnings.get(Position::K) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:forestgreen">K</div>})}else{None}}</div>
		<div style="flex:1 0 auto;">{if !warnings.get(Position::E) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:brown">E</div>})}else{None}}</div>
		<div style="flex:1 0 auto;">{if !warnings.get(Position::H) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:orange">H</div>})}else{None}}</div>
		<div style="flex:1 0 auto;">{if !warnings.get(Position::C) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:cornflowerblue">C</div>})}else{None}}</div>
		<div style="flex:1 0 auto;">{if !warnings.get(Position::M) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:pink">M</div>})}else{None}}</div>
		<div style="flex:1 0 auto;">{if !warnings.get(Position::B) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:purple">B</div>})}else{None}}</div>
		<div style="flex:1 0 auto;">{if !warnings.get(Position::F) {Some(rsx!{<div style="width: 1em; height:1em; border:1px solid black; border-radius:var(--corner-size); padding:0.1rem; text-align:center; justify-items:center; font-weight:bold; color:white; background:tan">F</div>})}else{None}}</div>
	}
}