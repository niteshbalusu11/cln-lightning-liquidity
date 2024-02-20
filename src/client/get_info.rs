use cln_rpc::{
    model::requests::{ConnectRequest, SendcustommsgRequest},
    primitives::PublicKey,
    ClnRpc, Request,
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::constants::{LSPS1_GET_INFO_METHOD, MESSAGE_TYPE};

use super::utils::{decode_uri, make_id};

pub struct GetInfo {
    pub client: ClnRpc,
    pub uri: String,
}

#[derive(Serialize, Deserialize)]
struct GetInfoJsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: String,
}

impl GetInfo {
    // This method now belongs to an instance of GetInfo and uses its data
    pub async fn lsps1_client(&mut self) -> anyhow::Result<()> {
        // Use the struct's fields directly
        let uri = decode_uri(&self.uri)?;

        info!("Connecting to {:?}", uri);

        Self::connect(&mut self.client, &uri.pubkey, &uri.host, &uri.port).await?;

        info!("Connected to {}", uri.pubkey);

        Self::send_get_info_message(&mut self.client, &uri.pubkey).await?;

        Ok(())
    }

    async fn connect(
        client: &mut ClnRpc,
        pubkey: &PublicKey,
        host: &str,
        port: &u16,
    ) -> anyhow::Result<()> {
        match client
            .call(Request::Connect(ConnectRequest {
                id: pubkey.to_string(),
                host: Some(host.to_string()),
                port: Some(*port),
            }))
            .await
        {
            Ok(_) => {
                return Ok(());
            }

            // Ignore errors
            Err(_) => {
                return Ok(());
            }
        }
    }

    async fn send_get_info_message(client: &mut ClnRpc, pubkey: &PublicKey) -> anyhow::Result<()> {
        let request = GetInfoJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: LSPS1_GET_INFO_METHOD.to_string(),
            params: serde_json::json!({}), // Creating an empty object for params.
            id: make_id(),
        };

        let json_request = serde_json::to_string(&request)?;

        // Encode the JSON request to hexadecimal
        let hex_json_request = hex::encode(json_request);

        // Convert the message type 37913 to a 2-byte hexadecimal string
        let message_type_prefix = MESSAGE_TYPE.to_be_bytes(); // Convert to big-endian bytes
        let hex_message_type_prefix = hex::encode(message_type_prefix);

        // Prepend the message type prefix to the hex-encoded JSON request
        let full_hex_message = format!("{}{}", hex_message_type_prefix, hex_json_request);

        client
            .call(Request::SendCustomMsg(SendcustommsgRequest {
                msg: full_hex_message,
                node_id: *pubkey,
            }))
            .await?;

        Ok(())
    }
}
