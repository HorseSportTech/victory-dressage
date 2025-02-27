use super::{position::Position, SurrealId};


#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct GroundJuryMember {
	pub id: SurrealId,
	pub position: Position,
	pub judge: super::judge::Judge,
	pub authority: JuryAuthority,
}
impl crate::traits::Entity for GroundJuryMember {
	fn key(&self) -> String {format!("{}:{}", self.id.tb, self.id.id())}
	fn get_id(&self) -> String {self.id.id()}
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum JuryAuthority {
    Chief,
    Member,
    Shadow,
    Observer, // not sure if this will be used, possibly for sit-ins
    Removed,  // for when a ground jury member is taken out of calculation
}
impl JuryAuthority {
    pub fn is_actual_jury(&self) -> bool {
        *self == Self::Chief || *self == Self::Member
    }
    pub fn can_score(&self) -> bool {
        *self == Self::Chief || *self == Self::Member || *self == Self::Shadow
    }
}
