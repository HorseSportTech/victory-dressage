use crate::{
    commands::fetch::{fetch, Method},
    debug,
    state::{ApplicationState, ManagedApplicationState, StatefulRequestError},
    traits::{Entity, Fetchable, Storable},
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

impl crate::traits::Storable for Show {}
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
        state: tauri::State<'_, ManagedApplicationState>,
    ) -> Result<Vec<Self>, StatefulRequestError> {
        let judge_id = {
            state.refresh_if_required().await?;
            let s = state.read_async(ApplicationState::get_user_id).await?;
            s.map(|x| x.id())
                .ok_or(StatefulRequestError::NotFound("Judge ID"))?
        };

        let shows = fetch(Method::Post, concat!(env!("API_URL"), "show"), &state)
            .body(format!("\"{judge_id}\""))
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Self>>()
            .await
            .inspect_err(|err| eprintln!("{err:?}"))?;
        debug!("{shows:?}");

        Ok(shows)
    }
    async fn select(
        state: tauri::State<'_, ManagedApplicationState>,
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
impl Storable for Shows {}
impl Entity for Shows {
    fn key(&self) -> String {
        String::from("shows")
    }
    fn get_id(&self) -> String {
        String::from("shows")
    }
}
