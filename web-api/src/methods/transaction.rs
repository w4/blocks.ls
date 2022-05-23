use crate::database::transactions::{fetch_latest_transactions, fetch_transaction_by_hash};
use crate::{methods::block::Transaction, Database};
use axum::extract::Query;
use axum::{extract::Path, Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    limit: u32,
}

#[derive(Serialize)]
pub struct ListResponseTransaction {
    #[serde(flatten)]
    transaction: Transaction,
    input_total_value: i64,
    output_total_value: i64,
}

pub async fn list(
    Extension(database): Extension<Database>,
    Query(query): Query<ListQuery>,
) -> Json<Vec<ListResponseTransaction>> {
    let database = database.get().await.unwrap();

    let limit = std::cmp::min(20, std::cmp::max(5, query.limit));

    let transactions = fetch_latest_transactions(&database, limit.into())
        .await
        .unwrap();

    Json(
        transactions
            .into_iter()
            .map(|v| ListResponseTransaction {
                transaction: v.transaction.into(),
                input_total_value: i64::try_from(v.input_total_value.mantissa()).unwrap(),
                output_total_value: i64::try_from(v.output_total_value.mantissa()).unwrap(),
            })
            .collect(),
    )
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
