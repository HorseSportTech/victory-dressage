#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub enum MovementCategory {
    #[default]
    #[serde(rename = "T")]
    Technical,
    #[serde(rename = "A")]
    Artistic,
    #[serde(rename = "C")]
    Collective,
    #[serde(rename = "J")]
    Joker, // TODO: maybe should contain mapping index??
    #[serde(rename = "P")]
    Particle, // never counted regardless of coefficient
    #[serde(rename = "E")]
    Acceptable, // combinations and transitions
}
impl MovementCategory {
    pub fn is_technical(&self) -> bool {
        Self::Technical == *self
    }
}