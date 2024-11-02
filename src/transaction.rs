use serde::Deserialize;

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
    pub amount: Option<f64>,
    // Flag to indicate if the transaction is under dispute
    #[serde(skip)]
    pub disputed: bool,
}
