use std::path::Path;

mod client;
mod constants;
use anyhow::{bail, Ok};
use cln_plugin::{Builder, Error, Plugin};
use cln_rpc::ClnRpc;
use log::warn;
use serde_json::json;
use tokio::io::{stdin, stdout};

use crate::client::get_info::GetInfo;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let state = ();

    if let Some(plugin) = Builder::new(stdin(), stdout())
        .dynamic()
        .rpcmethod(
            "buy-inbound-channel",
            "Buy an inbound channel from other peers",
            lsps1_client,
        )
        .hook("custommsg", subscribe_to_custom_message)
        .start(state)
        .await?
    {
        plugin.join().await
    } else {
        Ok(())
    }
}

async fn lsps1_client(p: Plugin<()>, v: serde_json::Value) -> Result<serde_json::Value, Error> {
    log::info!("Received request: {:?}", v);
    let conf = p.configuration();
    let socket_path = Path::new(&conf.lightning_dir).join(&conf.rpc_file);
    let client = ClnRpc::new(socket_path).await?;

    if v["request"].as_str() == Some("getinfo") {
        let uri_str = match v["uri"].as_str() {
            Some(uri) => uri,
            None => {
                bail!("Invalid URI")
            }
        };
        GetInfo {
            client,
            uri: uri_str.to_string(),
        }
        .lsps1_client()
        .await?;

        return Ok(json!({
            "result": "success"
        }));
    }

    Ok(json!({
        "cli_params": {
            "amount": "<number> enter the channel size you want to buy",
            "blocks": "<number> enter the number of blocks you want to wait for the channel to be confirmed",
            "getinfo": "returns info from nodes selling channels",
            "order_status": "<order_id> returns the status of the order",
            "uri": "<uri> pubkey@host:port",
        }
    }))
}

async fn subscribe_to_custom_message(
    _p: Plugin<()>,
    v: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    log::info!("Received custom message: {:?}", v);

    // Attempt to extract "payload"
    let payload = v.get("payload").and_then(|v| v.as_str());

    // Attempt to extract "peer_id"
    let peer_id = v.get("peer_id").and_then(|v| v.as_str());

    // Log issues instead of bailing
    if payload.is_none() {
        warn!("'payload' is missing or invalid");
    } else if !payload.unwrap().chars().all(|c| c.is_digit(16)) {
        // If payload exists but is not a valid hex string
        warn!("'payload' is not a valid hex string");
    }

    if peer_id.is_none() {
        warn!("'peer_id' is missing or invalid");
    }

    // Continue with your intended response regardless of issues
    Ok(json!({
        "result": "continue"
    }))
}
