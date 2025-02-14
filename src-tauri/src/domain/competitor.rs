use super::SurrealId;
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct Competitor {
	pub id: SurrealId,
	pub first_name: String,
	pub last_name: String,
	pub horse_name: String,
	pub comp_no: String,
}

impl crate::traits::Entity for Competitor {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
}