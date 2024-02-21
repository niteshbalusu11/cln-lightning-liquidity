use std::{collections::HashMap, sync::Arc};

mod client;
mod constants;
mod subscribe_to_messages;

use client::lsps1_client::lsps1_client;
use cln_plugin::{Builder, Error};
use constants::PluginMethodState;

use tokio::{
    io::{stdin, stdout},
    sync::Mutex,
};

use subscribe_to_messages::subscribe_to_custom_message;

struct PluginState {
    data: Mutex<HashMap<String, String>>,
    method: Mutex<PluginMethodState>,
}

impl PluginState {
    async fn new() -> Result<Self, Error> {
        Ok(Self {
            data: Mutex::new(HashMap::new()),
            method: Mutex::new(PluginMethodState::None),
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let plugin_state = Arc::new(PluginState::new().await?);

    if let Some(plugin) = Builder::new(stdin(), stdout())
        .dynamic()
        .rpcmethod(
            "buy-inbound-channel",
            "Buy an inbound channel from other peers",
            lsps1_client,
        )
        .hook("custommsg", subscribe_to_custom_message)
        .start(plugin_state)
        .await?
    {
        let plug_res = plugin.join().await;

        plug_res
    } else {
        Ok(())
    }
}
