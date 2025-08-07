use super::TxAttributes;
use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::state::ManagedApplicationState;
use crate::templates::icons;
use crate::traits::Entity;
use hypertext::*;

pub async fn result(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let html = state
        .read_async(|app_state| {
            let competition = app_state
                .competition()
                .expect("if we don't have a competition here, something has gone really wrong");
            let judge = app_state.get_judge_id();
            let test = app_state.get_test();
            let mut starters = competition.starters.clone();
            starters.sort_by_key(|x| x.status.abbreviate());
            hypertext::rsx! {
                <main id="page--results" style="position:fixed; inset:0; display:grid; grid: auto 1fr / 1fr;background:white">
                <header>
                    <h1>"Results for "{&competition.name}</h1>
                    <div style="display:flex;flex:row;justify-content:space-between">
                        <button class="back-button" tx-goto="scoresheet" tx-id={competition.get_id()}>{&icons::BACK_ARROW}" Go back"</button>
                        <button class="back-button" tx-command="sign_off_results">"Sign off"</button>
                    </div>
                </header>
                <div style="font-size:var(--text-info); overflow-y: auto">
                @for starter in starters.iter() {
                <details name="testsheet">
                    <summary style="border-bottom:1px solid grey">
                        <div class="main-result" style="display:flex;position:relative;width:100%;padding-inline:1rem;">
                            <div style="width:2.1rem;font-size:1.2rem;font-weight:500">{starter.status.abbreviate()}</div>
                            <div style="width:4.2rem;font-size:1.2rem;font-weight:500">{starter.score.map(|s|s.round(3))}</div>
                            <div style="width:30vw">
                                <div>{starter.name()}</div>
                                <div>{starter.horse()}</div>
                            </div>
                            <div
                                style="position:absolute;height:40%;width:calc(70vw - 5.8rem);background:var(--theme);inset:auto 0 0 auto;clip-path:polygon(0.7rem 0, 100% 0, 100% 100%, 0 100%);z-index:-1"></div>
                            @for scoresheet in starter.scoresheets.iter() {
                                <div style="flex: 1 0 auto;align-self:start; margin-top:.3rem; text-align:center;">
                                    <div>{scoresheet.score.map(|s|s.round(3))}</div>
                                    <div style="color:var(--foreground); padding-top:.2rem">{scoresheet.rank.map(|r|r.to_string())}</div>
                                </div>
                            }
                        </div>
                        <div style="flex: 0 1 auto;font-family:writing">{starter.scoresheets.iter().map(|s|s.notes.as_ref()).collect::<Vec<_>>()}
                        </div>
                    </summary>
                    <div style="display:grid;" class="test-sheet">
                        @if let Some(test) = test {
                            @for exercise in test.movements.iter() {
                                <div
                                    class="exercise-abbreviation"
                                    data-index=exercise.number+1
                                    style=format!("grid-row: {nr}; grid-column:1; min-width: 9rem", nr = exercise.number+1)
                                >{exercise.number}. {&exercise.abbreviation}</div>
                            }
                            @let number_of_scoresheets = starter.scoresheets.len();
                            @for (scoresheet, i) in starter.scoresheets.iter().zip(0..number_of_scoresheets) {
                                <div style=format!("grid-row: 1; grid-column: {i}", i = i+2)>{scoresheet.score}</div>
                                @let mut comments:Vec<String> = Vec::with_capacity(number_of_scoresheets);
                                @for movement in scoresheet.scores.iter() {
                                    <div style=format!("grid-row: {nr}; grid-column: {i}", i = i+2, nr = movement.number+1)>{movement.mark}</div>
                                }
                            }
                        }
                    </div>
                </details>
                }
                </div>
                </main>
            }
            .render()
        })
        .await?;

    Ok(ReplaceDirector::page(html))
}
