use decimal::{dec, Decimal};

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    #[serde(rename = "nr")]
    pub number: u8,

    #[serde(
        rename = "co",
        default = "coefficient_default",
        skip_serializing_if = "is_coefficient_default",
        serialize_with = "decimal::parsing::serialize_as_f32",
        deserialize_with = "decimal::parsing::deserialize_from_f32"
    )]
    pub coefficient: Decimal,

    #[serde(
        rename = "mx",
        default = "max_default",
        skip_serializing_if = "is_max_default",
        serialize_with = "decimal::parsing::serialize_as_f32",
        deserialize_with = "decimal::parsing::deserialize_from_f32"
    )]
    pub max: Decimal,

    #[serde(
        rename = "mn",
        default = "min_default",
        skip_serializing_if = "is_min_default",
        serialize_with = "decimal::parsing::serialize_as_f32",
        deserialize_with = "decimal::parsing::deserialize_from_f32"
    )]
    pub min: Decimal,

    #[serde(
        rename = "st",
        default = "step_default",
        skip_serializing_if = "is_step_default",
        serialize_with = "decimal::parsing::serialize_as_f32",
        deserialize_with = "decimal::parsing::deserialize_from_f32"
    )]
    pub step: Decimal,

    #[serde(
        rename = "ct",
        default,
        skip_serializing_if = "super::MovementCategory::is_technical"
    )]
    pub category: super::MovementCategory,

    #[serde(rename = "ln", default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<MovementLine>,

    #[serde(rename = "ab", default, skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    #[serde(rename = "di", default, skip_serializing_if = "Option::is_none")]
    pub directive_ideas: Option<Vec<String>>,
    // #[serde(rename = "df", default, skip_serializing_if = "Option::is_none")]
    // pub difficulty: Option<f32>,

    // #[serde(rename = "rp", default, skip_serializing_if = "std::ops::Not::not")]
    // pub repeat: bool,

    // #[serde(rename = "cm", default, skip_serializing_if = "Option::is_none")]
    // pub combination: Option<CombinationCategory>,

    // #[serde(rename = "do", default, skip_serializing_if = "Option::is_none")]
    // pub option: Option<DifficultyOption>,

    // #[serde(rename = "mi", default, skip_serializing_if = "Option::is_none")]
    // pub mapping_index: Option<u8>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MovementLine {
    #[serde(rename = "i")]
    pub index: u8,
    #[serde(rename = "l")]
    pub letter: String,
    #[serde(rename = "d")]
    pub description: String,
}

// Helper functions for Movement serialization
fn is_max_default(num: &Decimal) -> bool {
    *num == max_default()
}
fn is_min_default(num: &Decimal) -> bool {
    *num == min_default()
}
fn is_step_default(num: &Decimal) -> bool {
    *num == step_default()
}
fn is_coefficient_default(num: &Decimal) -> bool {
    *num == coefficient_default()
}

const fn max_default() -> Decimal {
    dec!(10.0)
}
const fn min_default() -> Decimal {
    dec!(0.0)
}
const fn step_default() -> Decimal {
    dec!(0.5)
}
pub const fn coefficient_default() -> Decimal {
    dec!(1.0)
}
// end helpers

