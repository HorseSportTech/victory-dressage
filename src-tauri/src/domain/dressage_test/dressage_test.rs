use crate::domain::{penalties::Penalties, SurrealId};

use super::Exercise;


#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct DressageTest {
	pub id: SurrealId,
	pub name: String,
	pub movements: Vec<Exercise>,
	pub errors_of_course: Penalties,
	pub technical_penalties: Penalties,
	pub artistic_penalties: Penalties,
}
impl crate::traits::Entity for DressageTest {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
}



#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct Penalty {
	pub index: u8,
	pub r#type: PenaltyType,
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub enum PenaltyType {
	Elimination,
	Points(f32),
	Percentage(f32),
}



