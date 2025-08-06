use crate::{
    commands::fetch::{fetch, Method},
    debug,
    state::{ManagedApplicationState, StatefulRequestError},
    traits::{Entity, Fetchable},
};

use super::{competition::Competition, SurrealId};

const API_URL: &str = env!("API_URL");

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Show {
    pub id: SurrealId,
    pub name: String,
    pub venue: String,
    #[serde(default)]
    pub competitions: Vec<Competition>,
}

impl crate::traits::Entity for Show {
    fn key(&self) -> String {
        format!("{}:{}", self.id.tb, self.id.id())
    }
    fn get_id(&self) -> String {
        self.id.id()
    }
}
impl Fetchable for Show {
    async fn fetch(
        state: &tauri::State<'_, ManagedApplicationState>,
    ) -> Result<Vec<Self>, StatefulRequestError> {
        let judge_id = {
            state.refresh_if_required().await?;
            state
                .read_async(|st| {
                    st.get_judge_id()
                        .ok_or(StatefulRequestError::NotFound("Judge ID"))
                        .map(|x| x.id())
                })
                .await??
        };

        let shows = fetch(Method::Post, concat!(env!("API_URL"), "show"), &state)
            .body(format!("\"{judge_id}\""))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Self>>()
            .await
            .inspect_err(|err| eprintln!("{err:?}"))?;

        Ok(shows)
    }
    async fn select(
        state: &tauri::State<'_, ManagedApplicationState>,
        id: &str,
    ) -> Result<Self, StatefulRequestError> {
        state.refresh_if_required().await?;

        let show: Show = fetch(Method::Get, &format!("{API_URL}show/{id}"), &state)
            .send()
            .await
            .inspect_err(|err| debug!("Response -> {err:?}"))?
            .error_for_status()
            .inspect_err(|err| debug!("Status -> {err:?}"))?
            .json()
            .await
            .inspect_err(|err| debug!("Decode -> {err:?}"))?;
        Ok(show)
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(transparent)]
pub struct Shows(pub Vec<Show>);
impl Shows {
    pub fn get_show_by_str_id(&self, id: &str) -> Option<&Show> {
        self.0.iter().find(|s| s.get_id() == id)
    }
}
