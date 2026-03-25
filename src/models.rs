#[derive(Clone, Default)]
pub struct Profile {
    pub name: Option<String>,
    pub gender: Option<Gender>,
    pub looking_for: Option<Gender>,
    pub age: Option<u8>,
    pub location: Option<String>,
    pub description: Option<String>,
    pub photo: Option<String>,
}

impl Profile {
    pub fn display_text(&self) -> String {
        format!(
            "{}, {}, {} - {}",
            self.name.as_deref().unwrap_or("neuvedené"),
            self.age
                .map(|a| a.to_string())
                .unwrap_or_else(|| "neuvedené".to_string()),
            self.location.as_deref().unwrap_or("neuvedené"),
            self.description.as_deref().unwrap_or(""),
        )
    }
}

#[derive(Clone)]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    pub fn from_text(text: &str) -> Option<Self> {
        match text.trim() {
            "Muž" => Some(Self::Male),
            "Žena" => Some(Self::Female),
            _ => None,
        }
    }

    pub fn from_db_code(code: &str) -> Option<Self> {
        match code {
            "M" => Some(Self::Male),
            "F" => Some(Self::Female),
            _ => None,
        }
    }

    pub fn as_db_code(&self) -> &'static str {
        match self {
            Self::Male => "M",
            Self::Female => "F",
        }
    }
}