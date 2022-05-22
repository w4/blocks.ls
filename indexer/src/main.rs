extern crate core;

mod config;
mod database;

use crate::{
    config::{Config, DatabaseConfig},
    database::Database,
};
use bitcoincore_rpc_async::{
    bitcoin::{Address, Block, BlockHash, Network, Transaction, TxIn, TxOut},
    Auth, Client, RpcApi,
};
use chrono::{TimeZone, Utc};
use clap::Parser;
use futures::stream::FuturesOrdered;
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::Instant;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(args.logging_level())
        .init();

    let bitcoin_rpc = Arc::new(
        Client::new(
            args.config.bitcoin_rpc.address,
            Auth::UserPass(
                args.config.bitcoin_rpc.username,
                args.config.bitcoin_rpc.password,
            ),
        )
        .await?,
    );

    let database = Database::new(args.config.database)?;
    database::migrations::runner()
        .run_async(&mut **database.get().await?)
        .await?;

    let height = bitcoin_rpc.get_block_count().await?;
    eprintln!("Current block height: {}", height);

    let start = 737000;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(u64, BlockHash, Block)>(200);

    let start_time = Instant::now();

    let fetch_blocks = tokio::spawn(async move {
        let mut blocks_fetching = FuturesOrdered::new();

        let mut height = start;

        loop {
            tokio::select! {
                Some(task) = blocks_fetching.next() => {
                    let task: Result<_, _> = task;
                    tx.send(task.unwrap()).await.unwrap();
                }
                _ = async {}, if blocks_fetching.len() < 20 => {
                    let bitcoin_rpc = bitcoin_rpc.clone();

                    if (height % 100) == 0 && (height - start) > 500 {
                        eprintln!("Average per tx fetched/s: {}. Current {}", (height - start) / start_time.elapsed().as_secs(), height);
                    }

                    blocks_fetching.push(tokio::spawn(async move {
                        let hash = bitcoin_rpc.get_block_hash(height).await.unwrap();
                        let block = bitcoin_rpc.get_block(&hash).await.unwrap();

                        (height, hash, block)
                    }));

                    height += 1;
                }
            }
        }
    });

    let process_blocks = tokio::spawn(async move {
        let mut database = database.get().await.unwrap();

        while let Some((height, hash, block)) = rx.recv().await {
            process_block(database.as_mut(), height as i64, hash, block)
                .await
                .unwrap();

            if (height % 100) == 0 && height > 500 {
                eprintln!(
                    "Average processed/s: {}. Current {}",
                    height / start_time.elapsed().as_secs(),
                    height
                );
            }
        }
    });

    tokio::try_join!(fetch_blocks, process_blocks)?;

    Ok(())
}

pub async fn process_block(
    database: &mut tokio_postgres::Client,
    height: i64,
    hash: BlockHash,
    block: Block,
) -> Result<(), Box<dyn std::error::Error>> {
    let tx = database.transaction().await?;

    let block_id: i64 = insert_block(&tx, height, &block, &hash).await?;

    {
        let tx = &tx;

        for transaction in block.txdata {
            let transaction_id = insert_transaction(tx, block_id, &transaction).await?;

            futures::future::try_join(
                futures::future::try_join_all(transaction.input.iter().enumerate().map(
                    |(index, transaction_in)| {
                        insert_transaction_input(tx, index as i64, transaction_id, transaction_in)
                    },
                )),
                futures::future::try_join_all(transaction.output.iter().enumerate().map(
                    |(index, transaction_out)| {
                        insert_transaction_output(tx, index as i64, transaction_id, transaction_out)
                    },
                )),
            )
            .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

async fn insert_block(
    tx: &tokio_postgres::Transaction<'_>,
    height: i64,
    block: &Block,
    block_hash: &BlockHash,
) -> Result<i64, Box<dyn std::error::Error>> {
    let query = "
        WITH inserted AS (
            INSERT INTO blocks
            (hash, height, version, size, merkle_root_hash, timestamp, bits, nonce, difficulty)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT DO NOTHING
            RETURNING id
        ) SELECT COALESCE(
            (SELECT id FROM inserted),
            (SELECT id FROM blocks WHERE hash = $1)
        ) AS id
    ";

    // TODO: previous_block_id
    Ok(tx
        .query_one(
            query,
            &[
                &block_hash.to_vec(),
                &height,
                &block.header.version,
                &(block.get_size() as i32),
                &block.header.merkle_root.to_vec(),
                &Utc.timestamp(block.header.time as i64, 0).naive_utc(),
                &(block.header.bits as i32),
                &(block.header.nonce as i32),
                &(block.header.difficulty(Network::Bitcoin) as i64),
            ],
        )
        .await?
        .get("id"))
}

async fn insert_transaction(
    tx: &tokio_postgres::Transaction<'_>,
    block_id: i64,
    transaction: &Transaction,
) -> Result<i64, Box<dyn std::error::Error>> {
    let query = "
        INSERT INTO transactions
        (hash, block_id, version, lock_time, weight, coinbase, replace_by_fee)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (hash) DO UPDATE
            SET block_id = excluded.block_id
        RETURNING id
    ";

    Ok(tx
        .query_one(
            query,
            &[
                &transaction.wtxid().to_vec(),
                &block_id,
                &transaction.version,
                &(transaction.lock_time as i32),
                &(transaction.get_weight() as i64),
                &transaction.is_coin_base(),
                &transaction.is_explicitly_rbf(),
            ],
        )
        .await?
        .get("id"))
}

async fn insert_transaction_input(
    tx: &tokio_postgres::Transaction<'_>,
    index: i64,
    transaction_id: i64,
    transaction_input: &TxIn,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = "
        INSERT INTO transaction_inputs
        (transaction_id, index, sequence, witness, script, previous_output)
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            (
                SELECT transaction_outputs.id
                FROM transactions
                INNER JOIN transaction_outputs
                    ON transactions.id = transaction_outputs.transaction_id
                WHERE transactions.hash = $6
                    AND transaction_outputs.index = $7
            )
        )
        ON CONFLICT DO NOTHING
    ";

    tx.execute(
        query,
        &[
            &transaction_id,
            &index,
            &i64::from(transaction_input.sequence),
            &transaction_input.witness.to_vec(),
            &transaction_input.script_sig.as_bytes(),
            &transaction_input.previous_output.txid.to_vec(),
            &(transaction_input.previous_output.vout as i64),
        ],
    )
    .await?;

    Ok(())
}

async fn insert_transaction_output(
    tx: &tokio_postgres::Transaction<'_>,
    index: i64,
    transaction_id: i64,
    transaction_output: &TxOut,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = "
        INSERT INTO transaction_outputs
        (transaction_id, index, value, script, unspendable, address)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT DO NOTHING
    ";

    tx.execute(
        query,
        &[
            &transaction_id,
            &index,
            &(transaction_output.value as i64),
            &transaction_output.script_pubkey.as_bytes(),
            &transaction_output.script_pubkey.is_provably_unspendable(),
            &Address::from_script(&transaction_output.script_pubkey, Network::Bitcoin)
                .map(|v| v.to_string()),
        ],
    )
    .await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Logging verbosity
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,
    #[clap(short, long, parse(try_from_str = Config::from_toml_path))]
    pub config: Config,
}

impl Args {
    #[must_use]
    pub fn logging_level(&self) -> Level {
        match self.verbose {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        }
    }
}
