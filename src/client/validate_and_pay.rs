use anyhow::bail;
use cln_rpc::{
    model::requests::{DecodepayRequest, PayRequest},
    ClnRpc, Request, Response,
};

use crate::constants::{
    CreateOrderJsonRpcRequest, CreateOrderJsonRpcResponse, OrderState, PaymentState,
    LSPS1_MAX_FEE_PAID,
};

pub struct Lsps1ValidateAndPay {
    pub order: String,
    pub client: ClnRpc,
    pub order_response_payload: CreateOrderJsonRpcResponse,
}

impl Lsps1ValidateAndPay {
    pub async fn validate_and_pay(&mut self) -> anyhow::Result<()> {
        let order_request = serde_json::from_str::<CreateOrderJsonRpcRequest>(&self.order)?;
        let payload = &self.order_response_payload;

        if order_request.id != payload.id {
            bail!("Order id mismatch");
        }

        if order_request.params.channel_expiry_blocks != payload.result.channel_expiry_blocks {
            bail!("Channel expiry blocks mismatch");
        }

        if order_request.params.confirms_within_blocks != payload.result.confirms_within_blocks {
            bail!("Confirms within blocks mismatch");
        }

        if order_request.params.lsp_balance_sat != payload.result.lsp_balance_sat {
            bail!("LSP balance mismatch");
        }

        if payload.result.order_state != OrderState::Created {
            bail!("Order state is not created");
        }

        if payload.result.payment.state != PaymentState::ExpectPayment {
            bail!("Payment state is not expect payment");
        }

        // Make sure you're not paying crazy fees
        // convert fee_total_sat to u32
        let fee_total_sat: u32 = payload.result.payment.fee_total_sat.parse()?;

        if fee_total_sat > LSPS1_MAX_FEE_PAID {
            bail!("Fee is too high");
        }

        let order_total_sat: u64 = payload.result.payment.order_total_sat.parse()?;

        // We don't support push amounts
        // So order total and fee total are equal
        if order_total_sat != fee_total_sat as u64 {
            bail!("Order total and fee total mismatch");
        }

        // Decode payment request
        let res = self
            .client
            .call(Request::DecodePay(DecodepayRequest {
                bolt11: payload.result.payment.lightning_invoice.clone(),
                description: None,
            }))
            .await?;

        let decoded = match res {
            Response::DecodePay(n) => n,
            _ => {
                bail!("Invalid response");
            }
        };

        if let Some(invoice_amount) = decoded.amount_msat {
            if invoice_amount.msat() != order_total_sat * 1000 {
                bail!("Invoice amount mismatch");
            }
        } else {
            bail!("No invoice amount");
        }

        // Pay in the invoice

        let res = self
            .client
            .call(Request::Pay(PayRequest {
                bolt11: payload.result.payment.lightning_invoice.clone(),
                amount_msat: None,
                maxfeepercent: None,
                description: None,
                exclude: None,
                exemptfee: None,
                label: None,
                localinvreqid: None,
                maxdelay: None,
                maxfee: None,
                retry_for: None,
                riskfactor: None,
            }))
            .await?;

        match res {
            Response::Pay(n) => {
                log::info!("Payment response: {:?}", n);
            }
            _ => {
                bail!("Invalid response");
            }
        };

        Ok(())
    }
}
