use crate::database::{Connection, Result};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use tokio::time::Instant;
use tokio_postgres::types::{Json, ToSql};
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
    #[serde(deserialize_with = "trim_hex_prefix")]
    pub script: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionOutput {
    pub value: i64,
    #[serde(deserialize_with = "trim_hex_prefix")]
    pub script: String,
    pub unspendable: bool,
    pub address: Option<String>,
}

fn trim_hex_prefix<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<String, D::Error> {
    let mut s = String::deserialize(deserializer)?;
    s.remove(0);
    s.remove(0);
    Ok(s)
}

pub async fn fetch_transactions_for_block(
    db: &Connection,
    id: i64,
    limit: i64,
    offset: i64,
) -> Result<(i64, Vec<Transaction>)> {
    let count_query = "
        SELECT COUNT(*) AS count
        FROM transactions
        WHERE transactions.block_id = $1
    ";

    let count_query_params: &[&(dyn ToSql + Sync)] = &[&id];

    let select_query = "
        SELECT
            transactions.*,
            (
                SELECT JSON_AGG(transaction_inputs)
                FROM (
                    SELECT ROW_TO_JSON(transaction_outputs) AS previous_output_tx, transaction_inputs.*
                    FROM transaction_inputs
                    LEFT JOIN transaction_outputs
                        ON transaction_outputs.id = transaction_inputs.previous_output
                    WHERE transactions.id = transaction_inputs.transaction_id
                ) transaction_inputs
            ) AS inputs,
            (
                SELECT JSON_AGG(transaction_outputs.*)
                FROM transaction_outputs
                WHERE transactions.id = transaction_outputs.transaction_id
            ) AS outputs
        FROM transactions
        WHERE transactions.block_id = $1
        ORDER BY transactions.id ASC
        LIMIT $2 OFFSET $3
    ";

    let select_query_params: &[&(dyn ToSql + Sync)] = &[&id, &limit, &offset];

    let (count, transactions) = tokio::try_join!(
        db.query_one(count_query, count_query_params),
        db.query(select_query, select_query_params)
    )?;

    Ok((
        count.try_get("count")?,
        transactions
            .into_iter()
            .map(Transaction::from_row)
            .collect::<Result<_>>()?,
    ))
}

pub async fn fetch_transactions_for_address(
    db: &Connection,
    address: &str,
) -> Result<Vec<Transaction>> {
    let select_query = "
        SELECT
	            transactions.*,
	            (
	                SELECT JSON_AGG(transaction_inputs)
	                FROM (
	                    SELECT ROW_TO_JSON(transaction_outputs) AS previous_output_tx, transaction_inputs.*
	                    FROM transaction_inputs
	                    LEFT JOIN transaction_outputs
	                        ON transaction_outputs.id = transaction_inputs.previous_output
	                    WHERE transactions.id = transaction_inputs.transaction_id
	                ) transaction_inputs
	            ) AS inputs,
	            (
	                SELECT JSON_AGG(transaction_outputs.*)
	                FROM transaction_outputs
	                WHERE transactions.id = transaction_outputs.transaction_id
	            ) AS outputs
	        FROM transactions
	        WHERE transactions.id IN (
	        	SELECT transaction_outputs.transaction_id
                    FROM transaction_outputs
                    WHERE transaction_outputs.address = $1
	        	UNION
	        	SELECT transaction_inputs.transaction_id
                    FROM transaction_inputs
                    LEFT JOIN transaction_outputs
                        ON transaction_outputs.id = transaction_inputs.previous_output
                    WHERE transaction_outputs.address = $1
	        )
            ORDER BY transactions.id DESC
    ";

    let transactions = db.query(select_query, &[&address]).await?;

    transactions
        .into_iter()
        .map(Transaction::from_row)
        .collect()
}
