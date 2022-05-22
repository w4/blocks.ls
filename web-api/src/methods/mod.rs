use axum::routing::get;
use axum::Router;

mod address;
mod block;
mod height;
mod transaction;

pub fn router() -> Router {
    Router::new()
        .route("/height", get(height::handle))
        .route("/block", get(block::list))
        .route("/block/:height", get(block::handle))
        .route("/address/:address", get(address::handle))
        .route("/tx", get(transaction::list))
        .route("/tx/:hash", get(transaction::handle))
}
