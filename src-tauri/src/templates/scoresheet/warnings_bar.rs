use super::super::{html_elements, TxAttributes};
use crate::templates::icons;
use crate::{
    domain::{dressage_test::DressageTest, scoresheet::Scoresheet, starter::Starter},
    templates::scoresheet::{artistic_row, errors_row, status_selection, technical_row},
};
use hypertext::{rsx_move, GlobalAttributes, Lazy};

pub fn warnings_bar<'b>(
    test: &'b DressageTest,
    starter: &'b Starter,
    scoresheet: &'b Scoresheet,
) -> Lazy<impl Fn(&mut String) + 'b> {
    let status_selection_html = status_selection(starter.status.clone());
    rsx_move! {
        <aside>
            <button
                class="scoresheet-menu-button left"
                tx-open="#warnings-menu"
            >{icons::WARNING}</button>
            <dialog
                id="warnings-menu"
                onclick="event.target==this && this.close()"
            >
                <div style="background:var(--foreground); padding:0.5rem;">
                    <fieldset>
                        <legend>"Judges’ Signalling System"</legend>
                        <div
                            class="dialog-header warning-button"
                            style="display:grid; inline-size:100%; aspect-ratio: 5/3; grid: 1fr 1fr/1fr 1fr; gap:0.1rem"
                        >
                            <button id="button-blood" tx-command="toggle_blood">"Blood"</button>
                            <button id="button-lameness" tx-command="toggle_lameness">"Lameness"</button>
                            <button id="button-equipment" tx-command="toggle_equipment">"Tack"</button>
                            <button id="button-meeting" tx-command="toggle_meeting">"Meeting"</button>
                        </div>
                    </fieldset>

                    <fieldset>
                        <legend>"Status"</legend>
                        <div>
                            <label for="competitor-status">"Competitor Status"</label>
                            <div class="selector-down-arrow" id="status-selector">
                                {&status_selection_html}
                            </div>
                        </div>
                    </fieldset>

                    <fieldset>
                        <legend>"Penalties"</legend>
                        <div>
                            <div class="penalty-row" id="penalties-errors">
                                {errors_row(test.errors_of_course.len() > 0, scoresheet.errors)}
                            </div>
                            <div class="penalty-row" id="penalties-technical">
                                {technical_row(test.technical_penalties.len() > 0, scoresheet.tech_penalties)}
                            </div>
                            <div class="penalty-row" id="penalties-artistic">
                                {artistic_row(test.artistic_penalties.len() > 0, scoresheet.art_penalties)}
                            </div>
                        </div>
                    </fieldset>
                </div>
            </dialog>
        </aside>
    }
}

