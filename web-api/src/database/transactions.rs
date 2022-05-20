use crate::database::{Connection, Result};
use serde::Deserialize;
use tokio::time::Instant;
use tokio_postgres::types::Json;
use tokio_postgres::Row;

#[derive(Debug)]
pub struct Transaction {
    pub hash: Vec<u8>,
    pub version: i32,
    pub lock_time: i32,
    pub weight: i64,
    pub coinbase: bool,
    pub replace_by_fee: bool,
    pub inputs: Json<Vec<TransactionInput>>,
    pub outputs: Json<Vec<TransactionOutput>>,
}

impl Transaction {
    pub fn from_row(row: Row) -> Result<Self> {
        Ok(Self {
            hash: row.try_get("hash")?,
            version: row.try_get("version")?,
            lock_time: row.try_get("lock_time")?,
            weight: row.try_get("weight")?,
            coinbase: row.try_get("coinbase")?,
            replace_by_fee: row.try_get("replace_by_fee")?,
            inputs: row.try_get("inputs")?,
            outputs: row.try_get("outputs")?,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct TransactionInput {
    pub previous_output_tx: Option<TransactionOutput>,
    pub script: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionOutput {
    pub value: i64,
    pub script: String,
    pub unspendable: bool,
    pub address: Option<String>,
}

pub async fn fetch_transactions_for_block(db: &Connection, id: i64) -> Result<Vec<Transaction>> {
    let transactions = db
        .query(
            "SELECT
           transactions.*,
           JSON_AGG(transaction_inputs) AS inputs,
           JSON_AGG(transaction_outputs) AS outputs
         FROM transactions
         LEFT JOIN
           (
             SELECT
               row_to_json(transaction_outputs) AS previous_output_tx,
               transaction_inputs.*
             FROM transaction_inputs
             LEFT JOIN transaction_outputs
               ON transaction_outputs.id = transaction_inputs.previous_output
           ) transaction_inputs
           ON transactions.id = transaction_inputs.transaction_id
         LEFT JOIN transaction_outputs
	       ON transactions.id = transaction_outputs.transaction_id
	     WHERE transactions.block_id = $1
	     GROUP BY transactions.id
	     ORDER BY transactions.id ASC",
            &[&id],
        )
        .await?;

    transactions
        .into_iter()
        .map(Transaction::from_row)
        .collect()
}
