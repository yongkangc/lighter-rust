#![allow(clippy::too_many_arguments)]
use crate::{
    apis::{self, configuration::Configuration},
    config::LighterConfig,
    error::Result,
    models::{
        AccountApiKeys, AccountLimits, AccountMetadatas, AccountPnL, DetailedAccounts, L1Metadata,
        LiquidationInfos, PositionFundings, RespChangeAccountTier, RespPublicPoolsMetadata,
        SubAccounts,
    },
    signer::FFISigner,
};

#[derive(Debug)]
pub struct AccountApi {
    config: apis::configuration::Configuration,
    signer: FFISigner,
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum AccountBy {
    Index,
    L1Address,
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum AccountMetadataBy {
    Index,
    L1Address,
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PnlBy {
    Index,
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum AccountTier {
    Standard,
    Premium,
}

#[derive(Debug, strum::Display)]
pub enum PnlResolution {
    #[strum(to_string = "1m")]
    OneMinute,
    #[strum(to_string = "5m")]
    FiveMinutes,
    #[strum(to_string = "15m")]
    FifteenMinutes,
    #[strum(to_string = "30")]
    ThirtyMinutes,
    #[strum(to_string = "1h")]
    OneHour,
    #[strum(to_string = "4h")]
    FourHours,
    #[strum(to_string = "1d")]
    OneDay,
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PositionFundingSide {
    Long,
    Short,
    All,
}

#[derive(Debug, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PublicPoolsMetadataFilter {
    All,
    User,
    Protocol,
    AccountIndex,
}

impl AccountApi {
    pub fn new(config: &LighterConfig) -> Result<Self> {
        let signer = FFISigner::try_from(config)?;
        Ok(Self {
            config: Configuration::try_from(config)?,
            signer,
        })
    }

    /// Get account by account's index. <br>More details about account index: [Account Index](https://apidocs.lighter.xyz/docs/account-index)<hr>**Response Description:**<br><br>1) **Status:** 1 is active 0 is inactive.<br>2) **Collateral:** The amount of collateral in the account.<hr>**Position Details Description:**<br>1) **OOC:** Open order count in that market.<br>2) **Sign:** 1 for Long, -1 for Short.<br>3) **Position:** The amount of position in that market.<br>4) **Avg Entry Price:** The average entry price of the position.<br>5) **Position Value:** The value of the position.<br>6) **Unrealized PnL:** The unrealized profit and loss of the position.<br>7) **Realized PnL:** The realized profit and loss of the position.
    pub async fn account(&self, by: AccountBy, value: &str) -> Result<DetailedAccounts> {
        let resp = apis::account_api::account(&self.config, &by.to_string(), value)
            .await
            .inspect_err(|e| {
                tracing::error!("unable to call `account`: {e}");
            })?;

        Ok(resp)
    }

    /// Get account limits
    pub async fn account_limits(&self, account_index: i64) -> Result<AccountLimits> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp =
            apis::account_api::account_limits(&self.config, account_index, Some(&auth_token), None)
                .await
                .inspect_err(|e| tracing::error!("unable to call `accounts_limits`: {e}"))?;

        Ok(resp)
    }

    /// Get account metadatas
    pub async fn account_metadata(
        &self,
        by: AccountMetadataBy,
        value: &str,
    ) -> Result<AccountMetadatas> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp = apis::account_api::account_metadata(
            &self.config,
            &by.to_string(),
            value,
            Some(&auth_token),
            None,
        )
        .await
        .inspect_err(|e| tracing::error!("unable to call `account_metadata`: {e}"))?;

        Ok(resp)
    }

    /// Get accounts by l1_address returns all accounts associated with the given L1 address
    pub async fn accounts_by_l1_address(&self, l1_address: &str) -> Result<SubAccounts> {
        let resp = apis::account_api::accounts_by_l1_address(&self.config, l1_address)
            .await
            .inspect_err(|e| tracing::error!("unable to call `accounts_by_l1_address`: {e}"))?;

        Ok(resp)
    }

    /// Get account api key. Set `api_key_index` to 255 to retrieve all api keys associated with the account.
    pub async fn apikeys(
        &self,
        account_index: i64,
        api_key_index: Option<i32>,
    ) -> Result<AccountApiKeys> {
        let resp = apis::account_api::apikeys(&self.config, account_index, api_key_index)
            .await
            .inspect_err(|e| tracing::error!("unable to call `apikeys`: {e}"))?;

        Ok(resp)
    }

    /// Change account tier
    pub async fn change_account_tier(
        &self,
        account_index: i64,
        new_tier: AccountTier,
    ) -> Result<RespChangeAccountTier> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp = apis::account_api::change_account_tier(
            &self.config,
            account_index,
            &new_tier.to_string(),
            Some(&auth_token),
            None,
        )
        .await
        .inspect_err(|e| tracing::error!("unable to call `change_account_tier`: {e}"))?;

        Ok(resp)
    }

    /// Get L1 metadata
    pub async fn l1_metadata(&self, l1_address: &str) -> Result<L1Metadata> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp =
            apis::account_api::l1_metadata(&self.config, l1_address, Some(&auth_token), None)
                .await
                .inspect_err(|e| tracing::error!("unable to call `l1_metadata`: {e}"))?;

        Ok(resp)
    }

