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
        let account = self.accounts.get_mut(&transaction.client).unwrap();

        if let Some(amount) = transaction.amount {
            account.available += amount;
            account.total += amount;
            self.transactions.insert(transaction.tx, transaction);
            Ok(())
        } else {
            Err(TransactionError::InvalidAmount(transaction.tx))
        }
    }

    fn process_withdrawal(&mut self, transaction: Transaction) -> Result<(), TransactionError> {
        let account = self.accounts.get_mut(&transaction.client).unwrap();

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
    }

    fn process_dispute(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let account = self.accounts.get_mut(&transaction.client).unwrap();

        if let Some(original_tx) = self.transactions.get_mut(&transaction.tx) {
            if !original_tx.disputed && original_tx.client == account.client {
                if let Some(amount) = original_tx.amount {
                    if let TransactionType::Deposit = original_tx.t_type {
                        // Procéder au litige
                        account.available -= amount;
                        account.held += amount;
                        original_tx.disputed = true;
                        Ok(())
                    } else {
                        // La transaction n'est pas un dépôt, ignorer le litige ou générer une erreur
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
    }

    fn process_resolve(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let account = self.accounts.get_mut(&transaction.client).unwrap();

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
    }

    fn process_chargeback(&mut self, transaction: &Transaction) -> Result<(), TransactionError> {
        let client_id = transaction.client;
        let account = self.accounts.get_mut(&client_id).unwrap();

        if let Some(original_tx) = self.transactions.get_mut(&transaction.tx) {
            if original_tx.disputed && original_tx.client == client_id {
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
            Err(TransactionError::NotFound(transaction.tx, client_id))
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
            // Assuming your Transaction struct includes a 'disputed' field
            disputed: false,
        };

        engine.process_transaction(deposit_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

        // Then, withdraw some funds
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx).unwrap();

        let account = engine.accounts.get(&1).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

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
        let account = engine.accounts.get(&1).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

        // Initiate a dispute on the deposit
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

        // Check account balances
        let account = engine.accounts.get(&1).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

        // Resolve the dispute
        let resolve_tx = Transaction {
            t_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(resolve_tx).unwrap();

        // Check account balances
        let account = engine.accounts.get(&1).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

        // Process chargeback
        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx).unwrap();

        // Check account balances and locked status
        let account = engine.accounts.get(&1).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx).unwrap();

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
        engine.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

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
        engine.process_transaction(deposit_tx).unwrap();

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
        engine.process_transaction(deposit_tx).unwrap();

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
        engine.process_transaction(deposit_tx).unwrap();

        // Withdraw funds
        let withdrawal_tx = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some(200.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx).unwrap();

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
        engine.process_transaction(deposit_tx1).unwrap();

        // Client 2 deposits
        let deposit_tx2 = Transaction {
            t_type: TransactionType::Deposit,
            client: 2,
            tx: 2,
            amount: Some(2000.0),
            disputed: false,
        };
        engine.process_transaction(deposit_tx2).unwrap();

        // Client 1 withdraws
        let withdrawal_tx1 = Transaction {
            t_type: TransactionType::Withdrawal,
            client: 1,
            tx: 3,
            amount: Some(500.0),
            disputed: false,
        };
        engine.process_transaction(withdrawal_tx1).unwrap();

        // Client 2 disputes their deposit
        let dispute_tx2 = Transaction {
            t_type: TransactionType::Dispute,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx2).unwrap();

        // Client 2 chargebacks the disputed transaction
        let chargeback_tx2 = Transaction {
            t_type: TransactionType::Chargeback,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx2).unwrap();

        // Verify Client 1's account
        let account1 = engine.accounts.get(&1).unwrap();
        assert_eq!(account1.available, 500.0);
        assert_eq!(account1.held, 0.0);
        assert_eq!(account1.total, 500.0);
        assert!(!account1.locked);

        // Verify Client 2's account
        let account2 = engine.accounts.get(&2).unwrap();
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
        engine.process_transaction(deposit_tx).unwrap();

        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

        let chargeback_tx = Transaction {
            t_type: TransactionType::Chargeback,
            client: 2,
            tx: 2,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(chargeback_tx).unwrap();

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
        engine.process_transaction(deposit_tx).unwrap();

        // Dispute the deposit
        let dispute_tx = Transaction {
            t_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
            disputed: false,
        };
        engine.process_transaction(dispute_tx).unwrap();

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
}
