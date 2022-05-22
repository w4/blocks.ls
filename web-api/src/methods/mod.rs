use axum::routing::get;
use axum::Router;

mod address;
mod block;
mod height;

pub fn router() -> Router {
    Router::new()
        .route("/height", get(height::handle))
        .route("/block", get(block::list))
        .route("/block/:height", get(block::handle))
        .route("/address/:address", get(address::handle))
}
