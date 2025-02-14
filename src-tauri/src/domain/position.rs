#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub enum Position {
	K,
	E,
	H,
	#[default]
	C,
	M,
	B,
	F
}

impl std::fmt::Display for Position {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::K => "K",
			Self::E => "E",
			Self::H => "H",
			Self::C => "C",
			Self::M => "M",
			Self::B => "B",
			Self::F => "F"
		})
	}
}