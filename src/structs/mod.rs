//! Module for storing structs and enums that are used in other modules.

use rust_decimal::Decimal;
use serde::Deserialize;

///Transaction type enum. Instruct serde how to deserialize by renaming to lowercase.
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}
///Transaction structure. Provides serde crate with field names in CSV.
#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Transaction {
    ///Type for transactions.
    #[serde(rename = "type")]
    pub col_type: TransactionType,
    ///Client id inside transaction.
    #[serde(rename = "client")]
    pub client_id: u16,
    ///Transaction id data field wrapped in option type.
    #[serde(rename = "tx")]
    pub transaction_id: Option<u32>,
    ///Amount data field wrapped in option type.
    #[serde(rename = "amount")]
    pub amount: Option<Decimal>,
    #[serde(default)]
    ///Disputed flag set to "false" on init for every new transaction.
    pub disputed: bool,
}

///Account data structure for storing account details.
#[derive(Debug)]
pub struct AccountData {
    ///Available funds associated with a client id.
    pub available: Decimal,
    ///Help funds associated with a client id.
    pub held: Decimal,
    ///Total funds associated with a client id.
    pub total: Decimal,
    ///Locked state for a client id.
    pub locked: bool,
}

///Implement default for AccountData. This get's stored when parsing a new client id.
impl Default for AccountData {
    fn default() -> Self {
        return Self {
            available: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
            total: Decimal::new(0, 4),
            locked: false,
        };
    }
}
