use crate::database::{Connection, Result};
use chrono::NaiveDateTime;
use tokio_postgres::Row;

#[derive(Debug)]
pub struct Block {
    pub hash: Vec<u8>,
    pub id: i64,
    pub height: i64,
    pub version: i32,
    pub size: i32,
    pub merkle_root_hash: Vec<u8>,
    pub timestamp: NaiveDateTime,
    pub bits: i32,
    pub nonce: u32,
    pub difficulty: i64,
}

impl Block {
    pub fn from_row(row: Row) -> Result<Self> {
        Ok(Self {
            hash: row.try_get("hash")?,
            id: row.try_get("id")?,
            height: row.try_get("height")?,
            version: row.try_get("version")?,
            size: row.try_get("size")?,
            merkle_root_hash: row.try_get("merkle_root_hash")?,
            timestamp: row.try_get("timestamp")?,
            bits: row.try_get("bits")?,
            nonce: row.try_get::<_, i32>("nonce")? as u32, // TODO
            difficulty: row.try_get("difficulty")?,
        })
    }
}

pub async fn fetch_height(db: &Connection) -> Result<u64> {
    let row = db
        .query_one("SELECT MAX(height) AS height FROM blocks", &[])
        .await?;
    let height: i64 = row.try_get("height")?;
    Ok(u64::try_from(height)?)
}

pub type TransactionCount = i64;
pub type TransactionWeight = rust_decimal::Decimal;

pub async fn fetch_latest_blocks(
    db: &Connection,
    count: i64,
    offset: i64,
) -> Result<Vec<(Block, TransactionCount, TransactionWeight, Vec<u8>)>> {
    let query = "
        SELECT
            blocks.*,
            tx.count AS tx_count,
            tx.weight AS tx_weight,
            (
                SELECT script
                FROM transactions
                INNER JOIN transaction_inputs
                ON transaction_inputs.transaction_id = transactions.id
                WHERE transactions.block_id = blocks.id
                AND transactions.coinbase = true
                LIMIT 1
            ) AS coinbase_script
        FROM blocks
        LEFT JOIN LATERAL (
            SELECT
                SUM(transactions.weight) AS weight,
                COUNT(transactions.id) AS count
            FROM transactions
            WHERE transactions.block_id = blocks.id
        ) tx ON true
        ORDER BY blocks.height DESC
        LIMIT $1 OFFSET $2
    ";

    let blocks = db
        .query(query, &[&count, &offset])
        .await?;

    blocks
        .into_iter()
        .map(|row| {
            let tx_count = row.try_get("tx_count")?;
            let tx_weight = row.try_get("tx_weight")?;
            let coinbase_script = row.try_get("coinbase_script")?;
            Ok((Block::from_row(row)?, tx_count, tx_weight, coinbase_script))
        })
        .collect::<Result<Vec<_>>>()
}

pub async fn fetch_block_by_height(db: &Connection, height: i64) -> Result<Option<Block>> {
    let query = "
        SELECT *
        FROM blocks
        WHERE height = $1
    ";

    let block = db
        .query_opt(query, &[&height])
        .await?;

    Ok(block.map(Block::from_row).transpose()?)
}
