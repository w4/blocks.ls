use crate::database::{Connection, Result};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use tokio_postgres::{
    types::{Json, ToSql},
    Row,
};

#[derive(Debug)]
pub struct Transaction {
    pub hash: Vec<u8>,
    pub version: i32,
    pub weight: i64,
    pub lock_time: i32,
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
            weight: row.try_get("weight")?,
            lock_time: row.try_get("lock_time")?,
            coinbase: row.try_get("coinbase")?,
            replace_by_fee: row.try_get("replace_by_fee")?,
            inputs: row.try_get("inputs")?,
            outputs: row.try_get("outputs")?,
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct TransactionInput {
    pub sequence: i64,
    #[serde(deserialize_with = "trim_hex_prefix_vec")]
    pub witness: Vec<String>,
    #[serde(deserialize_with = "trim_hex_prefix")]
    pub script: String,
    #[serde(deserialize_with = "parse_hex_opt")]
    pub previous_output_tx_hash: Option<Vec<u8>>,
    #[serde(rename = "previous_output_item")]
    pub previous_output: Option<TransactionOutput>,
}

#[derive(Deserialize, Debug)]
pub struct TransactionOutput {
    pub index: i64,
    pub value: i64,
    #[serde(deserialize_with = "trim_hex_prefix")]
    pub script: String,
    pub unspendable: bool,
    pub address: Option<String>,
}

fn parse_hex_opt<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<Vec<u8>>, D::Error> {
    let s = <Option<String>>::deserialize(deserializer)?;
    s.map(|mut s| {
        s.remove(0);
        s.remove(0);
        hex::decode(s).map_err(D::Error::custom)
    })
    .transpose()
}

fn trim_hex_prefix_vec<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Vec<String>, D::Error> {
    let s = <Vec<String>>::deserialize(deserializer)?;
    Ok(s.into_iter()
        .map(|mut s| {
            s.remove(0);
            s.remove(0);
            s
        })
        .collect())
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
                    SELECT
                        pot.hash AS previous_output_tx_hash,
                        ROW_TO_JSON(po) AS previous_output_item,
                        transaction_inputs.*
                    FROM transaction_inputs
                    LEFT JOIN transactions pot
                        ON pot.hash = transaction_inputs.previous_output_transaction
                    LEFT JOIN transaction_outputs po
                    	ON po.transaction_id = pot.id
                    	AND po.index = transaction_inputs.previous_output_index
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
	                    SELECT
                            pot.hash AS previous_output_tx_hash,
	                        ROW_TO_JSON(po) AS previous_output_item,
	                        transaction_inputs.*
	                    FROM transaction_inputs
                        LEFT JOIN transactions pot
                            ON pot.hash = transaction_inputs.previous_output_transaction
                        LEFT JOIN transaction_outputs po
                    	    ON po.transaction_id = pot.id
                    	    AND po.index = transaction_inputs.previous_output_index
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
                    LEFT JOIN transactions pot
                        ON pot.hash = transaction_inputs.previous_output_transaction
                    LEFT JOIN transaction_outputs po
                    	ON po.transaction_id = pot.id
                    	AND po.index = transaction_inputs.previous_output_index
                    WHERE po.address = $1
	        )
            ORDER BY transactions.id DESC
    ";

    let transactions = db.query(select_query, &[&address]).await?;

    transactions
        .into_iter()
        .map(Transaction::from_row)
        .collect()
}

pub struct TransactionWithDetails {
    pub input_total_value: rust_decimal::Decimal,
    pub output_total_value: rust_decimal::Decimal,
    pub transaction: Transaction,
}

pub async fn fetch_latest_transactions(
    db: &Connection,
    limit: i64,
) -> Result<Vec<TransactionWithDetails>> {
    let select_query = "
        SELECT transactions.*,
            JSON_BUILD_ARRAY() AS inputs,
            JSON_BUILD_ARRAY() AS outputs,
            (
                SELECT SUM(po.value)
                FROM transaction_inputs input
                LEFT JOIN transactions pot
                    ON pot.hash = input.previous_output_transaction
                LEFT JOIN transaction_outputs po
                    ON po.transaction_id = pot.id
                    AND po.index = input.previous_output_index
                WHERE input.transaction_id = transactions.id
            ) AS input_total_value,
            (
                SELECT SUM(out.value)
                FROM transaction_outputs out
                WHERE out.transaction_id = transactions.id
            ) AS output_total_value
        FROM transactions
        ORDER BY transactions.id DESC
        LIMIT $1
    ";

    let transactions = db.query(select_query, &[&limit]).await?;

    transactions
        .into_iter()
        .map(|tx| {
            Ok(TransactionWithDetails {
                input_total_value: tx
                    .try_get::<_, Option<_>>("input_total_value")?
                    .unwrap_or_default(),
                output_total_value: tx
                    .try_get::<_, Option<_>>("output_total_value")?
                    .unwrap_or_default(),
                transaction: Transaction::from_row(tx)?,
            })
        })
        .collect()
}

pub async fn fetch_transaction_by_hash(
    db: &Connection,
    hash: &[u8],
) -> Result<Option<Transaction>> {
    let select_query = "
        SELECT
	            transactions.*,
	            (
	                SELECT JSON_AGG(transaction_inputs)
	                FROM (
	                    SELECT
                            pot.hash AS previous_output_tx_hash,
	                        ROW_TO_JSON(po) AS previous_output_item,
	                        transaction_inputs.*
	                    FROM transaction_inputs
	                    LEFT JOIN transactions pot
                            ON pot.hash = transaction_inputs.previous_output_transaction
                        LEFT JOIN transaction_outputs po
                            ON po.transaction_id = pot.id
                            AND po.index = transaction_inputs.previous_output_index
	                    WHERE transactions.id = transaction_inputs.transaction_id
	                ) transaction_inputs
	            ) AS inputs,
	            (
	                SELECT JSON_AGG(transaction_outputs.*)
	                FROM transaction_outputs
	                WHERE transactions.id = transaction_outputs.transaction_id
	            ) AS outputs
	        FROM transactions
	        WHERE transactions.hash = $1
    ";

    let transaction = db.query_opt(select_query, &[&hash]).await?;

    transaction.map(Transaction::from_row).transpose()
}
