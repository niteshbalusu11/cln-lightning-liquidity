use std::{path::Path, sync::Arc};

use cln_plugin::{Error, Plugin};
use cln_rpc::ClnRpc;
use serde_json::json;

use crate::{
    client::validate_and_pay::Lsps1ValidateAndPay,
    constants::{
        CreateOrderJsonRpcResponse, GetInfoJsonRpcResponse, PluginMethodState, MESSAGE_TYPE,
    },
    PluginState,
};

pub async fn subscribe_to_custom_message(
    p: Plugin<Arc<PluginState>>,
    v: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let state_ref = p.state().clone();

    // Now, you can lock the mutex asynchronously
    let data = state_ref.data.lock().await;
    let method = state_ref.method.lock().await;

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
        _ => {
            return Ok(json!({ "result": "continue" }));
        }
    };

    // Ensure there are at least 2 bytes for the message type
    if bytes.len() < 2 {
        return Ok(json!({ "result": "continue" }));
    }

    // Extract the message type from the first 2 bytes
    let message_type = u16::from_be_bytes([bytes[0], bytes[1]]);

    if message_type != MESSAGE_TYPE {
        return Ok(json!({ "result": "continue" }));
    }

    let conf = p.configuration();
    let socket_path = Path::new(&conf.lightning_dir).join(&conf.rpc_file);
    let client = ClnRpc::new(socket_path).await?;

    // Extract the JSON payload starting from the 3rd byte
    let json_bytes = &bytes[2..];

    // Get info response method
    match serde_json::from_slice::<GetInfoJsonRpcResponse>(json_bytes) {
        Ok(json_payload) => {
            log::info!(
                "GetInfo Response: {:?}",
                serde_json::to_string_pretty(&json_payload)
            )
        }
        _ => {}
    };

    // Create order/get order response method
    match serde_json::from_slice::<CreateOrderJsonRpcResponse>(json_bytes) {
        Ok(json_payload) => match *method {
            PluginMethodState::GetOrder => {
                log::info!(
                    "GetOrder Response: {:?}",
                    serde_json::to_string_pretty(&json_payload)
                );
            }
            PluginMethodState::SendOrder => {
                log::info!(
                    "CreateOrder Response: {:?}",
                    serde_json::to_string_pretty(&json_payload)
                );

                let get_order = data.get(&json_payload.id);

                if let Some(order) = get_order {
                    let res = Lsps1ValidateAndPay {
                        order: order.to_string(),
                        client,
                        order_response_payload: json_payload,
                    }
                    .validate_and_pay()
                    .await;

                    match res {
                        Ok(_) => {
                            log::info!("Order validated and paid");
                        }
                        Err(e) => {
                            log::error!("Order validation and payment failed: {}", e);
                        }
                    }
                }
            }
            _ => {}
        },
        _ => {}
    };

    return Ok(json!({ "result": "continue" }));
}
