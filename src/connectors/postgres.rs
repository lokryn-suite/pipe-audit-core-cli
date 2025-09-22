// src/connectors/postgres.rs
use super::Connector;
use polars::prelude::*;
use postgres::{Client, NoTls};

pub struct PostgresConnector {
    pub conn_str: String,
    pub query: String,
}

impl Connector for PostgresConnector {
    fn name(&self) -> &str {
        "postgres"
    }

    fn fetch(&self) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut client = Client::connect(&self.conn_str, NoTls)?;
        let rows = client.query(&self.query, &[])?;

        // TODO: map rows into a Polars DataFrame
        
        Ok(DataFrame::default())
    }
}
