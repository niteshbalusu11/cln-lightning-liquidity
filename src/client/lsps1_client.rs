use std::path::Path;

use anyhow::bail;
use cln_plugin::{Error, Plugin};
use cln_rpc::ClnRpc;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::client::{get_info::Lsps1GetInfo, send_order::Lsps1SendOrder};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum BuyRequestTypes {
    Help,
    Buy,
    Dryrun,
    GetInfo,
    GetStatus,
}

fn str_to_buy_request_type(s: &str) -> Option<BuyRequestTypes> {
    match s.to_lowercase().as_str() {
        // Ensure matching is case-insensitive
        "help" => Some(BuyRequestTypes::Help),
        "buy" => Some(BuyRequestTypes::Buy),
        "dryrun" => Some(BuyRequestTypes::Dryrun),
        "getinfo" => Some(BuyRequestTypes::GetInfo),
        "getstatus" => Some(BuyRequestTypes::GetStatus),
        _ => None,
    }
}

pub async fn lsps1_client(p: Plugin<()>, v: serde_json::Value) -> Result<serde_json::Value, Error> {
    log::info!("Received request: {:?}", v);
    let conf = p.configuration();
    let socket_path = Path::new(&conf.lightning_dir).join(&conf.rpc_file);
    let client = ClnRpc::new(socket_path).await?;

    match v["request"].as_str().and_then(str_to_buy_request_type) {
        Some(BuyRequestTypes::Help) => {
            return Ok(json!({
                "cli_params": {
                    "method": "Method can be one of the following: (help, buy, dryrun, getinfo, getstatus)",
                    "amount": "<number> enter the channel size you want to buy",
                    "blocks": "<number> enter the number of blocks you want to wait for the channel to be confirmed",
                    "getinfo": "returns info from nodes selling channels",
                    "order_id": "<order_id> returns the status of the order",
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

            Lsps1SendOrder {
                client,
                amount,
                blocks,
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
            }
            .lsps1_client()
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
