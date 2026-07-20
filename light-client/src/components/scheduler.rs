//! Provides an interface and default implementation of the `Scheduler` component
use crate::{store::LightStore, verifier::types::Height};
use contracts::*;
use core::convert::TryInto;

/// Scheduler decides what block to verify next given the current and target heights.
#[contract_trait]
#[allow(missing_docs)]
pub trait Scheduler: Send + Sync {
    /// Decides what block to verify next.
    #[requires(light_store.highest_trusted_or_verified_before(target_height).is_some())]
    #[ensures(valid_schedule(ret, target_height, current_height, light_store))]
    fn schedule(
        &self,
        light_store: &dyn LightStore,
        current_height: Height,
        target_height: Height,
    ) -> Height;
}

#[contract_trait]
impl<F: Send + Sync> Scheduler for F
where
    F: Fn(&dyn LightStore, Height, Height) -> Height,
{
    fn schedule(
        &self,
        light_store: &dyn LightStore,
        current_height: Height,
        target_height: Height,
    ) -> Height {
        self(light_store, current_height, target_height)
    }
}

/// Basic bisecting scheduler which picks the appropriate midpoint without
/// optimizing for performance using the blocks available in the light store.
#[requires(light_store.highest_trusted_or_verified().is_some())]
#[ensures(valid_schedule(ret, target_height, current_height, light_store))]
pub fn basic_bisecting_schedule(
    light_store: &dyn LightStore,
    current_height: Height,
    target_height: Height,
) -> Height {
    let trusted_height = light_store
        .highest_trusted_or_verified_before(target_height)
        .map(|lb| lb.height())
        .unwrap();

    if trusted_height == current_height {
        target_height
    } else {
        midpoint(trusted_height, current_height)
    }
}

/// Checks whether the given `scheduled_height` is a valid schedule according to the
/// following specification.
pub fn valid_schedule(
    scheduled_height: Height,
    target_height: Height,
    current_height: Height,
    light_store: &dyn LightStore,
) -> bool {
    let latest_trusted_height = light_store
        .highest_trusted_or_verified_before(target_height)
        .map(|lb| lb.height())
        .unwrap();

    if latest_trusted_height == current_height && latest_trusted_height < target_height {
        current_height < scheduled_height && scheduled_height <= target_height
    } else if latest_trusted_height < current_height && latest_trusted_height < target_height {
        latest_trusted_height < scheduled_height && scheduled_height < current_height
    } else if latest_trusted_height == target_height {
        scheduled_height == target_height
    } else {
        true
    }
}

#[requires(low <= high)]
#[ensures(low <= ret && ret <= high)]
fn midpoint(low: Height, high: Height) -> Height {
    (low.value() + (high.value() + 1 - low.value()) / 2)
        .try_into()
        .unwrap()
}
