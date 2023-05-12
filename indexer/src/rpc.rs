use std::sync::Arc;

use base64::Engine;
use bitcoin::{Block, BlockHash};
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    Client,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Clone)]
pub struct BitcoinRpc {
    client: Arc<Client>,
    url: Arc<str>,
}

impl BitcoinRpc {
    pub fn new(config: &crate::config::BitcoinRpc) -> Self {
        let client = Arc::new(
            reqwest::ClientBuilder::new()
                .default_headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        AUTHORIZATION,
                        format!(
                            "Basic {}",
                            base64::engine::general_purpose::STANDARD
                                .encode(format!("{}:{}", config.username, config.password))
                        )
                        .parse()
                        .unwrap(),
                    );
                    headers
                })
                .build()
                .unwrap(),
        );

        Self {
            client,
            url: Arc::from(format!("http://{}", config.address)),
        }
    }

    pub async fn get_block_height(&self) -> u64 {
        self.client
            .post(&*self.url)
            .json(&json!({
                "jsonrpc": "1.0",
                "id": 0,
                "method": "getblockcount",
                "params": []
            }))
            .send()
            .await
            .unwrap()
            .json::<RpcResult<u64>>()
            .await
            .unwrap()
            .result
    }

    pub async fn get_block_hash(&self, height: u64) -> BlockHash {
        self.client
            .post(&*self.url)
            .json(&json!({
                "jsonrpc": "1.0",
                "id": 0,
                "method": "getblockhash",
                "params": [height],
            }))
            .send()
            .await
            .unwrap()
            .json::<RpcResult<BlockHash>>()
            .await
            .unwrap()
            .result
    }

    pub async fn get_block(&self, hash: &BlockHash) -> Block {
        let hash = hash.to_string();

        let res = self
            .client
            .post(&*self.url)
            .json(&json!({
                "jsonrpc": "1.0",
                "id": 0,
                "method": "getblock",
                "params": [hash, 0],
            }))
            .send()
            .await
            .unwrap()
            .json::<RpcResult<String>>()
            .await
            .unwrap()
            .result;

        let bytes: Vec<u8> = bitcoin::hashes::hex::FromHex::from_hex(&res).unwrap();

        bitcoin::consensus::encode::deserialize(&bytes).unwrap()
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct RpcResult<T> {
    result: T,
    error: Option<String>,
    id: u64,
}
