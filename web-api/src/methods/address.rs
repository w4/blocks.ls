use crate::{
    database::transactions::fetch_transactions_for_address, methods::block::Transaction, Database,
};
use axum::{extract::Path, Extension, Json};

pub async fn handle(
    Extension(database): Extension<Database>,
    Path(address): Path<String>,
) -> Json<Vec<Transaction>> {
    let database = database.get().await.unwrap();
    let transactions = fetch_transactions_for_address(&database, &address)
        .await
        .unwrap();

    Json(
        transactions
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
    )
}
