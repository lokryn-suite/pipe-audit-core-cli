use polars::prelude::*;

pub trait Connector {
    fn name(&self) -> &str;
    fn fetch(&self) -> Result<DataFrame, Box<dyn std::error::Error>>;
}
