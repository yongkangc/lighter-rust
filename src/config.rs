use std::time::Duration;

use crate::{
    apis::configuration::Configuration,
    error::{LighterError, Result},
};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{
    policies::ExponentialBackoff, Jitter, RetryTransientMiddleware, Retryable, RetryableStrategy,
};
use secrecy::SecretString;
use url::Url;

static DEFAULT_MIN_RETRY_INTERVAL: u64 = 100; // 100ms
static DEFAULT_MAX_RETRY_INTERVAL: u64 = 10000; // 10s
static DEFAULT_MAX_RETRIES: u32 = 10;
static DEFAUL_TIMEOUT: u64 = 30; // 30s
static DEFAULT_POOL_MAX_IDLE_PER_HOST: usize = 10;
static DEFAULT_POOL_TIMEOUT: u64 = 90; // 90s
static DEFAULT_TCP_KEEPALIVE_DURATION: u64 = 60; // 60s
static DEFAULT_TCP_NODELAY: bool = true;
static DEFAULT_HTTPV1_ONLY: bool = true;
static DEFAULT_CONNECTION_VERBOSE: bool = false;

/// Retries when the successfull response code is `429`.
struct TooManyRequestsStrategy;
impl RetryableStrategy for TooManyRequestsStrategy {
    fn handle(
        &self,
        res: &std::result::Result<reqwest::Response, reqwest_middleware::Error>,
    ) -> Option<reqwest_retry::Retryable> {
        match res {
            Ok(success) if success.status().as_u16() == 429 => Some(Retryable::Transient),
            Ok(success) if success.status().is_server_error() => Some(Retryable::Transient),
            Ok(_) => None, // do not retry in this case,
            Err(error) => reqwest_retry::default_on_request_failure(error),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LighterConfig {
    pub base_url: String,
    pub ws_url: String,
    pub account_index: Option<i64>,
    pub eth_private_key: Option<SecretString>,
    pub api_key_index: Option<i32>,
    pub api_key_private: Option<SecretString>,
    pub timeout_secs: Option<u64>,
    pub pool_max_idle_per_host: Option<usize>,
    pub pool_idle_timeout: Option<u64>,
    pub tcp_keepalive_duration: Option<u64>,
    pub tcp_nodelay: bool,
    pub http1_only: bool,
    pub connection_verbose: bool,
    pub retry_config: Option<RetryConfig>,
    pub local_nonce: bool,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub min_retry_interval: u64,
    pub max_retry_interval: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: DEFAULT_MAX_RETRIES,
            min_retry_interval: DEFAULT_MIN_RETRY_INTERVAL,
            max_retry_interval: DEFAULT_MAX_RETRY_INTERVAL,
        }
    }
}

impl LighterConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_account_index(mut self, account_index: i64) -> Self {
        self.account_index = Some(account_index);
        self
    }

    pub fn with_eth_private_key<S: Into<String>>(mut self, eth_private_key: S) -> Self {
        self.eth_private_key = Some(SecretString::from(eth_private_key.into()));
        self
    }

    pub fn with_api_key_index(mut self, api_key_index: i32) -> Self {
        self.api_key_index = Some(api_key_index);
        self
    }

    pub fn with_api_key_private<S: Into<String>>(mut self, api_key_private: S) -> Self {
        self.api_key_private = Some(SecretString::from(api_key_private.into()));
        self
    }

    pub fn with_base_url<S: AsRef<str>>(mut self, url: S) -> Result<Self> {
        self.base_url = Url::parse(url.as_ref())
            .map_err(|e| LighterError::Config(format!("Invalid base URL: {}", e)))?
            .to_string();
        Ok(self)
    }

    pub fn with_ws_url<S: AsRef<str>>(mut self, url: S) -> Result<Self> {
        self.ws_url = Url::parse(url.as_ref())
            .map_err(|e| LighterError::Config(format!("Invalid WebSocket URL: {}", e)))?
            .to_string();
        Ok(self)
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }

    pub fn with_pool_max_idle_per_host(mut self, pool_max_idle_per_host: usize) -> Self {
        self.pool_max_idle_per_host = Some(pool_max_idle_per_host);
        self
    }

