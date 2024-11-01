use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Account {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Account {
    pub fn formatted_values(&self) -> (String, String, String, bool) {
        (
            Self::format_value(self.available),
            Self::format_value(self.held),
            Self::format_value(self.total),
            self.locked,
        )
    }

    // Truncate to four decimal places by scaling and converting to integer
    fn format_value(value: f64) -> String {
        // Truncate to four decimal places
        let truncated = (value * 10_000.0).trunc() / 10_000.0;

        // Conditional formatting based on fractional part
        if (truncated * 10.0).fract() == 0.0 {
            format!("{truncated:.1}")
        } else if (truncated * 100.0).fract() == 0.0 {
            format!("{truncated:.2}")
        } else if (truncated * 1000.0).fract() == 0.0 {
            format!("{truncated:.3}")
        } else {
            format!("{truncated:.4}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import all from the current module

    #[test]
    fn test_format_value_truncation() {
        assert_eq!(Account::format_value(1000.9999999), "1000.9999");
        assert_eq!(Account::format_value(1000.12345), "1000.1234");
        assert_eq!(Account::format_value(1000.1), "1000.1");
        assert_eq!(Account::format_value(1000.12), "1000.12");
        assert_eq!(Account::format_value(500.0), "500.0");
        assert_eq!(Account::format_value(-123.456789), "-123.4567");
    }
}
