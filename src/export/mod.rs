//! Module for exporting data to CSV.

use crate::structs::AccountData;
use rust_decimal::Decimal;
use std::collections::HashMap;

///Function that exports accounts data to stdout.
pub fn export_to_stdout(accounts: &HashMap<u16, AccountData>) {
    //create header
    println!("client,available,held,total,locked");
    //iterate over hashmap and output result to stdout
    //this could probably be faster if using something other than println! or format!
    for (key, val) in accounts {
        println!(
            "{},{},{},{},{}",
            key,
            if val.available == Decimal::ZERO {
                Decimal::ZERO
            } else {
                val.available.round_dp(4)
            },
            if val.held == Decimal::ZERO {
                Decimal::ZERO
            } else {
                val.held.round_dp(4)
            },
            if val.total == Decimal::ZERO {
                Decimal::ZERO
            } else {
                val.total.round_dp(4)
            },
            val.locked
        )
    }
}
