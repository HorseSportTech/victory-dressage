use super::{position::Position, SurrealId};


#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct GroundJuryMember {
	pub id: SurrealId,
	pub position: Position,
	pub judge: super::judge::Judge,
}
impl crate::traits::Entity for GroundJuryMember {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn id(&self) -> String {self.id.id()}
}

