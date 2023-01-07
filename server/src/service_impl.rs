use std::time::Duration;

use log::info;
use rpc::World;
use tarpc::context;
use tokio::time::{sleep_until, Instant};

#[derive(Clone)]
pub struct WorldImpl {}

#[tarpc::server]
#[async_trait::async_trait]
impl World for WorldImpl {
    async fn ping(self, _: context::Context) -> Result<String, String> {
        info!("Ping Called.. responding with Pong!");
        Ok("Pong".into())
    }
    async fn echo(self, _: context::Context, value: String) -> Result<String, String> {
        info!("Echo Called.. responding with {}!", value);
        Ok(value)
    }
    async fn delay(self, _:context::Context, duration: u64) -> Result<String, String> {
        info!("Delayed called!");
        sleep_until(Instant::now()+ Duration::from_secs(duration)).await;
        info!("Delay ended!");
        Ok(format!("Delayed for {} seconds", duration))
    }
}
