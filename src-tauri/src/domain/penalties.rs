#[derive(Debug, Default, PartialEq, Clone)]
pub struct Penalties(pub Vec<Penalty>);
impl std::ops::Deref for Penalties {
    type Target = Vec<Penalty>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Penalties {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub struct Penalty {
    pub idx: u8,
    pub ty: PenaltyType,
}
impl serde::Serialize for Penalties {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut penalties = self.0.clone();
        penalties.sort_by(|a, b| a.idx.cmp(&b.idx));
        let res = penalties
            .iter()
            .map(|pen| match pen.ty {
                PenaltyType::Points(p) => format!("{}p", p),
                PenaltyType::Percentage(p) => format!("{}%", p),
                PenaltyType::Elimination => "E".to_string(),
            })
            .collect::<Vec<String>>()
            .join(";");
        serializer.serialize_str(&res)
    }
}
impl<'de> serde::Deserialize<'de> for Penalties {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s == "" {
            return Ok(Penalties(vec![]));
        }
        let penalty_strings = s.split(";");
        let mut penalties = Vec::new();
        for (idx, pen) in penalty_strings.enumerate() {
            let mut pen = pen.to_string();
            let ty = match pen.pop().ok_or(serde::de::Error::custom(
                "empty string which should not be empty",
            ))? {
                'p' => {
                    let p = pen
                        .parse::<f32>()
                        .ok()
                        .ok_or(serde::de::Error::custom("Invalid f32 for penalty points"))?;
                    PenaltyType::Points(p)
                }
                '%' => {
                    let p = pen.parse::<f32>().ok().ok_or(serde::de::Error::custom(
                        "Invalid f32 for penalty percentage",
                    ))?;
                    PenaltyType::Percentage(p)
                }
                'E' => PenaltyType::Elimination,
                _ => return Err(serde::de::Error::custom("Invalid penalty type")),
            };
            if idx > 255 {
                panic!("Too many penalties")
            };
            penalties.push(Penalty { idx: idx as u8, ty });
        }
        Ok(Penalties(penalties))
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum PenaltyType {
    Points(f32),
    Percentage(f32),
    Elimination,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum BroadcastPenaltyVariety {
    #[serde(alias = "errors")]
    ErrorsOfCourse,
    #[serde(alias = "artistic")]
    ArtisticPenalty,
    #[serde(alias = "technical")]
    TechnicalPenalty,
}
