use super::{dressage_test::DressageTest, ground_jury_member::GroundJuryMember, position::Position, starter::Starter, SurrealId};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct Competition {
	pub id: SurrealId,
	pub name: String,
	pub start_time: chrono::DateTime<chrono::Utc>,
	pub arena: Option<Arena>,
	pub tests: Vec<DressageTest>,
	pub jury: Vec<GroundJuryMember>,
	pub starters: Vec<Starter>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct Arena {
	pub id: SurrealId,
	pub name: String,
}

impl crate::traits::Storable for Competition{}
impl crate::traits::Entity for Competition {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
}

impl Competition {
	pub fn get_position(&self) -> Option<Position> {
		self.jury.first()
			.and_then(|j|Some(&j.position))
			.cloned()
	}
}