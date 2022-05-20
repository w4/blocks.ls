use crate::Database;
use axum::extract::Path;
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Serialize)]
pub struct Block {
    height: i64,
    version: i32,
    size: i32,
    merkle_root_hash: String,
    // #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: NaiveDateTime,
    bits: i32,
    nonce: i32,
    difficulty: i64,
    transactions: Vec<Transaction>,
}

#[derive(Serialize)]
pub struct Transaction {
    hash: String,
    version: i32,
    lock_time: i32,
    weight: i64,
    coinbase: bool,
    replace_by_fee: bool,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

#[derive(Serialize)]
pub struct TransactionInput {
    previous_output: Option<TransactionOutput>,
    script: String,
}

impl From<crate::database::transactions::TransactionInput> for TransactionInput {
    fn from(txi: crate::database::transactions::TransactionInput) -> Self {
        Self {
            previous_output: txi.previous_output_tx.map(Into::into),
            script: txi.script,
        }
    }
}

#[derive(Serialize)]
pub struct TransactionOutput {
    value: i64,
    script: String,
    unspendable: bool,
    address: Option<String>,
}

impl From<crate::database::transactions::TransactionOutput> for TransactionOutput {
    fn from(txo: crate::database::transactions::TransactionOutput) -> Self {
        Self {
            value: txo.value,
            script: txo.script,
            unspendable: txo.unspendable,
            address: txo.address,
        }
    }
}

pub async fn handle(
    Extension(database): Extension<Database>,
    Path(height): Path<i64>,
) -> Json<Block> {
    let database = database.get().await.unwrap();

    let block = crate::database::blocks::fetch_block_by_height(&database, height)
        .await
        .unwrap()
        .unwrap();

    let transactions =
        crate::database::transactions::fetch_transactions_for_block(&database, block.id)
            .await
            .unwrap();

    Json(Block {
        height: block.height,
        version: block.version,
        size: block.size,
        merkle_root_hash: hex::encode(block.merkle_root_hash),
        timestamp: block.timestamp,
        bits: block.bits,
        nonce: block.nonce,
        difficulty: block.difficulty,
        transactions: transactions
            .into_iter()
            .map(|tx| Transaction {
                hash: hex::encode(tx.hash),
                version: tx.version,
                lock_time: tx.lock_time,
                weight: tx.weight,
                coinbase: tx.coinbase,
                replace_by_fee: tx.replace_by_fee,
                inputs: tx.inputs.0.into_iter().map(Into::into).collect(),
                outputs: tx.outputs.0.into_iter().map(Into::into).collect(),
            })
            .collect(),
    })
}