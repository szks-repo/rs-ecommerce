use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageStatus {
    Draft,
    Published,
}

impl PageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }
}

impl fmt::Display for PageStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for PageStatus {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Ok(Self::Draft);
        }
        match normalized {
            "draft" => Ok(Self::Draft),
            "published" => Ok(Self::Published),
            _ => Err("status must be draft or published"),
        }
    }
}
