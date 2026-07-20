//! Provides interface and a default implementation of the `Io` component.
use std::time::Duration;

use async_trait::async_trait;
use cometbft_rpc as rpc;
#[cfg(all(feature = "rpc-client", not(target_arch = "wasm32")))]
use cometbft_rpc::Client;
use flex_error::define_error;

use crate::verifier::types::{Height, LightBlock};

#[cfg(feature = "tokio")]
type TimeoutError = flex_error::DisplayOnly<tokio::time::error::Elapsed>;

#[cfg(not(feature = "tokio"))]
type TimeoutError = flex_error::NoSource;

/// Type for selecting either a specific height or the latest one
pub enum AtHeight {
    /// Specific height
    At(Height),
    /// Latest height
    Highest,
}

impl From<Height> for AtHeight {
    fn from(height: Height) -> Self {
        if height.value() == 0 {
            Self::Highest
        } else {
            Self::At(height)
        }
    }
}

define_error! {
    #[derive(Debug)]
    IoError {
        Rpc [ rpc::Error ]
            | _ | { "rpc error" },

        InvalidHeight
            | _ | {
                "invalid height: given height must be greater than 0"
            },

        HeightTooHigh
        {
            height: Height,
            latest_height: Height,
        }
        |e| {
            format_args!("height ({0}) is higher than latest height ({1})",
                e.height, e.latest_height)
        },

        InvalidValidatorSet
            [ cometbft::Error ]
            | _ | { "fetched validator set is invalid" },

        Timeout
            { duration: Duration }
            [ TimeoutError ]
            | e | {
                format_args!("task timed out after {} ms",
                    e.duration.as_millis())
            },

    }
}

impl IoError {
    pub fn from_rpc(err: rpc::Error) -> Self {
        Self::from_height_too_high(&err).unwrap_or_else(|| Self::rpc(err))
    }

    pub fn from_height_too_high(err: &rpc::Error) -> Option<Self> {
        use regex::Regex;

        let err_str = err.to_string();

        if err_str.contains("must be less than or equal to") {
            let re = Regex::new(
                r"height (\d+) must be less than or equal to the current blockchain height (\d+)",
            )
            .ok()?;

            let captures = re.captures(&err_str)?;
            let height = Height::try_from(captures[1].parse::<i64>().ok()?).ok()?;
            let latest_height = Height::try_from(captures[2].parse::<i64>().ok()?).ok()?;

            Some(Self::height_too_high(height, latest_height))
        } else {
            None
        }
    }
}

impl IoErrorDetail {
    /// Whether this error means that a timeout occurred when querying a node.
    pub fn is_timeout(&self) -> Option<Duration> {
        match self {
            Self::Timeout(e) => Some(e.duration),
            _ => None,
        }
    }
}

/// Interface for fetching light blocks from a full node (via RPC client).
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait Io: Send + Sync {
    /// Fetch a light block at the given height from a peer
    async fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError>;
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<F: Send + Sync> Io for F
where
    F: Fn(AtHeight) -> Result<LightBlock, IoError>,
{
    async fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
        self(height)
    }
}

#[cfg(all(feature = "rpc-client", not(target_arch = "wasm32")))]
pub use self::prod::ProdIo;

#[cfg(all(feature = "rpc-client", not(target_arch = "wasm32")))]
mod prod {
    use cometbft::{
        account::Id as TMAccountId, block::signed_header::SignedHeader as TMSignedHeader,
        validator::Set as TMValidatorSet,
    };
    use cometbft_rpc::Paging;
    use std::time::Duration;

    use super::*;
    use crate::verifier::types::PeerId;

    /// Production implementation of the Io component, which fetches
    /// light blocks from full nodes via RPC.
    #[derive(Clone, Debug)]
    pub struct ProdIo {
        peer_id: PeerId,
        rpc_client: rpc::HttpClient,
        timeout: Option<Duration>,
    }

    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    impl Io for ProdIo {
        async fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, IoError> {
            let signed_header = self.fetch_signed_header(height).await?;
            let height = signed_header.header.height;
            let proposer_address = signed_header.header.proposer_address;

            let validator_set = self
                .fetch_validator_set(height.into(), Some(proposer_address))
                .await?;
            let next_validator_set = self
                .fetch_validator_set(height.increment().into(), None)
                .await?;

            let light_block = LightBlock::new(
                signed_header,
                validator_set,
                next_validator_set,
                self.peer_id,
            );

            Ok(light_block)
        }
    }

    impl ProdIo {
        /// Constructs a new ProdIo component.
        ///
        /// Peer map which maps peer IDS to their network address must be supplied.
        pub fn new(
            peer_id: PeerId,
            rpc_client: rpc::HttpClient,
            timeout: Option<Duration>,
        ) -> Self {
            Self {
                peer_id,
                rpc_client,
                timeout,
            }
        }

        pub fn peer_id(&self) -> PeerId {
            self.peer_id
        }

        pub fn rpc_client(&self) -> &rpc::HttpClient {
            &self.rpc_client
        }

        pub fn timeout(&self) -> Option<Duration> {
            self.timeout
        }

        pub async fn fetch_signed_header(
            &self,
            height: AtHeight,
        ) -> Result<TMSignedHeader, IoError> {
            let client = self.rpc_client.clone();
            let request = async move {
                match height {
                    AtHeight::Highest => client.latest_commit().await,
                    AtHeight::At(height) => client.commit(height).await,
                }
            };

            let res = match self.timeout {
                Some(timeout) => tokio::time::timeout(timeout, request)
                    .await
                    .map_err(|e| IoError::timeout(timeout, e))?,
                None => request.await,
            };

            match res {
                Ok(response) => Ok(response.signed_header),
                Err(err) => Err(IoError::from_rpc(err)),
            }
        }

        pub async fn fetch_validator_set(
            &self,
            height: AtHeight,
            proposer_address: Option<TMAccountId>,
        ) -> Result<TMValidatorSet, IoError> {
            let height = match height {
                AtHeight::Highest => {
                    return Err(IoError::invalid_height());
                }
                AtHeight::At(height) => height,
            };

            let client = self.rpc_client.clone();
            let request = async move { client.validators(height, Paging::All).await };
            let response = match self.timeout {
                Some(timeout) => tokio::time::timeout(timeout, request)
                    .await
                    .map_err(|e| IoError::timeout(timeout, e))?,
                None => request.await,
            }
            .map_err(IoError::rpc)?;

            let validator_set = match proposer_address {
                Some(proposer_address) => {
                    TMValidatorSet::with_proposer(response.validators, proposer_address)
                        .map_err(IoError::invalid_validator_set)?
                }
                None => TMValidatorSet::without_proposer(response.validators),
            };

            Ok(validator_set)
        }
    }
}
