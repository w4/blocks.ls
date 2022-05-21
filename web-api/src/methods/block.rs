use crate::Database;
use axum::extract::{Path, Query};
use axum::{Extension, Json};
use chrono::NaiveDateTime;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::ptr::hash;

#[derive(Serialize)]
pub struct BlockList {
    hash: String,
    height: i64,
    version: i32,
    timestamp: NaiveDateTime,
    bits: i32,
    nonce: u32,
    difficulty: i64,
    tx_count: i64,
}

pub async fn list(Extension(database): Extension<Database>) -> Json<Vec<BlockList>> {
    let database = database.get().await.unwrap();

    let blocks = crate::database::blocks::fetch_latest_blocks(&database, 5)
        .await
        .unwrap();

    Json(
        blocks
            .into_iter()
            .map(|(mut block, tx_count)| {
                // TODO: do this on insert
                block.hash.reverse();

                BlockList {
                    hash: hex::encode(block.hash),
                    height: block.height,
                    version: block.version,
                    timestamp: block.timestamp,
                    bits: block.bits,
                    nonce: block.nonce,
                    difficulty: block.difficulty,
                    tx_count,
                }
            })
            .collect(),
    )
}

#[derive(Serialize)]
pub struct GetResponse {
    tx_count: i64,
    #[serde(flatten)]
    block: Block,
}

#[derive(Serialize)]
pub struct Block {
    height: i64,
    version: i32,
    size: i32,
    merkle_root_hash: String,
    // #[serde(with = "chrono::serde::ts_seconds")]
    timestamp: NaiveDateTime,
    bits: i32,
    nonce: u32,
    difficulty: i64,
    transactions: Vec<Transaction>,
    hash: String,
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

#[derive(Deserialize)]
pub struct HandleQuery {
    #[serde(default)]
    offset: u32,
}

pub async fn handle(
    Extension(database): Extension<Database>,
    Path(height): Path<i64>,
    Query(query): Query<HandleQuery>,
) -> Json<GetResponse> {
    let database = database.get().await.unwrap();
    let offset = i64::from(query.offset);
    let limit = 30;

    let mut block = crate::database::blocks::fetch_block_by_height(&database, height)
        .await
        .unwrap()
        .unwrap();

    let (count, transactions) = crate::database::transactions::fetch_transactions_for_block(
        &database, block.id, limit, offset,
    )
    .await
    .unwrap();

    // TODO: do this on insert
    block.hash.reverse();

    let block = Block {
        hash: hex::encode(block.hash),
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
            .map(|mut tx| {
                tx.hash.reverse();

                Transaction {
                    hash: hex::encode(tx.hash),
                    version: tx.version,
                    lock_time: tx.lock_time,
                    weight: tx.weight,
                    coinbase: tx.coinbase,
                    replace_by_fee: tx.replace_by_fee,
                    inputs: tx.inputs.0.into_iter().map(Into::into).collect(),
                    outputs: tx.outputs.0.into_iter().map(Into::into).collect(),
                }
            })
            .collect(),
    };

    Json(GetResponse {
        tx_count: count,
        block,
    })
}
