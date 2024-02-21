use std::sync::Arc;

use cln_plugin::Plugin;
use cln_rpc::{
    model::requests::{ConnectRequest, SendcustommsgRequest},
    primitives::PublicKey,
    ClnRpc, Request,
};

use crate::{
    constants::{
        GetOrderJsonRpcRequest, GetOrderJsonRpcRequestParams, PluginMethodState,
        LSPS1_GET_ORDER_METHOD, MESSAGE_TYPE,
    },
    PluginState,
};

use super::utils::{decode_uri, make_id};

pub struct Lsps1GetOrder {
    pub client: ClnRpc,
    pub uri: String,
    pub order_id: String,
    pub plugin: Plugin<Arc<PluginState>>,
}

impl<'a> Lsps1GetOrder {
    pub async fn get_order(&mut self) -> anyhow::Result<()> {
        log::info!("inside getorder {}", self.uri);

        let state_ref = self.plugin.state();

        let mut method = state_ref.method.lock().await;
        // Set the state to get order so that
        // The subscribtion side knows what to do
        *method = PluginMethodState::GetOrder;

        let uri = decode_uri(&self.uri)?;

        Self::connect(&mut self.client, &uri.pubkey, &uri.host, &uri.port).await?;

        Self::send_get_order_message(&mut self.client, &uri.pubkey, &self.order_id).await?;

        Ok(())
    }

    async fn connect(
        client: &mut ClnRpc,
        pubkey: &PublicKey,
        host: &str,
        port: &u16,
    ) -> anyhow::Result<()> {
        // Ignore errors for connect requests
        let _ = client
            .call(Request::Connect(ConnectRequest {
                id: pubkey.to_string(),
                host: Some(host.to_string()),
                port: Some(*port),
            }))
            .await;

        Ok(())
    }

    async fn send_get_order_message(
        client: &mut ClnRpc,
        pubkey: &PublicKey,
        order_id: &str,
    ) -> anyhow::Result<()> {
        let params = GetOrderJsonRpcRequestParams {
            order_id: order_id.to_string(),
        };

        let request = GetOrderJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: LSPS1_GET_ORDER_METHOD.to_string(),
            params,
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
