use std::{
    fmt,
    num::ParseFloatError,
    str::FromStr,
};

/// Represents an amount of money in Euro (€) by storing the number of cents
/// as positive integer.
#[derive(Serialize, Deserialize)]
pub struct Euro(u64);

impl Euro {
    pub fn from_cents(cents: u64) -> Self {
        Euro(cents)
    }
}

impl FromStr for Euro {
    type Err = ParseFloatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>().map(|v| Euro((v * 100.0) as u64))
    }
}

impl fmt::Display for Euro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:01}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl fmt::Debug for Euro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}
