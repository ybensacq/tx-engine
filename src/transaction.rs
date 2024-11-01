use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub t_type: Type,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "deserialize_amount")]
    pub amount: Option<f64>,
    // Flag to indicate if the transaction is under dispute
    #[serde(skip)]
    pub disputed: bool,
}

// Custom deserializer for the amount to handle optional values
fn deserialize_amount<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct AmountVisitor;

    impl<'de> Visitor<'de> for AmountVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an optional decimal number")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            let amount = f64::deserialize(deserializer)?;
            Ok(Some(amount))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_option(AmountVisitor)
}
