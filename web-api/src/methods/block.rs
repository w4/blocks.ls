use crate::Database;
use axum::extract::{Path, Query};
use axum::{Extension, Json};
use bitcoin::blockdata::constants::WITNESS_SCALE_FACTOR;
use bitcoin::VarInt;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct MinedBy {
    pool: &'static str,
}

impl From<Pool> for MinedBy {
    fn from(pool: Pool) -> Self {
        Self { pool: pool.name() }
    }
}

#[derive(Serialize)]
pub struct BlockList {
    hash: String,
    mined_by: Option<MinedBy>,
    height: i64,
    version: i32,
    timestamp: NaiveDateTime,
    bits: i32,
    nonce: u32,
    difficulty: i64,
    weight: u64,
    tx_count: i64,
    size: i32,
}

#[derive(Deserialize)]
pub struct ListParams {
    #[serde(default)]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

pub async fn list(
    Extension(database): Extension<Database>,
    Query(params): Query<ListParams>,
) -> Json<Vec<BlockList>> {
    let database = database.get().await.unwrap();

    let limit = std::cmp::min(20, std::cmp::max(5, params.limit));
    let offset = params.offset;

    let blocks = crate::database::blocks::fetch_latest_blocks(
        &database,
        i64::from(limit),
        i64::from(offset),
    )
    .await
    .unwrap();

    Json(
        blocks
            .into_iter()
            .map(|(mut block, tx_count, tx_weight, coinbase_script)| {
                // TODO: do this on insert
                block.hash.reverse();

                BlockList {
                    hash: hex::encode(block.hash),
                    mined_by: Pool::fetch_from_script(&coinbase_script).map(Into::into),
                    height: block.height,
                    version: block.version,
                    timestamp: block.timestamp,
                    size: block.size,
                    bits: block.bits,
                    nonce: block.nonce,
                    difficulty: block.difficulty,
                    weight: (u64::try_from(WITNESS_SCALE_FACTOR).unwrap()
                        * u64::try_from(VarInt(u64::try_from(tx_count).unwrap()).len()).unwrap())
                        + u64::try_from(tx_weight.mantissa()).unwrap(),
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
    pub hash: String,
    pub version: i32,
    pub weight: i64,
    pub lock_time: i32,
    pub coinbase: bool,
    pub replace_by_fee: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<TransactionInput>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<TransactionOutput>,
}

impl From<crate::database::transactions::Transaction> for Transaction {
    fn from(mut tx: crate::database::transactions::Transaction) -> Self {
        tx.hash.reverse();

        Transaction {
            hash: hex::encode(tx.hash),
            version: tx.version,
            weight: tx.weight,
            lock_time: tx.lock_time,
            coinbase: tx.coinbase,
            replace_by_fee: tx.replace_by_fee,
            inputs: tx.inputs.0.into_iter().map(Into::into).collect(),
            outputs: tx.outputs.0.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct PreviousOutput {
    #[serde(flatten)]
    output: TransactionOutput,
    tx_hash: String,
    tx_index: i64,
}

#[derive(Serialize)]
pub struct TransactionInput {
    witness: Vec<String>,
    sequence: i64,
    previous_output: Option<PreviousOutput>,
    script: String,
}

impl From<crate::database::transactions::TransactionInput> for TransactionInput {
    fn from(txi: crate::database::transactions::TransactionInput) -> Self {
        Self {
            witness: txi.witness.into_iter().map(hex::encode).collect(),
            sequence: txi.sequence,
            previous_output: txi.previous_output.map(|v| PreviousOutput {
                tx_index: v.index,
                output: v.into(),
                tx_hash: txi
                    .previous_output_tx_hash
                    .map(|mut h| {
                        h.reverse();
                        hex::encode(h)
                    })
                    .unwrap_or_default(),
            }),
            script: txi.script,
        }
    }
}

#[derive(Serialize)]
pub struct TransactionOutput {
    index: i64,
    value: i64,
    script: String,
    unspendable: bool,
    address: Option<String>,
}

impl From<crate::database::transactions::TransactionOutput> for TransactionOutput {
    fn from(txo: crate::database::transactions::TransactionOutput) -> Self {
        Self {
            index: txo.index,
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
        transactions: transactions.into_iter().map(Into::into).collect(),
    };

    Json(GetResponse {
        tx_count: count,
        block,
    })
}

pub enum Pool {
    Luxor,
    F2Pool,
    Binance,
    FoundryUsa,
    Slush,
    Poolin,
    ViaBtc,
    BtcCom,
    AntPool,
    MaraPool,
    SbiCrypto,
}

impl Pool {
    pub fn name(&self) -> &'static str {
        match self {
            Pool::Luxor => "Luxor",
            Pool::F2Pool => "F2Pool",
            Pool::Binance => "Binance",
            Pool::FoundryUsa => "Foundry USA",
            Pool::Slush => "Slush",
            Pool::Poolin => "Poolin",
            Pool::ViaBtc => "ViaBTC",
            Pool::BtcCom => "BTC.com",
            Pool::AntPool => "AntPool",
            Pool::MaraPool => "MaraPool",
            Pool::SbiCrypto => "SBICrypto",
        }
    }
}

impl Pool {
    fn fetch_from_script(coinbase_script: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(coinbase_script);

        macro_rules! define {
            ($($signature:expr => $pool:ident,)*) => {
                if false {
                    None
                }
                $(
                    else if text.contains($signature) {
                        Some(Self::$pool)
                    }
                )*
                else {
                    None
                }
            }
        }

        define! {
            "Powered by Luxor Tech" => Luxor,
            "F2Pool" => F2Pool,
            "binance" => Binance,
            "Foundry USA Pool" => FoundryUsa,
            "slush" => Slush,
            "poolin.com" => Poolin,
            "ViaBTC" => ViaBtc,
            "btcpool" => BtcCom,
            "Mined by AntPool" => AntPool,
            "MARA Pool" => MaraPool,
            "SBICrypto" => SbiCrypto,
        }
    }
}
