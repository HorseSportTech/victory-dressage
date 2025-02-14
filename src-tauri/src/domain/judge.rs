use super::{user::User, SurrealId};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct Judge {
	pub id: SurrealId,
	pub first_name: String,
	pub last_name: String,
	pub user: Option<User>,
	pub signature: Option<String>,
	pub prefs: JudgePreferences,
}
impl crate::traits::Entity for Judge {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct JudgePreferences {
	#[serde(default)]
	pub hide_trend: bool,
	#[serde(default)]
	pub comment_last: bool,
	#[serde(default)]
	pub manually_sign: bool,
}