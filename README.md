# Transaction Processor - README

## Introduction

This project implements a simple transaction engine in Rust, designed to process a series of financial transactions, including deposits, withdrawals, disputes, resolutions, and chargebacks. It reads transactions from a CSV file, updates client accounts accordingly, handles disputes and chargebacks, and then outputs the final state of client accounts as a CSV.

This implementation strictly adheres to the specifications provided, focusing on code **correctness**, **safety**, **efficiency**, and **maintainability**.

## Features

- Processes the following transaction types:
    - **Deposit**
    - **Withdrawal**
    - **Dispute**
    - **Resolve**
    - **Chargeback**
- Handles multiple clients, each with a single asset account.
- Manages account states, including available and held funds, total balance, and account locking.
- Supports transaction amounts with up to four decimal places of precision.
- Efficiently handles large datasets by streaming transactions and minimizing memory usage.
- Provides comprehensive unit and integration tests to ensure correctness and robustness.

## Getting Started

### Prerequisites

- **Rust** programming language (version 1.54 or higher recommended)
- **Cargo** package manager

### Code Quality Tools

To ensure code quality and maintain a consistent code style, this project uses `rustfmt` and `clippy` with pedantic settings.

#### Formatting with `rustfmt`

`rustfmt` is used to automatically format your Rust code according to the official Rust style guidelines.

To format the code, run the following command in your terminal:

```sh
cargo fmt
```

This will format all the Rust files in your project.

Linting with clippy
clippy provides additional lints to catch common mistakes and improve your Rust code. We use clippy with the pedantic setting to enforce stricter checks.

To run clippy in pedantic mode and check for linting issues, use the following command:

```sh
cargo clippy -- -W clippy::pedantic
```

This will analyze your code and print warnings and suggestions.


### Building the Project

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/ybensacq/tx-engine.git
cd tx-engine
cargo build --release
```

### Running the Application

The application reads transactions from a CSV file and outputs the final account states to `stdout`. You can run the application as follows:

```bash
cargo run --release -- transactions.csv > accounts.csv
```

- **transactions.csv**: Input CSV file containing the list of transactions.
- **accounts.csv**: Output CSV file with the final state of client accounts.

### Input Format

The input CSV file should have the following columns:

- **type**: Transaction type (`deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`)
- **client**: Client ID (unsigned 16-bit integer)
- **tx**: Transaction ID (unsigned 32-bit integer)
- **amount**: Transaction amount (decimal with up to four decimal places; absent for `dispute`, `resolve`, `chargeback`)

Example:

```csv
type,client,tx,amount
deposit,1,1,1000.0
withdrawal,1,2,500.0
dispute,1,1,
chargeback,1,1,
```

### Output Format

The output is a CSV with the following columns:

- **client**: Client ID
- **available**: Available funds
- **held**: Held funds
- **total**: Total funds (available + held)
- **locked**: Account locked status (`true` or `false`)

Example:

```csv
client,available,held,total,locked
1,-500.0000,0.0000,-500.0000,true
2,1000.0000,0.0000,1000.0000,false
```

## Assumptions

- Dispute on Deposits Only: We assumed that only deposit transactions can be disputed. This choice was made to align with typical transaction processing practices where only credits to an account (deposits) are disputable, as withdrawals or other types would not usually be eligible for reversal.
- Each client has a single asset account.
- Once a client account is frozen (e.g., after a chargeback), any subsequent dispute or resolve events for that account are ignored and not processed.
- Transactions occur chronologically in the input file.
- Transaction amounts have a precision of up to four decimal places.
- Transactions reference existing clients or create new ones if they don't exist.
- Disputes, resolves, and chargebacks reference valid transactions; invalid references are ignored.

## Testing

### Unit Tests

Unit tests are located in the `src` directory alongside the implementation code. They cover individual components and functions to ensure correctness.

Run unit tests using:

```bash
cargo test
```

### Integration Tests

Integration tests, including end-to-end tests, are located in the `tests` directory. They simulate real-world scenarios by processing sample CSV files and comparing the output to expected results.

Run integration tests using:

```bash
cargo test --test e2e_tests
```

### Test Coverage

The tests cover various scenarios, including:

- Normal transaction processing (deposits and withdrawals)
- Disputes and resolutions
- Chargebacks and account locking
- Handling of insufficient funds
- Edge cases such as disputes on non-existent transactions

## Error Handling

- The application validates input data and gracefully handles invalid entries by logging warnings and continuing processing.
- Errors such as invalid transaction types, missing fields, or invalid amounts are reported but do not halt execution.
- Accounts are only locked upon a successful chargeback.

## Performance Considerations

- Transactions are streamed and processed line by line to minimize memory usage, allowing the application to handle large datasets efficiently.
- Data structures are optimized for quick access and updates, using `HashMap` for account storage.

## Dependencies

- **serde**: For serialization and deserialization of CSV data.
- **csv**: For reading and writing CSV files.
- **log**: For logging warnings and errors.
- **assert_cmd**, **predicates**: For integration testing.

## Code Quality and Maintainability

- The code adheres to Rust's best practices, following idiomatic patterns and proper error handling.
- Functions and modules are well-documented, with clear comments explaining complex logic.
- The project is organized into modules for clarity and separation of concerns.

## Future Improvements

- Replace primitive decimals by BigDecimal to properly handle large values.
- Implement concurrency to process multiple input files or streams simultaneously.
- Logging to File: Instead of printing error messages to standard error, implement structured logging to a file. This will allow for better error tracking, facilitate debugging, and keep the program's output clean when run in production environments.
- Enhance logging with different verbosity levels.
- Add support for additional transaction types or multi-asset accounts.
---

*This project is developed for educational purposes and is not intended for production use.*