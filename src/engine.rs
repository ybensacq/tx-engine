use std::collections::HashMap;

use crate::account::Account;
use crate::error::Transaction as TransactionError;
use crate::transaction::{Transaction, Type as TransactionType};

pub struct Engine {
    pub accounts: HashMap<u16, Account>,
    pub transactions: HashMap<u32, Transaction>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn process_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), TransactionError> {
        let client_id = transaction.client;
        let account = self.accounts.entry(client_id).or_insert_with(|| Account {
            client: client_id,
            ..Default::default()
        });

        if account.locked {
            return Err(TransactionError::AccountLocked(client_id));
        }

        match transaction.t_type {
            TransactionType::Deposit => self.process_deposit(transaction),
            TransactionType::Withdrawal => self.process_withdrawal(transaction),
            TransactionType::Dispute => self.process_dispute(&transaction),
            TransactionType::Resolve => self.process_resolve(&transaction),
            TransactionType::Chargeback => self.process_chargeback(&transaction),
        }
    }

    fn process_deposit(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        if let Some(account) = self.accounts.get_mut(&transaction.client) {
            if let Some(amount) = transaction.amount {
                account.available += amount;
                account.total += amount;
                self.transactions.insert(transaction.tx, transaction);
                Ok(())
            } else {
                Err(TransactionError::InvalidAmount(transaction.tx))
            }
        } else {
            eprintln!("Account not found for client ID: {}", transaction.client);
            Ok(())
        }
    }

    fn process_withdrawal(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        if let Some(account) = self.accounts.get_mut(&transaction.client) {
            if let Some(amount) = transaction.amount {
                if account.available >= amount {
                    account.available -= amount;
                    account.total -= amount;
                    self.transactions.insert(transaction.tx, transaction);
                    Ok(())
                } else {
                    Err(TransactionError::InsufficientFunds(account.client))
                }
            } else {
                Err(TransactionError::InvalidAmount(transaction.tx))
            }
        } else {
            Err(TransactionError::AccountNotFound(transaction.client))
        }
    }

    fn process_dispute(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        if let Some(account) = self.accounts.get_mut(&transaction.client) {
            if let Some(original_tx) = self.transactions.get_mut(&transaction.tx) {
                if !original_tx.disputed && original_tx.client == account.client {
                    if let Some(amount) = original_tx.amount {
                        if let TransactionType::Deposit = original_tx.t_type {
                            account.available -= amount;
                            account.held += amount;
                            original_tx.disputed = true;
                            Ok(())
                        } else {
                            Err(TransactionError::InvalidDispute(transaction.tx))
                        }
                    } else {
                        Err(TransactionError::InvalidAmount(transaction.tx))
                    }
                } else {
                    Err(TransactionError::AlreadyDisputed(transaction.tx))
                }
            } else {
                Err(TransactionError::NotFound(transaction.tx, account.client))
            }
        } else {
            Err(TransactionError::AccountNotFound(transaction.client))
        }
    }

    fn process_resolve(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        if let Some(account) = self.accounts.get_mut(&transaction.client) {
            if let Some(original_tx) = self.transactions.get_mut(&transaction.tx) {
                if original_tx.disputed && original_tx.client == account.client {
                    if let Some(amount) = original_tx.amount {
                        account.available += amount;
                        account.held -= amount;
                        original_tx.disputed = false;
                        Ok(())
                    } else {
                        Err(TransactionError::InvalidAmount(transaction.tx))
                    }
                } else {
                    Err(TransactionError::NotUnderDispute(transaction.tx))
                }
            } else {
                Err(TransactionError::NotFound(transaction.tx, account.client))
            }
        } else {
            Err(TransactionError::AccountNotFound(transaction.client))
        }
    }

    fn process_chargeback(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        if let Some(account) = self.accounts.get_mut(&transaction.client) {
            if let Some(original_tx) = self.transactions.get_mut(&transaction.tx) {
                if original_tx.disputed && original_tx.client == transaction.client {
                    if let TransactionType::Deposit = original_tx.t_type {
                        if let Some(amount) = original_tx.amount {
                            account.held -= amount;
                            account.total -= amount;

                            original_tx.disputed = false;
                            account.locked = true;

                            Ok(())
                        } else {
                            Err(TransactionError::InvalidAmount(transaction.tx))
                        }
                    } else {
                        Err(TransactionError::InvalidChargeback(transaction.tx))
                    }
                } else {
                    Err(TransactionError::NotUnderDispute(transaction.tx))
                }
            } else {
                Err(TransactionError::NotFound(transaction.tx, transaction.client))
            }
        } else {
            Err(TransactionError::AccountNotFound(transaction.client))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{Transaction, Type as TransactionType};

    // Test processing a deposit transaction
    #[test]
    fn test_process_deposit() {
        let mut engine = Engine::new();
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(1000.0),
            disputed: false,
        };

        engine.process_transaction(deposit_tx).expect("Failed to process deposit transaction");

        let account = engine.accounts.get(&1).expect("Account not found after deposit transaction");
        assert_eq!(account.available, 1000.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total, 1000.0);
        assert!(!account.locked);
    }

    // Test processing a withdrawal transaction
    #[test]
    fn test_process_withdrawal() {
        let mut engine = Engine::new();

        // First, deposit some funds
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(1000.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Then, withdraw some funds
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx).expect("Failed to process withdrawal");

        let account = engine.accounts.get(&1).expect("Account not found after withdrawal");
        assert_eq!(account.available, 500.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total, 500.0);
        assert!(!account.locked);
    }

    // Test withdrawal with insufficient funds
    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut engine = Engine::new();

        // Deposit some funds
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(300.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Attempt to withdraw more than available
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(500.0),
            disputed: false,
        };
        let result = engine.process_transaction(withdrawal_tx);

        assert!(result.is_err());
        if let Err(TransactionError::InsufficientFunds(client_id)) = result {
            assert_eq!(client_id, 1);
        } else {
            panic!("Expected InsufficientFunds error");
        }

        // Account balances should remain unchanged
        let account = engine.accounts.get(&1).expect("Account not found after insufficient funds withdrawal");
        assert_eq!(account.available, 300.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total, 300.0);
        assert!(!account.locked);
    }

    // Test processing a dispute on a deposit
    #[test]
    fn test_process_dispute_deposit() {
        let mut engine = Engine::new();

        // Deposit funds
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(1000.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Initiate a dispute on the deposit
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        // Check account balances
        let account = engine.accounts.get(&1).expect("Account not found after dispute");
        assert_eq!(account.available, 0.0);
        assert_eq!(account.held, 1000.0);
        assert_eq!(account.total, 1000.0);
        assert!(!account.locked);
    }

    // Test resolving a dispute
    #[test]
    fn test_process_resolve() {
        let mut engine = Engine::new();

        // Deposit funds and dispute
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        // Resolve the dispute
        let resolve_tx = Transaction {
            t_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(resolve_tx).expect("Failed to process resolve");

        // Check account balances
        let account = engine.accounts.get(&1).expect("Account not found after resolve");
        assert_eq!(account.available, 500.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total, 500.0);
        assert!(!account.locked);
    }

    // Test chargeback processing
    #[test]
    fn test_process_chargeback() {
        let mut engine = Engine::new();

        // Deposit funds and dispute
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(400.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        // Process chargeback
        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx).expect("Failed to process chargeback");

        // Check account balances and locked status
        let account = engine.accounts.get(&1).expect("Account not found after chargeback");
        assert_eq!(account.available, 0.0);
        assert_eq!(account.held, 0.0);
        assert_eq!(account.total, 0.0);
        assert!(account.locked);
    }

    // Test attempting transactions on a locked account
    #[test]
    fn test_transaction_on_locked_account() {
        let mut engine = Engine::new();

        // Deposit funds and process chargeback to lock the account
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(400.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx).expect("Failed to process chargeback");

        // Attempt to process a new deposit on the locked account
        let new_deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: Some(100.0),
            disputed: false,
        };
        let result = engine.process_transaction(new_deposit_tx);
        assert!(result.is_err());
        if let Err(TransactionError::AccountLocked(client_id)) = result {
            assert_eq!(client_id, 1);
        } else {
            panic!("Expected AccountLocked error");
        }
    }

    // Test dispute on a non-existent transaction
    #[test]
    fn test_dispute_nonexistent_transaction() {
        let mut engine = Engine::new();

        // Attempt to dispute a transaction that doesn't exist
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 999, // Non-existent transaction ID
            amount: None,
            disputed: false,
        };
        let result = engine.process_transaction(dispute_tx);

        assert!(result.is_err());
        if let Err(TransactionError::NotFound(tx_id, client_id)) = result {
            assert_eq!(tx_id, 999);
            assert_eq!(client_id, 1);
        } else {
            panic!("Expected TransactionNotFound error");
        }
    }

    // Test duplicate dispute on the same transaction
    #[test]
    fn test_duplicate_dispute() {
        let mut engine = Engine::new();

        // Deposit funds and dispute
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(300.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        // Attempt to dispute again
        let duplicate_dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        let result = engine.process_transaction(duplicate_dispute_tx);

        assert!(result.is_err());
        if let Err(TransactionError::AlreadyDisputed(tx_id)) = result {
            assert_eq!(tx_id, 1);
        } else {
            panic!("Expected TransactionAlreadyDisputed error");
        }
    }

    // Test resolve on a non-disputed transaction
    #[test]
    fn test_resolve_non_disputed_transaction() {
        let mut engine = Engine::new();

        // Deposit funds without dispute
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(200.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Attempt to resolve without a prior dispute
        let resolve_tx = Transaction {
            t_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        let result = engine.process_transaction(resolve_tx);

        assert!(result.is_err());
        if let Err(TransactionError::NotUnderDispute(tx_id)) = result {
            assert_eq!(tx_id, 1);
        } else {
            panic!("Expected TransactionNotUnderDispute error");
        }
    }

    // Test chargeback on a non-disputed transaction
    #[test]
    fn test_chargeback_non_disputed_transaction() {
        let mut engine = Engine::new();

        // Deposit funds without dispute
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(200.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Attempt to chargeback without a prior dispute
        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        let result = engine.process_transaction(chargeback_tx);

        assert!(result.is_err());
        if let Err(TransactionError::NotUnderDispute(tx_id)) = result {
            assert_eq!(tx_id, 1);
        } else {
            panic!("Expected TransactionNotUnderDispute error");
        }
    }

    // Test dispute on a withdrawal (should fail)
    #[test]
    fn test_dispute_withdrawal() {
        let mut engine = Engine::new();

        // Deposit funds
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Withdraw funds
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(200.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx).expect("Failed to process withdrawal");

        // Attempt to dispute the withdrawal
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 2,
            amount: None,
            disputed: false,
        };
        let result = engine.process_transaction(dispute_tx);

        assert!(result.is_err());
        if let Err(TransactionError::InvalidDispute(tx_id)) = result {
            assert_eq!(tx_id, 2);
        } else {
            panic!("Expected InvalidDisputeTransaction error");
        }
    }

    // Test processing multiple clients
    #[test]
    fn test_multiple_clients() {
        let mut engine = Engine::new();

        // Client 1 deposits
        let deposit_tx1 = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(1000.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx1).expect("Failed to process deposit for client 1");

        // Client 2 deposits
        let deposit_tx2 = Transaction {
            t_type: TransactionType::Deposit,
            client: 2,
            tx: 2,
            amount: Some(2000.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx2).expect("Failed to process deposit for client 2");

        // Client 1 withdraws
        let withdrawal_tx1 = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 3,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx1).expect("Failed to process withdrawal for client 1");

        // Client 2 disputes their deposit
        let dispute_tx2 = Transaction {
            t_type: TransactionType::Dispute,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx2).expect("Failed to process dispute for client 2");

        // Client 2 chargebacks the disputed transaction
        let chargeback_tx2 = Transaction {
            t_type: TransactionType::Chargeback,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx2).expect("Failed to process chargeback for client 2");

        // Verify Client 1's account
        let account1 = engine.accounts.get(&1).expect("Account 1 not found");
        assert_eq!(account1.available, 500.0);
        assert_eq!(account1.held, 0.0);
        assert_eq!(account1.total, 500.0);
        assert!(!account1.locked);

        // Verify Client 2's account
        let account2 = engine.accounts.get(&2).expect("Account 2 not found");
        assert_eq!(account2.available, 0.0);
        assert_eq!(account2.held, 0.0);
        assert_eq!(account2.total, 0.0);
        assert!(account2.locked);
    }

    // Test attempting to process a transaction for a locked account
    #[test]
    fn test_locked_account_transaction_rejection() {
        let mut engine = Engine::new();

        // Deposit and chargeback to lock the account
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 2,
            tx: 2,
            amount: Some(1000.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx).expect("Failed to process chargeback");

        // Attempt to process another deposit
        let new_deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 2,
            tx: 3,
            amount: Some(500.0),
            disputed: false,
        };
        let result = engine.process_transaction(new_deposit_tx);

        assert!(result.is_err());
        if let Err(TransactionError::AccountLocked(client_id)) = result {
            assert_eq!(client_id, 2);
        } else {
            panic!("Expected AccountLocked error");
        }
    }

    // Test insufficient funds for withdrawal after dispute
    #[test]
    fn test_withdrawal_after_dispute() {
        let mut engine = Engine::new();

        // Deposit funds
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit");

        // Dispute the deposit
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).expect("Failed to process dispute");

        // Attempt to withdraw funds (should fail due to insufficient available funds)
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(100.0),
            disputed: false,
        };
        let result = engine.process_transaction(withdrawal_tx);

        assert!(result.is_err());
        if let Err(TransactionError::InsufficientFunds(client_id)) = result {
            assert_eq!(client_id, 1);
        } else {
            panic!("Expected InsufficientFunds error");
        }
    }

    #[test]
    fn test_invalid_amount_in_deposit() {
        let mut engine = Engine::new();

        // Attempt to deposit with an invalid (None) amount
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: None,  // Invalid amount
            disputed: false,
        };
        let result = engine.process_transaction(deposit_tx);

        assert!(result.is_err());
        if let Err(TransactionError::InvalidAmount(tx_id)) = result {
            assert_eq!(tx_id, 1);
        } else {
            panic!("Expected InvalidAmount error for deposit transaction");
        }
    }

    #[test]
    fn test_invalid_amount_in_withdrawal() {
        let mut engine = Engine::new();

        // Attempt to withdraw with an invalid (None) amount
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: None,  // Invalid amount
            disputed: false,
        };
        let result = engine.process_transaction(withdrawal_tx);

        assert!(result.is_err());
        if let Err(TransactionError::InvalidAmount(tx_id)) = result {
            assert_eq!(tx_id, 2);
        } else {
            panic!("Expected InvalidAmount error for withdrawal transaction");
        }
    }

    #[test]
    fn test_invalid_dispute_on_non_deposit_transaction() {
        let mut engine = Engine::new();

        // Register a deposit
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(600.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit transaction");

        // Register a withdrawal transaction
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx).expect("Failed to process withdrawal");

        // Attempt to dispute the withdrawal (invalid operation)
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 2,
            amount: None,
            disputed: false,
        };
        let result = engine.process_transaction(dispute_tx);

        assert!(result.is_err());
        if let Err(TransactionError::InvalidDispute(tx_id)) = result {
            assert_eq!(tx_id, 2);
        } else {
            panic!("Expected InvalidDispute error for disputing a withdrawal transaction");
        }
    }

    #[test]
    fn test_invalid_chargeback_on_non_deposit_transaction() {
        let mut engine = Engine::new();

        // Register a deposit
        let deposit_tx = Transaction {
            t_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(600.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx).expect("Failed to process deposit transaction");

        // Register a withdrawal transaction
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(500.0),
            disputed: true, // intentionally set to cover edge case error handling :-)
        };
        engine.process_transaction(withdrawal_tx).expect("Failed to process withdrawal");

        // Attempt to chargeback the withdrawal (invalid operation)
        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 1,
            tx: 2,
            amount: None,
            disputed: false,
        };

        let result = engine.process_transaction(chargeback_tx);
        assert!(result.is_err());
        if let Err(TransactionError::InvalidChargeback(tx_id)) = result {
            assert_eq!(tx_id, 2);
        } else {
            panic!("Expected InvalidChargeback error for charging back a withdrawal transaction");
        }
    }

}
