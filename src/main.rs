use crate::account::Account;
use chrono::Local;
use csv::WriterBuilder;
use std::env;
use std::error::Error;
use std::process;

mod account;
mod engine;
mod error;
mod transaction;

fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Local::now();
    eprintln!(
        "Program started at {}",
        start_time.format("%Y-%m-%d %H:%M:%S")
    );

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <transactions.csv>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];

    let mut engine = engine::Engine::new();
    let mut successful_count = 0;
    let mut error_count = 0;

    let mut rdr = csv::Reader::from_path(input_path)?;
    for result in rdr.deserialize() {
        // Process each transaction and handle any errors
        match result {
            Ok(transaction) => {
                if let Err(e) = engine.process_transaction(transaction) {
                    // Error processing transaction: this will be logged to a file in future iterations.
                    eprintln!("An error has occured on transaction processing : {e}");
                    error_count += 1;
                } else {
                    successful_count += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to parse transaction record at line {}: {}.",
                    e.position().map_or("unknown".to_string(), |pos| pos.line().to_string()),
                    e
                );
            }
        }
    }

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

    let end_time = Local::now();
    eprintln!(
        "Processing completed at {} in {} ms. Successful transactions: {}. Errors encountered: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        (end_time - start_time).num_milliseconds(),
        successful_count,
        error_count
    );

    Ok(())
}
