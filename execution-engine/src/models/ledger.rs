use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub account: String,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub timestamp: i64,
    pub description: String,
    pub entries: Vec<LedgerEntry>,
}

impl Transaction {
    pub fn new(description: String, entries: Vec<LedgerEntry>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            description,
            entries,
        }
    }

    /// Verifies that the sum of all entries is zero (Double Entry Principle).
    /// Returns true if balanced.
    pub fn is_balanced(&self) -> bool {
        let sum: f64 = self.entries.iter().map(|e| e.amount).sum();
        sum.abs() < 1e-6
    }
}

pub struct TransactionLogger {
    file_path: PathBuf,
}

impl TransactionLogger {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    pub fn log(&mut self, transaction: &Transaction) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;

        // Format: Date, Description, Account, Amount, Currency, TxID
        let date = chrono::DateTime::from_timestamp(transaction.timestamp / 1000, 0)
            .unwrap_or_default()
            .to_rfc3339();

        for entry in &transaction.entries {
            writeln!(
                file,
                "{},{},{},{:.4},{},{}",
                date,
                transaction.description,
                entry.account,
                entry.amount,
                entry.currency,
                transaction.id
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_balance() {
        let t = Transaction::new(
            "Rebalance".into(),
            vec![
                LedgerEntry {
                    account: "StratA".into(),
                    amount: -100.0,
                    currency: "USD".into(),
                },
                LedgerEntry {
                    account: "Treasury".into(),
                    amount: 100.0,
                    currency: "USD".into(),
                },
            ],
        );

        assert!(t.is_balanced());
    }
}