    pub fn with_pool_idle_timeout(mut self, pool_idle_timeout: u64) -> Self {
        self.pool_idle_timeout = Some(pool_idle_timeout);
        self
    }

    pub fn with_tcp_keepalive_duration(mut self, tcp_keepalive_duration_secs: u64) -> Self {
        self.tcp_keepalive_duration = Some(tcp_keepalive_duration_secs);
        self
    }

    pub fn with_tcp_nodelay(mut self, tcp_nodelay: bool) -> Self {
        self.tcp_nodelay = tcp_nodelay;
        self
    }

    pub fn with_http1_only(mut self, http1_only: bool) -> Self {
        self.http1_only = http1_only;
        self
    }

    pub fn with_connection_verbose(mut self, connection_verbose: bool) -> Self {
        self.connection_verbose = connection_verbose;
        self
    }
}

impl Default for LighterConfig {
    fn default() -> Self {
        Self {
            base_url: "https://mainnet.zklighter.elliot.ai".to_string(),
            ws_url: "wss://mainnet.zklighter.elliot.ai/stream".to_string(),
            account_index: None,
            eth_private_key: None,
            api_key_index: None,
            api_key_private: None,
            timeout_secs: Some(DEFAUL_TIMEOUT),
            pool_max_idle_per_host: Some(DEFAULT_POOL_MAX_IDLE_PER_HOST),
            pool_idle_timeout: Some(DEFAULT_POOL_TIMEOUT),
            tcp_keepalive_duration: Some(DEFAULT_TCP_KEEPALIVE_DURATION),
            tcp_nodelay: DEFAULT_TCP_NODELAY,
            http1_only: DEFAULT_HTTPV1_ONLY,
            connection_verbose: DEFAULT_CONNECTION_VERBOSE,
            retry_config: Some(RetryConfig::default()),
            local_nonce: true, // by default we have the nonce generation as local to avoid further API requests; if `false` it will use API nonce
        }
    }
}

// Adding this trait implementation here so that the openapi generated file `apis/configuration.rs`
// can have as less changes as possible.
impl TryFrom<&LighterConfig> for Configuration {
    type Error = LighterError;

    fn try_from(config: &LighterConfig) -> std::result::Result<Self, Self::Error> {
        // create the inner client
        let mut builder = Client::builder();

        // timeout
        if let Some(timeout) = config.timeout_secs {
            builder = builder.timeout(Duration::from_secs(timeout));
        }

        // pool_max_idle_per_host
        if let Some(pool_max_idle_per_host) = config.pool_max_idle_per_host {
            builder = builder.pool_max_idle_per_host(pool_max_idle_per_host);
        }

        // pool_idle_timeout
        if let Some(pool_idle_timeout) = config.pool_idle_timeout {
            builder = builder.pool_idle_timeout(Duration::from_secs(pool_idle_timeout));
        }

        // tcp_keepalive
        if let Some(tcp_keepalive) = config.tcp_keepalive_duration {
            builder = builder.tcp_keepalive(Duration::from_secs(tcp_keepalive));
        }

        // tcp_nodelay
        builder = builder.tcp_nodelay(config.tcp_nodelay);

        // http1_only
        if config.http1_only {
            builder = builder.http1_only();
        }

        // connection_verbose
        builder = builder.connection_verbose(config.connection_verbose);

        let client = builder.build().map_err(|e| {
            tracing::error!("unable to create reqwest client: {e}");
            LighterError::Config("Unable to create client".into())
        })?;
        let mut middleware_builder = ClientBuilder::new(client);

        // retry strategy
        if let Some(retry_config) = &config.retry_config {
            let exp_backoff = ExponentialBackoff::builder()
                .retry_bounds(
                    Duration::from_millis(retry_config.min_retry_interval),
                    Duration::from_millis(retry_config.max_retry_interval),
                )
                .jitter(Jitter::Bounded)
                .build_with_max_retries(retry_config.max_retries);

            middleware_builder =
                middleware_builder.with(RetryTransientMiddleware::new_with_policy_and_strategy(
                    exp_backoff,
                    TooManyRequestsStrategy,
                ));
        }

        let openapi_config = Configuration {
            base_path: config.base_url.to_string(),
            user_agent: Some(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )),
            client: middleware_builder.build(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        };

        Ok(openapi_config)
    }
}
