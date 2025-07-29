use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::state::ManagedApplicationState;
use hypertext::*;

pub async fn result(state: tauri::State<'_, ManagedApplicationState>) -> ResponseDirector {
    let html = state
        .read_async(|app_state| {
            let competition = app_state
                .competition
                .as_ref()
                .expect("if we don't have a competition here, something has gone really wrong");
            let test = app_state.get_test();
            let mut starters = competition.starters.clone();
            starters.sort_by_key(|x| x.status.abbreviate());
            hypertext::rsx! {
                <main id="page--results" style="position:fixed; inset:0; display:grid; grid: auto 1fr / 1fr;background:white">
                <header>
                    <h1>"Results for "{&competition.name}</h1>
                </header>
                <div style="font-size:var(--text-info); overflow-y: auto">
                @for starter in starters.iter() {
                    <details name="testsheet">
                    <summary style="border-bottom:1px solid grey">
                        <div style="display:flex;">
                            <div>{starter.status.abbreviate()}</div>
                            <div>{starter.name()}<br>{starter.horse()}</div>
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
                            @for (scoresheet, i) in starter.scoresheets.iter().zip(0..starter.scoresheets.len()) {
                                <div style=format!("grid-row: 1; grid-column: {i}", i = i+2)>{scoresheet.score}</div>
                                @for movement in scoresheet.scores.iter() {
                                    <div style=format!("grid-row: {nr}; grid-column: {i}", i = i+2, nr = movement.nr+1)>{movement.mk}</div>
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
