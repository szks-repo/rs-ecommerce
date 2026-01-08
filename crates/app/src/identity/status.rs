use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreStaffStatus {
    Active,
    Invited,
}

impl StoreStaffStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Invited => "invited",
        }
    }
}

impl fmt::Display for StoreStaffStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for StoreStaffStatus {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let normalized = value.trim();
        match normalized {
            "active" => Ok(Self::Active),
            "invited" => Ok(Self::Invited),
            _ => Err("status must be active or invited"),
        }
    }
}
