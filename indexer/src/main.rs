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
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_max_level(args.logging_level())
        .init();

    let bitcoin_rpc = Client::new(
        args.config.bitcoin_rpc.address,
        Auth::UserPass(
            args.config.bitcoin_rpc.username,
            args.config.bitcoin_rpc.password,
        ),
    )
    .await?;

    let database = Database::new(args.config.database)?;
    database::migrations::runner()
        .run_async(&mut **database.get().await?)
        .await?;

    let height = bitcoin_rpc.get_block_count().await?;
    eprintln!("Current block height: {}", height);

    process_block(&bitcoin_rpc, database.get().await.unwrap().as_mut(), 736822).await?;

    Ok(())
}

pub async fn process_block(
    bitcoin_rpc: &Client,
    database: &mut tokio_postgres::Client,
    block: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let hash = bitcoin_rpc.get_block_hash(block).await?;
    let block = bitcoin_rpc.get_block(&hash).await?;

    let tx = database.transaction().await?;

    let block_id: i64 = insert_block(&tx, &block, &hash).await?;
    eprintln!("block_id = {}", block_id);

    {
        let tx = &tx;
        futures::future::try_join_all({
            block.txdata.iter().map(|transaction| async move {
                let transaction_id = insert_transaction(tx, block_id, transaction).await?;

                futures::future::try_join(
                    futures::future::try_join_all(transaction.input.iter().map(|transaction_in| {
                        insert_transaction_input(tx, transaction_id, transaction_in)
                    })),
                    futures::future::try_join_all(transaction.output.iter().enumerate().map(
                        |(index, transaction_out)| {
                            insert_transaction_output(
                                tx,
                                index as i64,
                                transaction_id,
                                transaction_out,
                            )
                        },
                    )),
                )
                .await?;

                Ok::<_, Box<dyn std::error::Error>>(())
            })
        })
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn insert_block(
    tx: &tokio_postgres::Transaction<'_>,
    block: &Block,
    block_hash: &BlockHash,
) -> Result<i64, Box<dyn std::error::Error>> {
    let query = "
        INSERT INTO blocks
        (hash, height, version, size, merkle_root_hash, timestamp, bits, nonce, difficulty)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
    ";

    // TODO: previous_block_id
    Ok(tx
        .query_one(
            query,
            &[
                &block_hash.to_vec(),
                &(block.bip34_block_height().unwrap_or(0) as i64),
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
    transaction_id: i64,
    transaction_input: &TxIn,
) -> Result<(), Box<dyn std::error::Error>> {
    let previous_output = select_transaction_output(
        tx,
        &transaction_input.previous_output.txid.to_vec(),
        transaction_input.previous_output.vout as i64,
    )
    .await?;

    let query = "
        INSERT INTO transaction_outputs
        (transaction_id, previous_output, script, address)
        VALUES ($1, $2, $3, $4)
    ";

    tx.execute(
        query,
        &[
            &transaction_id,
            &previous_output.as_ref().map(|(id, _)| *id),
            &transaction_input.script_sig.as_bytes(),
            &previous_output.map(|(_, address)| address),
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

// TODO: this is a _very_ efficient query involving just two index scans, right now we're inserting
//  it alongside transaction_outputs, but we need sequential inserts for that to work. maybe we can
//  just call this query on-demand? or figure out a way to sequentialise inserts - that's quite risky
//  to our insert speed though.
async fn select_transaction_output(
    tx: &tokio_postgres::Transaction<'_>,
    transaction_hash: &[u8],
    transaction_index: i64,
) -> Result<Option<(i64, String)>, Box<dyn std::error::Error>> {
    let query = "
        SELECT transaction_outputs.id AS output_id, address
        FROM transactions
        INNER JOIN transaction_outputs
            ON transactions.id = transaction_outputs.transaction_id
        WHERE transactions.hash = $1
            AND transaction_outputs.index = $2
    ";

    let row = tx
        .query_opt(query, &[&transaction_hash, &transaction_index])
        .await?;

    Ok(row.map(|v| (v.get("output_id"), v.get("address"))))
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
