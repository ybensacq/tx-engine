use crate::account::Account;
use csv::WriterBuilder;
use std::env;
use std::error::Error;
use std::process;

mod account;
mod engine;
mod error;
mod transaction;
// Module to handle errors

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <transactions.csv>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];

    // Initialize the transaction engine
    let mut engine = engine::Engine::new();

    // Read the CSV file and process each transaction
    let mut rdr = csv::Reader::from_path(input_path)?;
    for result in rdr.deserialize() {
        let transaction: transaction::Transaction = result?;
        // Process each transaction and handle any errors
        if let Err(e) = engine.process_transaction(transaction) {
            eprintln!("Error processing transaction: {e:?}");
        }
    }

    // Write the final state of accounts to standard output
    let mut accounts: Vec<&Account> = engine.accounts.values().collect();
    accounts.sort_by_key(|account| account.client);
    let mut wtr = WriterBuilder::new().from_writer(std::io::stdout());
    wtr.write_record(["client", "available", "held", "total", "locked"])?;
    for account in accounts {
        let (available, held, total, locked) = account.formatted_values();
        wtr.write_record(&[
            account.client.to_string(),
            available,
            held,
            total,
            locked.to_string(),
        ])?;
    }

    Ok(())
}
