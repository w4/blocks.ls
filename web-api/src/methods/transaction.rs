use crate::database::transactions::{fetch_latest_transactions, fetch_transaction_by_hash};
use crate::{
    database::transactions::fetch_transactions_for_address, methods::block::Transaction, Database,
};
use axum::extract::Query;
use axum::{extract::Path, Extension, Json};
use futures::StreamExt;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    limit: u32,
}

pub async fn list(
    Extension(database): Extension<Database>,
    Query(query): Query<ListQuery>,
) -> Json<Vec<Transaction>> {
    let database = database.get().await.unwrap();

    let limit = std::cmp::min(20, std::cmp::max(5, query.limit));

    let transactions = fetch_latest_transactions(&database, limit.into())
        .await
        .unwrap();

    Json(transactions.into_iter().map(Into::into).collect())
}

pub async fn handle(
    Extension(database): Extension<Database>,
    Path(hash): Path<String>,
) -> Json<Transaction> {
    let mut hash = hex::decode(&hash).unwrap();
    hash.reverse();

    let database = database.get().await.unwrap();
    let transaction = fetch_transaction_by_hash(&database, &hash)
        .await
        .unwrap()
        .unwrap();

    Json(transaction.into())
}
