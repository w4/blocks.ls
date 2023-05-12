use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub bitcoin_rpc: BitcoinRpc,
    pub database: DatabaseConfig,
}

impl Config {
    pub fn from_toml_path(path: &str) -> Result<Config, std::io::Error> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct BitcoinRpc {
    pub address: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub database: String,
}
