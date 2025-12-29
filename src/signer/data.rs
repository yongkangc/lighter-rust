use crate::{api::order::GroupingType, signer::ffi::ffisigner};

#[derive(Debug)]
pub struct TxInfo {
    pub data: Option<TxInfoData>,
    pub payload: String, // tx_info
}

#[derive(Debug)]
pub struct TxInfoData {
    pub message: String,
    pub signature: String,
}

#[derive(Debug)]
pub enum TxData {
    ChangePubKey(ChangePubKeyData),
    //SwitchApiKey(SwitchApiKeyData), // I don't think it's strictly necessary to have it in. Leaving it out for now.
    CreateOrder(CreateOrderData),
    SignCreateGroupedOrders(SignCreateGroupedOrdersData),
    SignCancelOrder(SignCancelOrderData),
    SignWithdraw(SignWithdrawData),
    SignCreateSubaccount,
    SignCancelAllOrders(SignCancelAllOrdersData),
    SignModifyOrder(SignModifyOrderData),
    SignTransfer(SignTransferData),
    SignCreatePublicPool(SignCreatePublicPoolData),
    SignUpdatePublicPool(SignUpdatePublicPoolData),
    SignMintShares(SignMintSharesData),
    SignBurnShares(SignBurnSharesData),
    SignUpdateLeverage(SignUpdateLeverageData),
    SignUpdateMargin(SignUpdateMarginData),
}

// ------------------ Requests data structs -------------------

#[derive(Debug)]
pub struct ChangePubKeyData {
    pub new_pubk: String,
}

#[derive(Debug)]
pub struct CreateOrderData {
    pub market_index: i32,
    pub client_order_index: i64,
    pub base_amount: i64,
    pub price: i32,
    pub is_ask: bool,
    pub order_type: u8,
    pub time_in_force: u8,
    pub reduce_only: bool,
    pub trigger_price: i32,
    pub order_expiry: i64,
}

#[derive(Debug)]
pub struct SignCreateGroupedOrdersData {
    pub grouping_type: GroupingType,
    pub orders: Vec<ffisigner::CreateOrderTxReq>,
}

#[derive(Debug)]
pub struct SignCancelOrderData {
    pub market_index: i32,
    pub order_index: i64,
}

#[derive(Debug)]
pub struct SignWithdrawData {
    pub asset_index: i16,
    pub route_type: i32,
    pub amount: u64, // unsigned long long in C
}

#[derive(Debug)]
pub struct SignCancelAllOrdersData {
    pub time_in_force: u8,
    pub time: i64,
}

#[derive(Debug)]
pub struct SignModifyOrderData {
    pub market_index: i32,
    pub order_index: i64,
    pub amount: i64,
    pub price: i64,
    pub trigger_price: i64,
}

#[derive(Debug)]
pub struct SignTransferData {
    pub to_account_index: i64,
    pub asset_index: i16,
    pub from_route_type: u8,
    pub to_route_type: u8,
    pub amount: i64,
    pub usdc_fee: i64,
    pub memo: [u8; 32],
}

#[derive(Debug)]
pub struct SignCreatePublicPoolData {
    pub operator_fee: i64,
    pub initial_total_shares: i32, // int in C
    pub min_operator_share_rate: i64,
}

#[derive(Debug)]
pub struct SignUpdatePublicPoolData {
    pub public_pool_index: i64,
    pub status: i32,
    pub operator_fee: i64,
    pub min_operator_share_rate: i32, // int in C
}

#[derive(Debug)]
pub struct SignMintSharesData {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

#[derive(Debug)]
pub struct SignBurnSharesData {
    pub public_pool_index: i64,
    pub share_amount: i64,
}

#[derive(Debug)]
pub struct SignUpdateLeverageData {
    pub market_index: i32,
    pub initial_margin_fraction: i32,
    pub margin_mode: i32,
}

#[derive(Debug)]
pub struct SignUpdateMarginData {
    pub market_index: i32,
    pub usdc_amount: i64,
    pub direction: i32,
}
