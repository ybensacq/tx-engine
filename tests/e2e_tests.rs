use assert_cmd::Command;
use rstest::rstest;
use std::fs;
use std::path::Path;

#[rstest]
#[case("input1.csv", "expected_output1.csv")] // Test Case 1: Basic transactions with dispute and chargeback
#[case("input2.csv", "expected_output2.csv")] // Test Case 2: Withdrawal with insufficient funds
#[case("input3.csv", "expected_output3.csv")] // Test Case 3: Dispute on a non-existent transaction
#[case("input4.csv", "expected_output4.csv")] // Test Case 4: Chargeback without prior dispute
#[case("input5.csv", "expected_output5.csv")] // Test Case 5: Multiple clients with interleaved transactions
#[case("input6.csv", "expected_output6.csv")] // Test Case 6: Dispute without resolution or chargeback
#[case("input7.csv", "expected_output7.csv")] // Test Case 7: Multiple disputes without resolution
#[case("input8.csv", "expected_output8.csv")] // Test Case 8: Dispute followed by attempted withdrawal
fn test_end_to_end_processing(#[case] input_file: &str, #[case] expected_output_file: &str) {
    // Build the full paths to the input and expected output files
    let input_path = Path::new("tests/data").join(input_file);
    let expected_output_path = Path::new("tests/data").join(expected_output_file);

    // Read the expected output
    let expected_output = fs::read_to_string(&expected_output_path)
        .expect(&format!("Failed to read {}", expected_output_file));

    // Run binary and capture the output
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