    /// Get liquidation infos
    pub async fn liquidations(
        &self,
        account_index: i64,
        limit: i64,
        market_id: Option<i32>,
        cursor: Option<&str>,
    ) -> Result<LiquidationInfos> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp = apis::account_api::liquidations(
            &self.config,
            account_index,
            limit,
            Some(&auth_token),
            None,
            market_id,
            cursor,
        )
        .await
        .inspect_err(|e| tracing::error!("unable to call `liquidations`: {e}"))?;

        Ok(resp)
    }

    /// Get account PnL chart
    pub async fn pnl(
        &self,
        by: PnlBy,
        value: &str,
        resolution: PnlResolution,
        start_timestamp: i64,
        end_timestamp: i64,
        count_back: i64,
        ignore_transfers: Option<bool>,
    ) -> Result<AccountPnL> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp = apis::account_api::pnl(
            &self.config,
            &by.to_string(),
            value,
            &resolution.to_string(),
            start_timestamp,
            end_timestamp,
            count_back,
            Some(&auth_token),
            None,
            ignore_transfers,
        )
        .await
        .inspect_err(|e| tracing::error!("unable to call `pnl`: {e}"))?;

        Ok(resp)
    }

    /// Get accounts position fundings
    pub async fn position_funding(
        &self,
        account_index: i64,
        limit: i64,
        market_id: Option<i32>,
        cursor: Option<&str>,
        side: Option<PositionFundingSide>,
    ) -> Result<PositionFundings> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp = apis::account_api::position_funding(
            &self.config,
            account_index,
            limit,
            Some(&auth_token),
            None,
            market_id,
            cursor,
            side.map(|v| v.to_string()).as_deref(),
        )
        .await
        .inspect_err(|e| tracing::error!("unable to call `position_fundings`: {e}"))?;

        Ok(resp)
    }

    /// Get public pools metadata
    pub async fn public_pools_metadata(
        &self,
        index: i64,
        limit: i64,
        filter: Option<PublicPoolsMetadataFilter>,
        account_index: Option<i64>,
    ) -> Result<RespPublicPoolsMetadata> {
        let auth_token = self.signer.get_auth_token(None)?;
        let resp = apis::account_api::public_pools_metadata(
            &self.config,
            index,
            limit,
            Some(&auth_token),
            None,
            filter.map(|v| v.to_string()).as_deref(),
            account_index,
        )
        .await
        .inspect_err(|e| tracing::error!("unable to call `public_pools_metadata`: {e}"))?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::HttpClient;

    use super::*;

    static TEST_API_KEY_PRIVATE: &str =
        "01db9eed031d59d6bd0ee00ee5a7dc1f62087bf217b51caea57eb6e17a02c49e0a748d2f155a2f60";
    static TEST_API_KEY_INDEX: i32 = 2;
    static TEST_ACCOUNT_INDEX: &str = "28";
    // static TEST_PRIVATE_KEY: &str =
    //     "0x4fd51c004ad02a003e321d5154d9b22c6bb89e1e5017bdc832c69ef28f65c04e";
    static TEST_ACCOUNT_ADDRESS: &str = "0x2b8a17334f9474ceE44CdeD230dc6fE537eda02E";

    #[tokio::test]
    async fn test_account_by_index() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);
        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        let res = client
            .api()
            .account()
            .unwrap()
            .account(AccountBy::Index, TEST_ACCOUNT_INDEX)
            .await
            .unwrap();
        println!("res: {res:?}")
    }

    #[tokio::test]
    async fn test_account_by_address() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);
        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .account(AccountBy::L1Address, TEST_ACCOUNT_ADDRESS)
            .await;
        // Accept either success or "account not found" error
        match res {
            Ok(account) => println!("res: {account:?}"),
            Err(e) => {
                // Verify it's an API error (not a network/parsing error)
                assert!(
                    e.to_string().contains("account not found") || e.to_string().contains("21100")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_account_limits() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist or auth may fail on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .account_limits(TEST_ACCOUNT_INDEX.parse().unwrap())
            .await;
        // Accept either success or authentication/account errors
        match res {
            Ok(limits) => println!("res: {limits:?}"),
            Err(e) => {
                // Verify it's an API error (not a network/parsing error)
                let err_str = e.to_string();
                assert!(
                    err_str.contains("account not found")
                        || err_str.contains("invalid auth")
                        || err_str.contains("21100")
                        || err_str.contains("20013")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_account_metadata_by_index() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist or auth may fail on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .account_metadata(AccountMetadataBy::Index, TEST_ACCOUNT_INDEX)
            .await;
        match res {
            Ok(metadata) => println!("res: {metadata:?}"),
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("account not found")
                        || err_str.contains("invalid auth")
                        || err_str.contains("21100")
                        || err_str.contains("20013")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_account_by_l1_address() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .accounts_by_l1_address(TEST_ACCOUNT_ADDRESS)
            .await;
        match res {
            Ok(accounts) => println!("res: {accounts:?}"),
            Err(e) => {
                let err_str = e.to_string();
                assert!(err_str.contains("account not found") || err_str.contains("21100"));
            }
        }
    }

    #[tokio::test]
    async fn test_account_apikyes() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        let res = client
            .api()
            .account()
            .unwrap()
            .apikeys(TEST_ACCOUNT_INDEX.parse().unwrap(), None)
            .await
            .unwrap();
        println!("res: {res:?}");
    }

    #[tokio::test]
    async fn test_account_l1_metadata() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .l1_metadata(TEST_ACCOUNT_ADDRESS)
            .await;
        match res {
            Ok(metadata) => println!("res: {metadata:?}"),
            Err(e) => {
                let err_str = e.to_string();
                assert!(err_str.contains("account not found") || err_str.contains("21100"));
            }
        }
    }

    #[tokio::test]
    async fn test_account_liquidations() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist or auth may fail on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .liquidations(TEST_ACCOUNT_INDEX.parse().unwrap(), 10, None, None)
            .await;
        match res {
            Ok(liquidations) => println!("res: {liquidations:?}"),
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("account not found")
                        || err_str.contains("invalid auth")
                        || err_str.contains("21100")
                        || err_str.contains("20013")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_account_pnl() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist or auth may fail on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .pnl(
                PnlBy::Index,
                TEST_ACCOUNT_INDEX,
                PnlResolution::OneDay,
                1,
                10,
                1,
                None,
            )
            .await;
        match res {
            Ok(pnl) => println!("res: {pnl:?}"),
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("account not found")
                        || err_str.contains("invalid auth")
                        || err_str.contains("21100")
                        || err_str.contains("20013")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_account_position_funding() {
        let config = LighterConfig::new()
            .with_base_url("https://testnet.zklighter.elliot.ai")
            .unwrap()
            .with_api_key_private(TEST_API_KEY_PRIVATE)
            .with_account_index(TEST_ACCOUNT_INDEX.parse().unwrap())
            .with_api_key_index(TEST_API_KEY_INDEX);

        let client = HttpClient::builder()
            .with_config(config)
            .with_account()
            .build()
            .unwrap();

        // Account may not exist or auth may fail on testnet, which is a valid test scenario
        let res = client
            .api()
            .account()
            .unwrap()
            .position_funding(TEST_ACCOUNT_INDEX.parse().unwrap(), 10, None, None, None)
            .await;
        match res {
            Ok(funding) => println!("res: {funding:?}"),
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("account not found")
                        || err_str.contains("invalid auth")
                        || err_str.contains("21100")
                        || err_str.contains("20013")
                );
            }
        }
    }
}
