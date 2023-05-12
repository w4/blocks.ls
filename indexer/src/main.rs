extern crate core;

mod config;
mod database;
mod rpc;

use crate::{
    config::{Config, DatabaseConfig},
    database::Database,
};
use bitcoin::{Address, Block, BlockHash, Network, Transaction, TxIn, TxOut};
use chrono::{TimeZone, Utc};
use clap::{ArgAction, Parser};
use futures::stream::{FuturesOrdered, FuturesUnordered};
use futures::StreamExt;
use thiserror::Error;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::{error, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(args.logging_level())
        .init();

    let bitcoin_rpc = rpc::BitcoinRpc::new(&args.config.bitcoin_rpc);

    let database = Database::new(args.config.database)?;
    database::migrations::runner()
        .run_async(&mut **database.get().await?)
        .await?;

    eprintln!(
        "Current block height: {}",
        bitcoin_rpc.get_block_height().await
    );

    let start = args.start;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(u64, BlockHash, Block)>(args.buffer);

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
                _ = async {}, if blocks_fetching.len() < args.fetch_concurrent => {
                    let bitcoin_rpc = bitcoin_rpc.clone();

                    if (height % 100) == 0 && (height - start) > 500 && start_time.elapsed().as_secs() > 0 {
                        eprintln!("Average per tx fetched/s: {}. Current {}", (height - start) / start_time.elapsed().as_secs(), height);
                    }

                    blocks_fetching.push_back(tokio::spawn(async move {
                        let hash = bitcoin_rpc.get_block_hash(height).await;
                        let block = bitcoin_rpc.get_block(&hash).await;

                        (height, hash, block)
                    }));

                    height += 1;
                }
            }
        }
    });

    let process_blocks = tokio::spawn(async move {
        let mut futures: FuturesUnordered<JoinHandle<_>> = FuturesUnordered::new();
        let mut count = 0;

        loop {
            tokio::select! {
                Some(task) = futures.next() => {
                    if let Err(e) = task {
                        error!(?e, "Failed to insert block");
                    }

                    count += 1;

                    if (count % 100) == 0 && count > 500 && start_time.elapsed().as_secs() > 0 {
                        eprintln!(
                            "Average processed/s: {}. Current {}",
                            count / start_time.elapsed().as_secs(),
                            count
                        );
                    }
                }
                Some((height, hash, block)) = rx.recv() => {
                    let database = database.clone();

                    futures.push(tokio::spawn(async move {
                        let mut database = database.get().await.unwrap();
                        process_block(database.as_mut(), height as i64, hash, block).await.unwrap();
                    }));
                }
            }
        }
    });

    tokio::try_join!(fetch_blocks, process_blocks)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum ProcessBlockError {
    #[error("Failed to write to database: {0}")]
    Database(#[from] tokio_postgres::Error),
}

pub async fn process_block(
    database: &mut tokio_postgres::Client,
    height: i64,
    hash: BlockHash,
    block: Block,
) -> Result<(), ProcessBlockError> {
    let tx = database.transaction().await?;

    let block_id: i64 = insert_block(&tx, height, &block, &hash).await?;

    {
        let tx = &tx;

        futures::future::try_join_all(block.txdata.into_iter().map(|transaction| async move {
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
            .await
        }))
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn insert_block(
    tx: &tokio_postgres::Transaction<'_>,
    height: i64,
    block: &Block,
    block_hash: &BlockHash,
) -> Result<i64, tokio_postgres::Error> {
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
                &AsRef::<[u8]>::as_ref(&block_hash.as_raw_hash()),
                &height,
                &block.header.version.to_consensus(),
                &(block.size() as i32),
                &AsRef::<[u8]>::as_ref(&block.header.merkle_root.as_raw_hash()),
                &Utc.timestamp_opt(block.header.time as i64, 0)
                    .unwrap()
                    .naive_utc(),
                &(block.header.bits.to_consensus() as i32),
                &(block.header.nonce as i32),
                &(block.header.difficulty() as i64),
            ],
        )
        .await?
        .get("id"))
}

async fn insert_transaction(
    tx: &tokio_postgres::Transaction<'_>,
    block_id: i64,
    transaction: &Transaction,
) -> Result<i64, tokio_postgres::Error> {
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
                &AsRef::<[u8]>::as_ref(&transaction.wtxid().as_raw_hash()),
                &block_id,
                &transaction.version,
                &(transaction.lock_time.to_consensus_u32() as i32),
                &(transaction.weight().to_wu() as i64),
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
) -> Result<(), tokio_postgres::Error> {
    let query = "
        INSERT INTO transaction_inputs
        (transaction_id, index, sequence, witness, script, previous_output_transaction, previous_output_index)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT DO NOTHING
    ";

    tx.execute(
        query,
        &[
            &transaction_id,
            &index,
            &(transaction_input.sequence.to_consensus_u32() as i64),
            &transaction_input.witness.to_vec(),
            &transaction_input.script_sig.as_bytes(),
            &AsRef::<[u8]>::as_ref(&transaction_input.previous_output.txid.as_raw_hash()),
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
) -> Result<(), tokio_postgres::Error> {
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
                .map(|v| v.to_string())
                .ok(),
        ],
    )
    .await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Logging verbosity
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,
    #[arg(short, long, value_parser = Config::from_toml_path)]
    pub config: Config,
    /// Block height to start at
    #[arg(short, long)]
    pub start: u64,
    /// Channel buffer between grab & push to db
    #[arg(short, long)]
    pub buffer: usize,
    /// Amount of concurrent requests to open to bitcoin rpc
    #[arg(short, long)]
    pub fetch_concurrent: usize,
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
