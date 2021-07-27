//! Module for parsing CSV and transaction engine.

use crate::export::export_to_stdout;
use crate::structs::AccountData;
use crate::structs::Transaction;
use crate::structs::TransactionType;
use csv::ReaderBuilder;
use csv::Trim;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::ErrorKind;

///Function for parsing CSV data and handling transactions.
pub fn parse_csv(path: &String) -> Result<(), std::io::Error> {
    //create a mutable hashmap for storing client data
    let mut accounts_map: HashMap<u16, AccountData> = HashMap::new();

    //this is a little messy because we are storing transaction id in two places
    //but i'm really time limited, maybe i will come back to this
    //anyway this provides a quick lookup by transaction id

    let mut transactions_map: HashMap<u32, Transaction> = HashMap::new();
    //create a csv reader builder
    let mut uninit_builder = ReaderBuilder::new();
    //set that csv will always have headers
    uninit_builder.has_headers(true);
    //set that all whitespace be trimmed in headers and data
    uninit_builder.trim(Trim::All);
    //set flexible mode on, maybe fields such as amount are ommited
    uninit_builder.flexible(true);

    //open the file or return error
    let csv_file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Cannot continue, unable to read file '{}'.", path),
            ));
        }
    };

    //create a buffered reader from this file
    let buff = BufReader::new(csv_file);

    //read to csv using the reader
    let mut csv_reader = uninit_builder.from_reader(buff);

    //deserialize all record to Transaction struct
    let csv_records = csv_reader.deserialize::<Transaction>();

    //iterate over records
    for record in csv_records {
        //handle ok and failed deserialization
        match record {
            Ok(data) => {
                //our transaction logic will be here
                //we could write it to another function but since it's simple we should keep it inside this block for performance reasons
                //if client id is not found then create a new client
                if !accounts_map.contains_key(&data.client_id) {
                    accounts_map.insert(data.client_id, AccountData::default());
                }
                //get a mutable reference to internal client data or throw error
                let current_client_data = match accounts_map.get_mut(&data.client_id) {
                    Some(client) => client,
                    None => {
                        return Err(std::io::Error::new(
                            ErrorKind::Other,
                            format!(
                                "Cannot continue, cannot find client id '{}'.",
                                &data.client_id
                            ),
                        ));
                    }
                };

                //guard for locked accounts
                if !current_client_data.locked {
                    //match on transaction type
                    match data.col_type {
                        TransactionType::Deposit => {
                            //deposit is a credit to the client's asset account, meaning it should increase the available
                            //and total funds of the client account

                            //guard for not overwritting transactions with a previously used id
                            //this transactions should not be in our storage for now

                            if let Some(transaction_id) = data.transaction_id {
                                if !transactions_map.contains_key(&transaction_id) {
                                    match data.amount {
                                        Some(amount) => {
                                            current_client_data.total += amount;
                                            current_client_data.available += amount;
                                        }
                                        None => {
                                            //TODO this technicaly is an error from csv, text says we should ignore it
                                        }
                                    }
                                }
                            }
                        }
                        TransactionType::Withdrawal => {
                            // withdraw is a debit to the client's asset account, meaning it should decrease the available and
                            // total funds of the client account

                            //If a client does not have sufficient available funds the withdrawal should fail and the total amount
                            //of funds should not change

                            //guard for not overwritting transactions with a previously used id
                            //this transactions should not be in our storage for now
                            if let Some(transaction_id) = data.transaction_id {
                                if !transactions_map.contains_key(&transaction_id) {
                                    match data.amount {
                                        Some(amount) => {
                                            //guard for withdrawing funds
                                            if (current_client_data.available - amount)
                                                >= Decimal::new(0, 4)
                                            {
                                                current_client_data.available -= amount;
                                                current_client_data.total -= amount;
                                            }
                                        }
                                        None => {
                                            //TODO this technicaly is an error from csv, text says we should ignore it
                                        }
                                    }
                                }
                            }
                        }

                        TransactionType::Dispute => {
                            //guard for getting transaction id
                            match data.transaction_id {
                                Some(transaction_id) => {
                                    //get the underlying transaction for doing this operation
                                    match transactions_map.get_mut(&transaction_id) {
                                        Some(transaction) => {
                                            //guard for disputes referencing a transaction id
                                            //that does not belong to the current client id
                                            //or that is not already in a disputed state

                                            if !transaction.disputed
                                                && data.client_id == transaction.client_id
                                            {
                                                //if by some error the amount was not provided we can asume it's 0 because it will not change anything
                                                let amount = transaction
                                                    .amount
                                                    .unwrap_or(Decimal::new(0, 4));

                                                current_client_data.held += amount;
                                                current_client_data.available -= amount;

                                                //set transaction as disputed
                                                transaction.disputed = true;
                                            }
                                        }
                                        None => {
                                            //TODO this technicaly is an error from csv, text says we should ignore it
                                        }
                                    }
                                }
                                None => {
                                    //TODO this technicaly is an error from csv, text says we should ignore it
                                }
                            }
                        }
                        TransactionType::Resolve => {
                            //guard for getting transaction id
                            match data.transaction_id {
                                Some(transaction_id) => {
                                    //get the underlying transaction for doing this operation
                                    match transactions_map.get_mut(&transaction_id) {
                                        Some(transaction) => {
                                            //check if transaction is under dispute and
                                            //guard for resolves referencing a transaction id
                                            //that does not belong to the current client id

                                            if transaction.disputed
                                                && data.client_id == transaction.client_id
                                            {
                                                //if by some error the amount was not provided we can asume it's 0 because it will not change anything
                                                let amount = transaction
                                                    .amount
                                                    .unwrap_or(Decimal::new(0, 4));
                                                current_client_data.held -= amount;
                                                current_client_data.available += amount;

                                                transaction.disputed = false;
                                            }
                                        }
                                        None => {
                                            //TODO this technicaly is an error from csv, text says we should ignore it
                                        }
                                    }
                                }
                                None => {
                                    //TODO this technicaly is an error from csv, text says we should ignore it
                                }
                            }
                        }
                        TransactionType::Chargeback => {
                            //guard for getting transaction id
                            match data.transaction_id {
                                Some(transaction_id) => {
                                    //get the underlying transaction for doing this operation
                                    match transactions_map.get_mut(&transaction_id) {
                                        Some(transaction) => {
                                            //check if transaction is under dispute and
                                            //guard for resolves referencing a transaction id
                                            //that does not belong to the current client id

                                            if transaction.disputed
                                                && data.client_id == transaction.client_id
                                            {
                                                //if by some error the amount was not provided we can asume it's 0 because it will not change anything
                                                let amount = transaction
                                                    .amount
                                                    .unwrap_or(Decimal::new(0, 4));
                                                //on chargeback take funds from held an total accounts
                                                current_client_data.held -= amount;
                                                current_client_data.total -= amount;
                                                //set disputed to false
                                                transaction.disputed = false;
                                                //lock the client when chargeback occurs
                                                current_client_data.locked = true;
                                            }
                                        }
                                        None => {
                                            //TODO this technicaly is an error from csv, text says we should ignore it
                                        }
                                    }
                                }
                                None => {
                                    //TODO this technicaly is an error from csv, text says we should ignore it
                                }
                            }
                        }
                    }
                }

                //add this transaction to our storage if id was provided
                //this should be the last step to avoid getting erronous results
                //we should only add transactions that are withdrawal or deposit
                //it doesn't make sense that there would be disputes over resolves or resolves over resolves, etc

                match data.transaction_id {
                    Some(id) => {
                        //we should only insert valid transactions if their id does not collide
                        //maybe we get a new transaction with an id that was already used before? this should never happen
                        //but we should also guard for the possibility, if we dont take this into account we end up overwritting
                        //the initial transaction

                        match data.col_type {
                            TransactionType::Deposit | TransactionType::Withdrawal => {
                                if !transactions_map.contains_key(&id) {
                                    transactions_map.insert(id, data.to_owned());
                                }
                            }

                            _ => {}
                        };
                    }
                    None => {
                        //ignore it, no transaction id means no need of storing this
                    }
                }

                //println!("{:#?}", data);
            }
            Err(_) => {
                //TODO if record fails deserialization this should not brake our program
                //println!("{}", e)
            }
        }
    }

    //println!("{:#?}", accounts_map);
    //println!("{:#?}", transactions_map);

    //at this point csv parsing and transactions engine should be finished
    //we need to export our data to valid CSV as stdout
    export_to_stdout(&accounts_map);

    return Ok(());
}
