//! Supervisor and Handle implementation.

use cometbft::block::Height;

use crate::{
    errors::Error,
    light_client::LightClient,
    state::State,
    verifier::types::{LightBlock, Status},
};

/// Light client `Instance` packages a `LightClient` together with its `State`.
#[derive(Debug)]
pub struct Instance {
    /// Light client for this instance
    pub light_client: LightClient,

    /// State of the light client for this instance
    pub state: State,
}

impl Instance {
    /// Constructs a new instance from the given light client and its state.
    pub fn new(light_client: LightClient, state: State) -> Self {
        Self {
            light_client,
            state,
        }
    }

    /// Return the peer id of this instance.
    pub fn peer_id(&self) -> &cometbft::node::Id {
        &self.light_client.peer
    }

    /// Get the latest trusted block.
    pub fn latest_trusted(&self) -> Option<LightBlock> {
        self.state.light_store.highest(Status::Trusted)
    }

    /// Trust the given block.
    pub fn trust_block(&mut self, lb: &LightBlock) {
        self.state.light_store.update(lb, Status::Trusted);
    }

    /// Get or fetch the block at the given height
    pub async fn get_or_fetch_block(&mut self, height: Height) -> Result<LightBlock, Error> {
        let (block, _) = self
            .light_client
            .get_or_fetch_block(height, &mut self.state)
            .await
            .map_err(|e| {
                if e.to_string()
                    .contains("must be less than or equal to the current blockchain height")
                {
                    Error::height_too_high(height, Height::default())
                } else {
                    e
                }
            })?;

        Ok(block)
    }
}
