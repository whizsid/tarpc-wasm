use futures::{StreamExt, TryStreamExt};
use log::info;
use rpc::World;
use service_impl::WorldImpl;
use tarpc::{
    serde::{Deserialize, Serialize},
    server::Channel,
};
use web::bind;

mod service_impl;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("First Message");

    let server = build_server().await.expect("Failed to get server channel");
    let stream = server.map_ok(move |x| {
        info!("Mapping the client session");
        let server = tarpc::server::BaseChannel::with_defaults(x);
        let service = WorldImpl {};
        info!("Spawning client channel");
        tokio::spawn(server.execute(service.serve()))
    });

    //TODO: Will likely need a way to kill the connection. Need to figure that out.
    let handle = tokio::spawn(stream.for_each(|_| async {}));
    handle.await.unwrap();
    Ok(())
}

async fn build_server<Item, SinkItem>(
) -> Option<impl TryStreamExt<Ok = impl tarpc::Transport<SinkItem, Item>, Error = std::io::Error>>
where
    Item: for<'de> Deserialize<'de> + Unpin,
    SinkItem: Serialize + Unpin,
{
    Some(
        bind(tokio_serde::formats::Json::<Item, SinkItem>::default)
            .await
            .unwrap(),
    )
}
