use crate::{
    api::{
        account::AccountApi, announcement::AnnouncementApi, block::BlockApi, bridge::BridgeApi,
        candlestick::CandlestickApi, funding::FundingApi, info::InfoApi,
        notification::NotificationApi, order::OrderApi, referral::ReferralApi, root::RootApi,
        transaction::TransactionApi,
    },
    client::nonce::NonceManager,
    config::LighterConfig,
    LighterError, Result,
};

#[derive(Default, Debug)]
pub struct ApiInterface {
    account: Option<AccountApi>,
    announcement: Option<AnnouncementApi>,
    block: Option<BlockApi>,
    bridge: Option<BridgeApi>,
    candlestick: Option<CandlestickApi>,
    funding: Option<FundingApi>,
    info: Option<InfoApi>,
    notification: Option<NotificationApi>,
    order: Option<OrderApi>,
    referral: Option<ReferralApi>,
    root: Option<RootApi>,
    transaction: Option<TransactionApi>,
}

impl ApiInterface {
    pub fn account(&self) -> Result<&AccountApi> {
        self.account
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Account API not initialized".into()))
    }

    pub fn announcement(&self) -> Result<&AnnouncementApi> {
        self.announcement
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Announcement API not initialized".into()))
    }

    pub fn block(&self) -> Result<&BlockApi> {
        self.block
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Block API not initialized".into()))
    }

    pub fn candlestick(&self) -> Result<&CandlestickApi> {
        self.candlestick
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Candlestick API not initialized".into()))
    }

    pub fn funding(&self) -> Result<&FundingApi> {
        self.funding
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Funding API not initialized".into()))
    }

    pub fn info(&self) -> Result<&InfoApi> {
        self.info
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Info API not initialized".into()))
    }

    pub fn notification(&self) -> Result<&NotificationApi> {
        self.notification
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Notification API not initialized".into()))
    }

    pub fn order(&self) -> Result<&OrderApi> {
        self.order
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Order API not initialized".into()))
    }

    pub fn referral(&self) -> Result<&ReferralApi> {
        self.referral
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Referral API not initialized".into()))
    }

    pub fn root(&self) -> Result<&RootApi> {
        self.root
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Root API not initialized".into()))
    }

    pub fn transaction(&self) -> Result<&TransactionApi> {
        self.transaction
            .as_ref()
            .ok_or_else(|| LighterError::Generic("Transaction API not initialized".into()))
    }
}

#[derive(Debug)]
pub struct HttpClient {
    // instance specific
    account_index: i64,
    api_key_index: i32,
    apis: ApiInterface,
    nonce_manager: Option<NonceManager>, // it can be API or local nonce management, so it's optional
}

impl HttpClient {
    /// Returns a `HttpClientBuilder`
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
    }

    pub fn api(&self) -> &ApiInterface {
        &self.apis
    }

    pub async fn get_nonce(&self) -> Result<i64> {
        if let Some(nonce_manager) = &self.nonce_manager {
            nonce_manager.generate()
        } else {
            self.apis
                .transaction()?
                .next_nonce(self.account_index, self.api_key_index)
                .await
                .map(|v| v.nonce)
        }
    }
}

#[derive(Default)]
pub struct HttpClientBuilder {
    config: Option<LighterConfig>,
    account: bool,
    announcement: bool,
    block: bool,
    bridge: bool,
    candlestick: bool,
    funding: bool,
    info: bool,
    notification: bool,
    order: bool,
    referral: bool,
    root: bool,
    transaction: bool,
}

impl HttpClientBuilder {
    pub fn with_config(mut self, config: LighterConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_account(mut self) -> Self {
        self.account = true;
        self
    }

    pub fn with_announcement(mut self) -> Self {
        self.announcement = true;
        self
    }

    pub fn with_block(mut self) -> Self {
        self.block = true;
        self
    }

    pub fn with_bridge(mut self) -> Self {
        self.bridge = true;
        self
    }

    pub fn with_candlestick(mut self) -> Self {
        self.candlestick = true;
        self
    }

    pub fn with_funding(mut self) -> Self {
        self.funding = true;
        self
    }

    pub fn with_info(mut self) -> Self {
        self.info = true;
        self
    }

    pub fn with_notification(mut self) -> Self {
        self.notification = true;
        self
    }

    pub fn with_order(mut self) -> Self {
        self.order = true;
        self
    }

    pub fn with_referral(mut self) -> Self {
        self.referral = true;
        self
    }

    pub fn with_root(mut self) -> Self {
        self.root = true;
        self
    }

    pub fn with_transaction(mut self) -> Self {
        self.transaction = true;
        self
    }

    pub fn build(self) -> Result<HttpClient> {
        let config = self.config.unwrap_or_default();
        let mut apis = ApiInterface::default();

        if self.account {
            apis.account = Some(AccountApi::new(&config)?);
        }

        if self.announcement {
            apis.announcement = Some(AnnouncementApi::new(&config)?);
        }

        if self.block {
            apis.block = Some(BlockApi::new(&config)?);
        }

        if self.bridge {
            apis.bridge = Some(BridgeApi::new(&config)?);
        }

        if self.candlestick {
            apis.candlestick = Some(CandlestickApi::new(&config)?);
        }

        if self.funding {
            apis.funding = Some(FundingApi::new(&config)?);
        }

        if self.info {
            apis.info = Some(InfoApi::new(&config)?);
        }

        if self.notification {
            apis.notification = Some(NotificationApi::new(&config)?);
        }

        if self.order {
            apis.order = Some(OrderApi::new(&config)?);
        }

        if self.referral {
            apis.referral = Some(ReferralApi::new(&config)?);
        }

        if self.root {
            apis.root = Some(RootApi::new(&config)?);
        }

        if self.transaction {
            apis.transaction = Some(TransactionApi::new(&config)?);
        }

        //let signer = Signer::try_from(&config)?;
        let mut client = HttpClient {
            account_index: config
                .account_index
                .ok_or_else(|| LighterError::Generic("`acount_index` is not set".into()))?,
            api_key_index: config
                .api_key_index
                .ok_or_else(|| LighterError::Generic("`api_key_index` is not set".into()))?,
            apis,
            nonce_manager: None, // API nonce
        };

        if config.local_nonce {
            client.nonce_manager = Some(NonceManager::new()); // Local nonce
        }

        Ok(client)
    }
}
