use hypertext::{rsx_move, GlobalAttributes, Lazy, Raw, Renderable};

use super::super::{html_elements, TxAttributes};
use crate::commands::replace_director::PageLocation;
use crate::templates::icons;
use crate::{
    domain::{
        ground_jury_member::{GroundJuryMember, JuryAuthority},
        show::Show,
        starter::Starter,
        SurrealId,
    },
    traits::Entity,
};

pub fn start_list_bar<'b, 'a>(
    show: &'b Show,
    start_list: &'b Vec<Starter>,
    judge: &'b GroundJuryMember,
    current_starter: &'b SurrealId,
) -> Lazy<impl Fn(&mut String) + use<'a, 'b>> {
    let show_id = show.get_id();
    rsx_move! {<aside>
        <button
            class="scoresheet-menu-button right"
            tx-open=PageLocation::StartlistMenu.href()
        >{icons::MENU}</button>
        <dialog
            id="startlist-menu"
            class="start-list"
            onclick="event.target==this && this.close()"
        >
            <div
                class="dialog-header"
                style="background:var(--foreground); border-radius:var(--corner-size);
				block-size: 100%; display:grid;
				grid: min-content 1fr / 1fr; box-sizing: border-box;"
            >
                <div
                    style="box-sizing: border-box; border-block-end: 0.2rem solid var(--theme);
                        padding:var(--padding); box-shadow: 0 0.2rem 0.2rem #0003">
                    <div
                        style="display:flex;flex-direction:row; gap:var(--padding); inline-size: 100%;
                        box-sizing:border-box; padding-inline:var(--padding); block-size: 1.9rem"
                    >
                        <button
                            onclick="document.getElementById('startlist-menu').close()"
                            style="border-radius:var(--corner-size); background: red; color:white; border:none;
							border:1px solid color-mix(in srgb, red 95%, black);
							outline:none;font-size:2rem;line-height:1.3rem; align-content:center;
							font-weight: 300; display:flex; flex: 0 0 auto; box-sizing: border-box;"
                        >{Raw("&times;")}</button>
                        <button
                            tx-goto="competition_list"
                            tx-id=&show_id
                            style="background:var(--theme);
							border:1px solid color-mix(in srgb, var(--theme) 95%, black);
							font-size:var(--text-info); border-radius:var(--corner-size);
							flex: 1 1 100%; color:white;"
                        >"Return to competitions"</button>
                    </div>
                    <div style="display:flex; width:100%; padding:var(--padding);box-sizing: border-box;">
                        <input
                            type="search"
                            placeholder="Search ID, horse name, athlete name"
                            style="flex: 1 1 100%; border-radius:var(--corner-size); box-sizing: border-box;
							outline: none; border: 1px solid var(--theme); font-size: 0.8rem;
							margin-block-start: .8rem; padding:var(--padding);
							min-height:1.8rem"
                            tx-command="filter_starters"
                            tx-trigger="input"
                        >
                    </div>
                </div>
                <div style="padding:calc(2 * var(--padding)); font-size:var(--text" id="starters-list">
                    {get_starters_list(start_list, current_starter, judge, None)}
                </div>
            </div>
        </dialog>
    </aside>
    }
}

pub fn get_starters_list<'a>(
    start_list: &'a Vec<Starter>,
    current_starter: &'a SurrealId,
    judge: &'a GroundJuryMember,
    filter_term: Option<String>,
) -> Lazy<impl Fn(&mut String) + 'a> {
    let mut competition_finished = true;
    let mut finished_starters: Vec<&Starter> = vec![];
    let mut upcoming_starters: Vec<&Starter> = vec![];
    for starter in start_list {
        if !starter.status.is_finished() {
            competition_finished = false;
        }
        if let Some(ref term) = filter_term {
            if !format!(
                "{} {}\n{}\n{}",
                starter.competitor.first_name,
                starter.competitor.last_name,
                starter.competitor.horse_name,
                starter.competitor.comp_no,
            )
            .to_lowercase()
            .contains(&term.to_lowercase())
            {
                continue;
            }
        }
        if starter.status.is_finished() {
            finished_starters.push(starter);
        } else {
            upcoming_starters.push(starter)
        }
    }
    let show_final = match competition_finished {
        true if judge.authority == JuryAuthority::Chief => FinalButton::ActiveFinalize,
        true => FinalButton::ActiveView,
        false if judge.authority == JuryAuthority::Chief => FinalButton::DisabledFinalize,
        false => FinalButton::DisabledView,
    }
    .render();

    let separator = !finished_starters.is_empty() && !upcoming_starters.is_empty();
    let finished = list(finished_starters, true, current_starter);
    let to_come = list(upcoming_starters, false, current_starter);
    rsx_move! {
        <ul style="margin:0; padding:0">{&finished}</ul>
        @if separator {
            <hr style="border: 0.1rem solid var(--theme);margin: 0.5rem 0;"/>
        }
        <ul style="margin:0; padding:0">{&to_come}</ul>
        {&show_final}
    }
}

fn list<'b>(
    starters: Vec<&'b Starter>,
    finished: bool,
    current_starter: &'b SurrealId,
) -> Lazy<impl Fn(&mut String) + use<'b>> {
    rsx_move! {
        @for x in starters.iter() {
            <li
                class=(if &x.id == current_starter {"selected"} else {""}).to_string()
                style=format!("padding:0.1rem; margin:0; display:block;border:1px solid;{}",
                    if finished {"border-color:var(--theme);"} else {"border-color:gainsboro;"}
                )>
                <button tx-command="choose_starter" tx-id=x.get_id()>
                    <div class="starter-select">
                        {Raw(if x.status.is_finished() {
                            rsx_move!{<div class="done-icon done">{x.status.list_abbreviation().to_string()}</div>}.render()
                        } else {
                            rsx_move!{<div class="done-icon">{x.status.list_abbreviation().to_string()}</div>}.render()
                        })}
                        <div>{format!("{} {}", x.competitor.first_name, x.competitor.last_name)}</div>
                        <div>{x.score_or_number()}</div>
                        <div>{&x.competitor.horse_name} <span class="comp-no">{&x.competitor.comp_no}</span></div>
                        <div>{x.time_or_rank()}</div>
                    </div>
                </button>
            </li>
        }
    }
}

enum FinalButton {
    ActiveFinalize,
    ActiveView,
    DisabledFinalize,
    DisabledView,
}

impl FinalButton {
    fn render(self) -> Lazy<impl Fn(&mut String)> {
        let res = match self {
            Self::ActiveFinalize | Self::DisabledFinalize => "Finalise competition result",
            Self::ActiveView | Self::DisabledView => "See competition result",
        };

        rsx_move! {
            @match self {
                Self::ActiveFinalize|Self::ActiveView => {
                    <button class="finalize" tx-goto="results">{res}</button>
                },
                _ => {
                    <button class="finalize" disabled>{res}</button>
                }
            }
        }
    }
}
