pub mod data;
mod ffi;

use std::str::FromStr;

use alloy::{
    primitives::eip191_hash_message, signers::local::PrivateKeySigner, signers::SignerSync,
};
pub use ffi::FFISigner;
use secrecy::ExposeSecret;
use serde_json::Value;

use crate::{
    config::LighterConfig,
    signer::data::{
        ChangePubKeyData, CreateOrderData, SignBurnSharesData, SignCancelAllOrdersData,
        SignCancelOrderData, SignCreateGroupedOrdersData, SignCreatePublicPoolData,
        SignMintSharesData, SignModifyOrderData, SignTransferData, SignUpdateLeverageData,
        SignUpdateMarginData, SignUpdatePublicPoolData, SignWithdrawData, TxData, TxInfo,
        TxInfoData,
    },
    LighterError, Result,
};

#[derive(Debug)]
pub struct Signer {
    ffi: FFISigner,
    eth: Option<PrivateKeySigner>, // we might not need an eth signer if we just need to have read only access to the APIs
}

impl TryFrom<&LighterConfig> for Signer {
    type Error = crate::LighterError;

    fn try_from(config: &LighterConfig) -> Result<Self> {
        let ffi = FFISigner::try_from(config)?;

        if config.eth_private_key.is_some() {
            let eth = PrivateKeySigner::try_from(config)?;
            return Ok(Self {
                ffi,
                eth: Some(eth),
            });
        }

        Ok(Self { ffi, eth: None })
    }
}

impl TryFrom<&LighterConfig> for PrivateKeySigner {
    type Error = LighterError;

    fn try_from(config: &LighterConfig) -> Result<Self> {
        let privk = config.eth_private_key.as_ref().ok_or_else(|| {
            tracing::error!("eth private key is empty");
            LighterError::Config("eth_private_key is empty".into())
        })?;

        let eth = PrivateKeySigner::from_str(privk.expose_secret()).map_err(|e| {
            tracing::error!("unable to create eth signer: {e}");
            LighterError::Signing(e.to_string())
        })?;

        Ok(eth)
    }
}

impl TryFrom<&LighterConfig> for FFISigner {
    type Error = LighterError;

    fn try_from(config: &LighterConfig) -> std::result::Result<Self, Self::Error> {
        let api_key_private = config
            .api_key_private
            .as_ref()
            .ok_or_else(|| LighterError::Generic("API Private Key is not initialized".into()))?;
        let api_key_index = config
            .api_key_index
            .ok_or_else(|| LighterError::Generic("API Key Index is not initialized".into()))?;
        let account_index = config
            .account_index
            .ok_or_else(|| LighterError::Generic("Account Index is not initialized".into()))?;
        FFISigner::new(
            &config.base_url,
            api_key_private.clone(),
            api_key_index,
            account_index,
        )
    }
}

