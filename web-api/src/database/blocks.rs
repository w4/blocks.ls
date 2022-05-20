use crate::database::{Connection, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio_postgres::Row;

#[derive(Debug)]
pub struct Block {
    pub id: i64,
    pub height: i64,
    pub version: i32,
    pub size: i32,
    pub merkle_root_hash: Vec<u8>,
    pub timestamp: NaiveDateTime,
    pub bits: i32,
    pub nonce: i32,
    pub difficulty: i64,
}

impl Block {
    pub fn from_row(row: Row) -> Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            height: row.try_get("height")?,
            version: row.try_get("version")?,
            size: row.try_get("size")?,
            merkle_root_hash: row.try_get("merkle_root_hash")?,
            timestamp: row.try_get("timestamp")?,
            bits: row.try_get("bits")?,
            nonce: row.try_get("nonce")?,
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

pub async fn fetch_block_by_height(db: &Connection, height: i64) -> Result<Option<Block>> {
    let block = db
        .query_opt(
            "SELECT *
         FROM blocks
         WHERE height = $1",
            &[&height],
        )
        .await?;

    Ok(block.map(Block::from_row).transpose()?)
}
