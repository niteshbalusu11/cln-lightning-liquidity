use serde::{Deserialize, Serialize};

pub const MESSAGE_TYPE: u16 = 37913u16;

pub const LSPS1_GET_INFO_METHOD: &str = "lsps1.get_info";
pub const LSPS1_CREATE_ORDER_METHOD: &str = "lsps1.create_order";
pub const LSPS1_GET_ORDER_METHOD: &str = "lsps1.get_order";
pub const LSPS1_MAX_FEE_PAID: u32 = 100000;

#[derive(Serialize, Deserialize)]
pub struct GetInfoJsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInfoJsonRpcResponse {
    pub id: String,
    pub jsonrpc: String,
    pub result: GetInfoJsonRpcResponseResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInfoJsonRpcResponseResult {
    pub options: GetInfoJsonRpcResponseOptions,
    pub website: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetInfoJsonRpcResponseOptions {
    pub min_channel_confirmations: u32,
    pub min_onchain_payment_confirmations: Option<u32>,
    pub supports_zero_channel_reserve: bool,
    pub min_onchain_payment_size_sat: Option<u32>,
    pub max_channel_expiry_blocks: u32,
    pub min_initial_client_balance_sat: String,
    pub max_initial_client_balance_sat: String,
    pub min_initial_lsp_balance_sat: String,
    pub max_initial_lsp_balance_sat: String,
    pub min_channel_balance_sat: String,
    pub max_channel_balance_sat: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderJsonRpcRequest {
    pub id: String,
    pub jsonrpc: String,
    pub method: String,
    pub params: CreateOrderJsonRpcRequestParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderJsonRpcRequestParams {
    pub lsp_balance_sat: String,
    pub client_balance_sat: String,
    pub confirms_within_blocks: u32,
    pub channel_expiry_blocks: u32,
    pub token: String,
    pub refund_onchain_address: String,
    pub announce_channel: bool,
}

pub const LSPS1_CREATE_ORDER_CLIENT_SAT_BALANCE: &str = "0";
pub const LSPS1_CREATE_ORDER_CHANNEL_EXPIRY_BLOCKS: u32 = 13000;
pub const LSPS1_CREATE_ORDER_TOKEN: &str = "";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderState {
    Created,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentState {
    ExpectPayment,
    Hold,
    Paid,
    Refunded,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrderJsonRpcResponse {
    pub id: String,
    pub jsonrpc: String,
    pub result: CreateOrderJsonRpcResponseResult,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrderJsonRpcResponseResult {
    pub order_id: String,
    pub lsp_balance_sat: String,
    pub client_balance_sat: String,
    pub confirms_within_blocks: u32,
    pub channel_expiry_blocks: u32,
    pub token: String,
    pub created_at: String,
    pub expires_at: String,
    pub announce_channel: bool,
    pub order_state: OrderState,
    pub payment: CreateOrderJsonRpcResponsePayment,
    pub channel: Option<CreateOrderJsonRpcResponseChannel>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrderJsonRpcResponsePayment {
    pub state: PaymentState,
    pub fee_total_sat: String,
    pub order_total_sat: String,
    pub lightning_invoice: String,
    pub onchain_address: Option<String>,
    pub min_onchain_payment_confirmations: Option<u32>,
    pub min_fee_for_0conf: Option<u32>,
    pub onchain_payment: Option<CreateOrderJsonRpcResponsePaymentOnchainPayment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrderJsonRpcResponsePaymentOnchainPayment {
    pub outpoint: String,
    pub sat: String,
    pub confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateOrderJsonRpcResponseChannel {
    pub funded_at: String,
    pub funding_outpoint: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrderJsonRpcRequest {
    pub id: String,
    pub jsonrpc: String,
    pub method: String,
    pub params: GetOrderJsonRpcRequestParams,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetOrderJsonRpcRequestParams {
    pub order_id: String,
}
