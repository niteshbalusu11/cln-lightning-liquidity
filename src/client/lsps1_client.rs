use std::{path::Path, sync::Arc};

use anyhow::bail;
use cln_plugin::{Error, Plugin};
use cln_rpc::ClnRpc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    client::{get_info::Lsps1GetInfo, send_order::Lsps1SendOrder},
    constants::PluginMethodState,
    PluginState,
};

use super::get_order::Lsps1GetOrder;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BuyRequestTypes {
    Help,
    Buy,
    Dryrun,
    GetInfo,
    GetOrder,
}

fn str_to_buy_request_type(s: &str) -> Option<BuyRequestTypes> {
    match s.to_lowercase().as_str() {
        // Ensure matching is case-insensitive
        "help" => Some(BuyRequestTypes::Help),
        "buy" => Some(BuyRequestTypes::Buy),
        "getinfo" => Some(BuyRequestTypes::GetInfo),
        "getorder" => Some(BuyRequestTypes::GetOrder),
        _ => None,
    }
}

pub async fn lsps1_client(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let conf = p.configuration();
    let socket_path = Path::new(&conf.lightning_dir).join(&conf.rpc_file);
    let client = ClnRpc::new(socket_path).await?;

    // Set the method to blank
    // Its later updated based on the method
    let state_ref = p.state().clone();

    let mut method = state_ref.method.lock().await;

    *method = PluginMethodState::None;
    std::mem::drop(method);

    match v["request"].as_str().and_then(str_to_buy_request_type) {
        Some(BuyRequestTypes::Help) => {
            return Ok(json!({
                "cli_params": {
                    "method": "Method can be one of the following: (help, buy, getinfo, getorder)",
                    "amount": "<number> enter the channel size you want to buy",
                    "blocks": "<number> enter the number of blocks you want to wait for the channel to be confirmed",
                    "type": "<private/public> the type of channel you want to buy",
                    "orderid": "<orderid> returns the status of the order",
                    "uri": "<uri> pubkey@host:port",
                }
            }));
        }
        Some(BuyRequestTypes::Buy) => {
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

            let is_public_channel = match v["type"].as_str() {
                Some(t) => {
                    if t.to_lowercase() == "private" {
                        false
                    } else if t.to_lowercase() == "public" {
                        true
                    } else {
                        bail!("Invalid channel type")
                    }
                }
                None => {
                    bail!("Invalid type")
                }
            };

            Lsps1SendOrder {
                amount,
                blocks,
                client,
                is_public_channel,
                plugin: p,
                uri: uri_str.to_string(),
            }
            .send_order()
            .await?;

            return Ok(json!({
                "result": "success"
            }));
        }
        Some(BuyRequestTypes::GetInfo) => {
            let uri_str = match v["uri"].as_str() {
                Some(uri) => uri,
                None => {
                    bail!("Invalid URI")
                }
            };
            Lsps1GetInfo {
                client,
                uri: uri_str.to_string(),
                plugin: p,
            }
            .get_info()
            .await?;

            return Ok(json!({
                "result": "success"
            }));
        }
        Some(BuyRequestTypes::GetOrder) => {
            let order_id = match v["orderid"].as_str() {
                Some(order_id) => order_id,
                None => {
                    bail!("Invalid orderid")
                }
            };

            let uri_str = match v["uri"].as_str() {
                Some(uri) => uri,
                None => {
                    bail!("Invalid URI")
                }
            };

            Lsps1GetOrder {
                client,
                uri: uri_str.to_string(),
                order_id: order_id.to_string(),
                plugin: p,
            }
            .get_order()
            .await?;

            return Ok(json!({
                "result": "success"
            }));
        }
        _ => {
            return Ok(json!({
                "result": "error",
                "message": "Invalid request"
            }));
        }
    }
}
