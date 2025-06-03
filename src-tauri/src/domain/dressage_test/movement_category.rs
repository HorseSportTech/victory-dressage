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
    pub fn has_letters(&self) -> bool {
        Self::Technical == *self || Self::Particle == *self
    }
    pub fn has_attempts(&self) -> bool {
        Self::Technical == *self
    }
}

impl std::fmt::Display for MovementCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Technical => "Technical",
                Self::Artistic => "Artistic",
                Self::Collective => "Collective",
                Self::Acceptable => "Acceptable",
                Self::Joker => "Joker",
                Self::Particle => "Particle",
            }
        )
    }
}

