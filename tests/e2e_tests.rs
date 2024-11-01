use assert_cmd::Command;
use std::fs;
use std::path::Path;

#[test]
fn test_end_to_end_processing() {
    // List of test cases: (input file, expected output file)
    let test_cases = vec![
        // Test Case 1: Basic transactions with dispute and chargeback
        // - Client 1: Deposit, withdrawal, dispute, and chargeback leading to account lock
        // - Client 2: Deposit, withdrawal, dispute, and resolution
        ("input1.csv", "expected_output1.csv"),
        // Test Case 2: Withdrawal with insufficient funds
        // - Withdrawal should fail, account balance remains unchanged
        ("input2.csv", "expected_output2.csv"),
        // Test Case 3: Dispute on a non-existent transaction
        // - Dispute should be ignored or cause an error, account remains unaffected
        ("input3.csv", "expected_output3.csv"),
        // Test Case 4: Chargeback without prior dispute
        // - Chargeback should fail, account remains unaffected
        ("input4.csv", "expected_output4.csv"),
        // Test Case 5: Multiple clients with interleaved transactions
        // - Client 1: Dispute and resolve
        // - Client 2: Dispute and chargeback leading to account lock
        ("input5.csv", "expected_output5.csv"),
        // Test Case 6: Dispute without resolution or chargeback
        // - Account has held funds, available balance is negative
        ("input6.csv", "expected_output6.csv"),
        // Test Case 7: Multiple disputes without resolution
        // - Account has multiple held transactions
        ("input7.csv", "expected_output7.csv"),
        // Test Case 8: Dispute followed by attempted withdrawal
        // - Withdrawal should fail due to insufficient available funds
        ("input8.csv", "expected_output8.csv"),
    ];

    for (input_file, expected_output_file) in test_cases {
        // Build the full paths to the input and expected output files
        let input_path = Path::new("tests/data").join(input_file);
        let expected_output_path = Path::new("tests/data").join(expected_output_file);

        // Read the expected output
        let expected_output = fs::read_to_string(&expected_output_path)
            .expect(&format!("Failed to read {}", expected_output_file));

        // Run your binary and capture the output
        let output = Command::cargo_bin("process-tx")
            .expect("Binary not found")
            .arg(input_path)
            .output()
            .expect("Failed to execute command");

        // Convert the output to a string
        let actual_output = String::from_utf8(output.stdout).expect("Output not valid UTF-8");

        // Normalize line endings and trim whitespace
        let expected_output = expected_output.replace("\r\n", "\n").trim_end().to_string();
        let actual_output = actual_output.replace("\r\n", "\n").trim_end().to_string();

        // Compare the actual output with the expected output
        assert_eq!(
            actual_output, expected_output,
            "Test failed for input file: {}",
            input_file
        );
    }
}
