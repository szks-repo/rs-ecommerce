use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionType {
    Open,
    Sealed,
}

impl AuctionType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Sealed => "sealed",
        }
    }
}

impl fmt::Display for AuctionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AuctionType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "open" => Ok(Self::Open),
            "sealed" => Ok(Self::Sealed),
            _ => Err("auction_type is invalid"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionStatus {
    Draft,
    Scheduled,
    Running,
    Ended,
    AwaitingApproval,
    Approved,
}

impl AuctionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Scheduled => "scheduled",
            Self::Running => "running",
            Self::Ended => "ended",
            Self::AwaitingApproval => "awaiting_approval",
            Self::Approved => "approved",
        }
    }
}

impl fmt::Display for AuctionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AuctionStatus {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "draft" => Ok(Self::Draft),
            "scheduled" => Ok(Self::Scheduled),
            "running" => Ok(Self::Running),
            "ended" => Ok(Self::Ended),
            "awaiting_approval" => Ok(Self::AwaitingApproval),
            "approved" => Ok(Self::Approved),
            _ => Err("status is invalid"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoBidStatus {
    Active,
    Disabled,
}

impl AutoBidStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

impl fmt::Display for AutoBidStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AutoBidStatus {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            _ => Err("auto_bid status is invalid"),
        }
    }
}
