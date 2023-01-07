use async_trait::async_trait;
use tarpc::service;

#[service]
#[async_trait]
pub trait World {
    async fn ping() -> Result<String, String>;
    async fn echo(value: String) -> Result<String, String>;
    async fn delay(duration: u64) -> Result<String, String>;
}
