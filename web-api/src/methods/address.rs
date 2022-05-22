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

    Json(transactions.into_iter().map(Into::into).collect())
}