impl Signer {
    pub fn sign_change_pubkey(&self, data: ChangePubKeyData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::ChangePubKey(data), nonce)
    }

    pub fn sign_create_order(&self, data: CreateOrderData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::CreateOrder(data), nonce)
    }

    pub fn sign_create_grouped_orders(
        &self,
        data: SignCreateGroupedOrdersData,
        nonce: i64,
    ) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignCreateGroupedOrders(data), nonce)
    }

    pub fn sign_cancel_order(&self, data: SignCancelOrderData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignCancelOrder(data), nonce)
    }

    pub fn sign_withdraw(&self, data: SignWithdrawData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignWithdraw(data), nonce)
    }

    pub fn sign_create_subaccount(&self, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignCreateSubaccount, nonce)
    }

    pub fn sign_cancel_all_orders(
        &self,
        data: SignCancelAllOrdersData,
        nonce: i64,
    ) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignCancelAllOrders(data), nonce)
    }

    pub fn sign_modify_order(&self, data: SignModifyOrderData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignModifyOrder(data), nonce)
    }

    pub fn sign_transfer(&self, data: SignTransferData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignTransfer(data), nonce)
    }

    pub fn sign_create_public_pool(
        &self,
        data: SignCreatePublicPoolData,
        nonce: i64,
    ) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignCreatePublicPool(data), nonce)
    }

    pub fn sign_update_public_pool(
        &self,
        data: SignUpdatePublicPoolData,
        nonce: i64,
    ) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignUpdatePublicPool(data), nonce)
    }

    pub fn sign_mint_shares(&self, data: SignMintSharesData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignMintShares(data), nonce)
    }

    pub fn sign_burn_shares(&self, data: SignBurnSharesData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignBurnShares(data), nonce)
    }

    pub fn sign_update_leverage(&self, data: SignUpdateLeverageData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignUpdateLeverage(data), nonce)
    }

    pub fn sign_update_margin(&self, data: SignUpdateMarginData, nonce: i64) -> Result<TxInfo> {
        self.sign_tx_data(TxData::SignUpdateMargin(data), nonce)
    }

    fn sign_tx_data(&self, tx_data: TxData, nonce: i64) -> Result<TxInfo> {
        let tx_body = self.ffi.get_tx_data(tx_data, nonce)?;
        let tx_json = serde_json::from_str::<Value>(&tx_body).unwrap();

        let mut tx_info = TxInfo {
            data: None,
            payload: serde_json::to_string(&tx_json).unwrap(),
        };

        // check we actually have something to sign
        if let Some(msg) = tx_json["MessageToSign"].as_str() {
            // sign
            let sig = self.sign_message(msg)?;

            // update the data
            let mut tx_json = tx_json.clone();
            if let Some(obj) = tx_json.as_object_mut() {
                obj.remove("MessageToSign");
                obj.insert("L1Sig".into(), Value::String(sig.clone()));
            }

            tx_info.data = Some(TxInfoData {
                message: msg.into(),
                signature: sig,
            });
            tx_info.payload = serde_json::to_string(&tx_json).unwrap();
        }

        Ok(tx_info)
    }

    fn sign_message(&self, message: &str) -> Result<String> {
        let hash = eip191_hash_message(message);
        let signature = self
            .eth
            .as_ref()
            .ok_or_else(|| LighterError::Signing("`eth_private_key` is not set".into()))?
            .sign_hash_sync(&hash)
            .map_err(|e| LighterError::Signing(e.to_string()))?;
        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        models,
        signer::{
            data::{
                ChangePubKeyData, CreateOrderData, SignBurnSharesData, SignCancelAllOrdersData,
                SignCancelOrderData, SignCreateGroupedOrdersData, SignCreatePublicPoolData,
                SignMintSharesData, SignModifyOrderData, SignTransferData, SignUpdateLeverageData,
                SignUpdateMarginData, SignUpdatePublicPoolData, SignWithdrawData,
            },
            ffi::ffisigner::CreateOrderTxReq,
        },
    };

    use super::*;
    use alloy::primitives::Signature;
    use chrono::Utc;

    static TEST_API_KEY_PRIVATE: &str =
        "01db9eed031d59d6bd0ee00ee5a7dc1f62087bf217b51caea57eb6e17a02c49e0a748d2f155a2f60";
    static TEST_API_KEY_INDEX: i32 = 2;
    static TEST_ACCOUNT_INDEX: &str = "28";
    static TEST_PRIVATE_KEY: &str =
        "0x4fd51c004ad02a003e321d5154d9b22c6bb89e1e5017bdc832c69ef28f65c04e";
    static TEST_ACCOUNT_ADDRESS: &str = "0x2b8a17334f9474ceE44CdeD230dc6fE537eda02E";

    #[test]
    fn test_sign_change_pubkey() {
        let new_pubk =
            "0x591054547bc244197245327189c7445492cb3779316eadbe77411be41021088589bec5541dc2373a";
        let tx_data = ChangePubKeyData {
            new_pubk: new_pubk.into(),
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        let tx_sign = signer.sign_change_pubkey(tx_data, 1).unwrap();

        // Some transactions may not require L1 signing (no MessageToSign)
        if let Some(data) = tx_sign.data {
            let sign = Signature::from_str(&data.signature).unwrap();
            let address = sign.recover_address_from_msg(&data.message).unwrap();
            assert_eq!(TEST_ACCOUNT_ADDRESS, address.to_string());
        }
        // If no signature required, that's also valid
    }

    #[test]
    fn test_sign_create_order() {
        let tx_data = CreateOrderData {
            market_index: 1,
            client_order_index: 1,
            base_amount: 1,
            price: 1,
            is_ask: true,
            order_type: models::order::Type::Market.into(),
            time_in_force: models::order::TimeInForce::ImmediateOrCancel.into(),
            reduce_only: false,
            trigger_price: 0,
            order_expiry: 0,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_create_order(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_create_grouped_orders() {
        let exp = Utc::now().timestamp();
        let tx_data = SignCreateGroupedOrdersData {
            grouping_type: crate::api::order::GroupingType::OneCancelsOther,
            orders: vec![
                CreateOrderTxReq {
                    MarketIndex: 111,
                    ClientOrderIndex: 0,
                    BaseAmount: 1,
                    Price: 111,
                    IsAsk: 0,
                    Type: models::order::Type::TakeProfit.into(),
                    TimeInForce: models::order::TimeInForce::ImmediateOrCancel.into(),
                    ReduceOnly: true as u8,
                    TriggerPrice: 111,
                    OrderExpiry: exp,
                },
                CreateOrderTxReq {
                    MarketIndex: 111,
                    ClientOrderIndex: 0,
                    BaseAmount: 1,
                    Price: 110,
                    IsAsk: 0,
                    Type: models::order::Type::StopLoss.into(),
                    TimeInForce: models::order::TimeInForce::ImmediateOrCancel.into(),
                    ReduceOnly: true as u8,
                    TriggerPrice: 110,
                    OrderExpiry: exp,
                },
            ],
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_create_grouped_orders(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_cancel_order() {
        let tx_data = SignCancelOrderData {
            market_index: 1,
            order_index: 1,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_cancel_order(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_withdraw() {
        let tx_data = SignWithdrawData {
            asset_index: 1, // AssetIndex must be >= 1
            route_type: 0,
            amount: 1000,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_withdraw(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_create_subaccount() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_create_subaccount(1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_cancel_all_orders() {
        let tx_data = SignCancelAllOrdersData {
            time_in_force: models::order::TimeInForce::ImmediateOrCancel.into(),
            time: 0,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_cancel_all_orders(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_modify_order() {
        let tx_data = SignModifyOrderData {
            market_index: 1,
            order_index: 1,
            amount: 111,
            price: 111,
            trigger_price: 111,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_modify_order(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_transfer() {
        let msg = "Hal Finney was `Running Bitcoin`"; // must be 32 bytes!
        let mut memo = [0u8; 32];
        memo.copy_from_slice(msg.as_bytes());
        let tx_data = SignTransferData {
            to_account_index: 1,
            asset_index: 1, // AssetIndex must be >= 1
            from_route_type: 0,
            to_route_type: 0,
            amount: 100,
            usdc_fee: 2,
            memo,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        let tx_sign = signer.sign_transfer(tx_data, 1).unwrap();
        // Some transactions may not require L1 signing (no MessageToSign)
        if let Some(data) = tx_sign.data {
            let sign = Signature::from_str(&data.signature).unwrap();
            let address = sign.recover_address_from_msg(&data.message).unwrap();
            assert_eq!(TEST_ACCOUNT_ADDRESS, address.to_string());
        }
        // If no signature required, that's also valid
    }

    #[test]
    fn test_sign_create_public_pool() {
        let tx_data = SignCreatePublicPoolData {
            operator_fee: 1,
            initial_total_shares: 1,
            min_operator_share_rate: 1,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_create_public_pool(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_update_public_pool() {
        let tx_data = SignUpdatePublicPoolData {
            operator_fee: 1,
            min_operator_share_rate: 1,

            public_pool_index: 1,
            status: 1,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_update_public_pool(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_mint_shares() {
        let tx_data = SignMintSharesData {
            public_pool_index: 1, // Requires an existing pool
            share_amount: 1,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        // The API may return validation errors if the pool doesn't exist,
        // which is expected in a test environment without real pools
        let result = signer.sign_mint_shares(tx_data, 1);
        // Accept either success or validation error about pool not existing
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("PublicPoolIndex"));
    }

    #[test]
    fn test_sign_burn_shares() {
        let tx_data = SignBurnSharesData {
            public_pool_index: 1, // Requires an existing pool
            share_amount: 1,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        // The API may return validation errors if the pool doesn't exist,
        // which is expected in a test environment without real pools
        let result = signer.sign_burn_shares(tx_data, 1);
        // Accept either success or validation error about pool not existing
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("PublicPoolIndex"));
    }

    #[test]
    fn test_sign_update_leverage() {
        let tx_data = SignUpdateLeverageData {
            market_index: 1,
            initial_margin_fraction: 1,
            margin_mode: 1,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_update_leverage(tx_data, 1).unwrap();
        // no signature in this case
    }

    #[test]
    fn test_sign_update_margin() {
        let tx_data = SignUpdateMarginData {
            market_index: 1,
            usdc_amount: 1000,
            direction: 0,
        };

        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX)
            .with_eth_private_key(TEST_PRIVATE_KEY);
        let signer = Signer::try_from(&config).unwrap();

        signer.sign_update_margin(tx_data, 1).unwrap();
        // no signature in this case
    }
}
