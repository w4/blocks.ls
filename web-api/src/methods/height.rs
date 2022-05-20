use crate::Database;
use axum::Extension;

pub async fn handle(Extension(database): Extension<Database>) -> String {
    let database = database.get().await.unwrap();
    let height = crate::database::blocks::fetch_height(&database)
        .await
        .unwrap();

    height.to_string()
}
