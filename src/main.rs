use std::path::Path;

mod client;
mod constants;
use anyhow::bail;
use cln_plugin::{Builder, Error, Plugin};
use cln_rpc::ClnRpc;
use constants::MESSAGE_TYPE;
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

    if v["request"].as_str() == Some("help") {
        return Ok(json!({
            "cli_params": {
                "amount": "<number> enter the channel size you want to buy",
                "blocks": "<number> enter the number of blocks you want to wait for the channel to be confirmed",
                "getinfo": "returns info from nodes selling channels",
                "order_status": "<order_id> returns the status of the order",
                "uri": "<uri> pubkey@host:port",
            }
        }));
    }

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

    if v["request"].as_str() == Some("buy") {
        let amount = match v["amount"].as_u64() {
            Some(amount) => amount,
            None => {
                bail!("Invalid amount")
            }
        };

        let blocks = match v["blocks"].as_u64() {
            Some(blocks) => blocks,
            None => {
                bail!("Invalid blocks")
            }
        };

        let uri_str = match v["uri"].as_str() {
            Some(uri) => uri,
            None => {
                bail!("Invalid URI")
            }
        };

        // client::buy::buy_channel(client, amount, blocks, uri_str).await?;

        return Ok(json!({
            "result": "success"
        }));
    }

    return Ok(json!({
        "result": "error",
        "message": "Invalid request"
    }));
}

async fn subscribe_to_custom_message(
    p: Plugin<()>,
    v: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    // Attempt to extract "payload"
    let payload_hex = match v.get("payload").and_then(|v| v.as_str()) {
        Some(payload_hex) => payload_hex,
        None => {
            log::warn!("No payload found in custom message");
            return Ok(json!({ "result": "continue" }));
        }
    };

    let bytes = match hex::decode(payload_hex) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::warn!("Failed to decode hex: {}", e);
            return Ok(json!({ "result": "continue" }));
        }
    };

    // Ensure there are at least 2 bytes for the message type
    if bytes.len() < 2 {
        log::warn!("Payload is too short to contain a message type");
        return Ok(json!({ "result": "continue" }));
    }

    // Extract the message type from the first 2 bytes
    let message_type = u16::from_be_bytes([bytes[0], bytes[1]]);

    if message_type != MESSAGE_TYPE {
        log::info!("Received message with unexpected type: {}", message_type);
        return Ok(json!({ "result": "continue" }));
    }

    // Extract the JSON payload starting from the 3rd byte
    let json_bytes = &bytes[2..];

    let json_payload = match serde_json::from_slice::<serde_json::Value>(json_bytes) {
        Ok(json_payload) => {
            log::info!("Decoded JSON payload: {:?}", json_payload);
            json_payload
        }
        Err(e) => {
            log::warn!("Failed to decode JSON payload: {}", e);
            return Ok(json!({ "result": "continue" }));
        }
    };

    let conf = p.configuration();
    let socket_path = Path::new(&conf.lightning_dir).join(&conf.rpc_file);
    let client = ClnRpc::new(socket_path).await?;

    // Attempt to extract "peer_id"
    // let peer_id = v.get("peer_id").and_then(|v| v.as_str());

    // Continue with your intended response regardless of issues
    Ok(json!({
        "result": "continue"
    }))
}
