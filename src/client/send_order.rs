use std::sync::Arc;

use cln_plugin::Plugin;
use cln_rpc::{
    model::requests::{ConnectRequest, NewaddrRequest, SendcustommsgRequest},
    primitives::PublicKey,
    ClnRpc, Request, Response,
};

use crate::{
    constants::{
        CreateOrderJsonRpcRequest, CreateOrderJsonRpcRequestParams,
        LSPS1_CREATE_ORDER_CHANNEL_EXPIRY_BLOCKS, LSPS1_CREATE_ORDER_CLIENT_SAT_BALANCE,
        LSPS1_CREATE_ORDER_METHOD, LSPS1_CREATE_ORDER_TOKEN, MESSAGE_TYPE,
    },
    PluginState,
};

use super::utils::{decode_uri, make_id};

pub struct Lsps1SendOrder {
    pub client: ClnRpc,
    pub amount: u64,
    pub blocks: u64,
    pub uri: String,
    pub plugin: Plugin<Arc<PluginState>>,
}

impl Lsps1SendOrder {
    pub async fn send_order(&mut self) -> anyhow::Result<()> {
        let state_ref = self.plugin.state().clone();

        let mut method = state_ref.method.lock().await;
        // Set the state to get info so that
        // The subscribtion side knows what to do
        method.insert("method".to_string(), LSPS1_CREATE_ORDER_METHOD.to_string());

        let uri = decode_uri(&self.uri)?;

        Self::connect(&mut self.client, &uri.pubkey, &uri.host, &uri.port).await?;

        let refund_address = Self::make_refund_address(&mut self.client).await?;

        Self::send_get_order_message(
            &mut self.client,
            self.amount,
            self.blocks as u32,
            &uri.pubkey,
            &refund_address,
            &self.plugin,
        )
        .await?;

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

    async fn make_refund_address(client: &mut ClnRpc) -> anyhow::Result<String> {
        let res = client
            .call(Request::NewAddr(NewaddrRequest {
                addresstype: Some(cln_rpc::model::requests::NewaddrAddresstype::BECH32),
            }))
            .await?;

        let address = match res {
            Response::NewAddr(a) => a.bech32.expect("No bech32 address generated"),
            _ => {
                return Err(anyhow::anyhow!("Invalid response"));
            }
        };

        Ok(address)
    }

    async fn send_get_order_message(
        client: &mut ClnRpc,
        amount: u64,
        blocks: u32,
        pubkey: &PublicKey,
        refund_address: &str,
        plugin: &Plugin<Arc<PluginState>>,
    ) -> anyhow::Result<()> {
        let id = make_id();

        let params = CreateOrderJsonRpcRequestParams {
            lsp_balance_sat: amount.to_string(),
            client_balance_sat: LSPS1_CREATE_ORDER_CLIENT_SAT_BALANCE.to_string(),
            confirms_within_blocks: blocks,
            channel_expiry_blocks: LSPS1_CREATE_ORDER_CHANNEL_EXPIRY_BLOCKS,
            token: LSPS1_CREATE_ORDER_TOKEN.to_string(),
            announce_channel: true,
            refund_onchain_address: refund_address.to_string(),
        };

        let request = CreateOrderJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: LSPS1_CREATE_ORDER_METHOD.to_string(),
            params,
            id: id.clone(),
        };

        let json_request = serde_json::to_string(&request)?;

        // Encode the JSON request to hexadecimal
        let hex_json_request = hex::encode(json_request.clone());

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

        let state_ref = plugin.state().clone();

        // Now, you can lock the mutex asynchronously
        let mut data = state_ref.data.lock().await;

        data.insert(id, json_request);

        Ok(())
    }
}
