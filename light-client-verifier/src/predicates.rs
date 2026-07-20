//! Predicates for light block validation and verification.

use core::time::Duration;

use cometbft::{
    block::Height, chain::Id as ChainId, crypto::Sha256, hash::Hash, merkle::MerkleHash,
};

use crate::{
    errors::VerificationError,
    operations::{CommitValidator, VotingPowerCalculator},
    prelude::*,
    types::{Header, SignedHeader, Time, TrustThreshold, ValidatorSet},
};

/// Production predicates, using the default implementation
/// of the `VerificationPredicates` trait.
#[cfg(feature = "rust-crypto")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProdPredicates;

#[cfg(feature = "rust-crypto")]
impl VerificationPredicates for ProdPredicates {
    type Sha256 = cometbft::crypto::default::Sha256;
}

/// Defines the various predicates used to validate and verify light blocks.
pub trait VerificationPredicates: Send + Sync {
    /// The implementation of SHA256 digest
    type Sha256: MerkleHash + Sha256 + Default;

    /// Compare the provided validator_set_hash against the hash produced from hashing the validator
    /// set.
    fn validator_sets_match(
        &self,
        validators: &ValidatorSet,
        header_validators_hash: Hash,
    ) -> Result<(), VerificationError> {
        let validators_hash = validators.hash_with::<Self::Sha256>();
        if header_validators_hash == validators_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_validator_set(
                header_validators_hash,
                validators_hash,
            ))
        }
    }

    /// Check that the hash of the next validator set in the header match the actual one.
    fn next_validators_match(
        &self,
        next_validators: &ValidatorSet,
        header_next_validators_hash: Hash,
    ) -> Result<(), VerificationError> {
        let next_validators_hash = next_validators.hash_with::<Self::Sha256>();
        if header_next_validators_hash == next_validators_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_next_validator_set(
                header_next_validators_hash,
                next_validators_hash,
            ))
        }
    }

    /// Check that the hash of the header in the commit matches the actual one.
    fn header_matches_commit(
        &self,
        header: &Header,
        commit_hash: Hash,
    ) -> Result<(), VerificationError> {
        let header_hash = header.hash_with::<Self::Sha256>();
        if header_hash == commit_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_commit_value(
                header_hash,
                commit_hash,
            ))
        }
    }

    /// Validate the commit using the given commit validator.
    fn valid_commit(
        &self,
        signed_header: &SignedHeader,
        validators: &ValidatorSet,
        commit_validator: &dyn CommitValidator,
    ) -> Result<(), VerificationError> {
        commit_validator.validate(signed_header, validators)?;
        commit_validator.validate_full(signed_header, validators)?;

        Ok(())
    }

    /// Check that the trusted header is within the trusting period, adjusting for clock drift.
    fn is_within_trust_period(
        &self,
        trusted_header_time: Time,
        trusting_period: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        let expires_at =
            (trusted_header_time + trusting_period).map_err(VerificationError::cometbft)?;

        if expires_at > now {
            Ok(())
        } else {
            Err(VerificationError::not_within_trust_period(expires_at, now))
        }
    }

    /// Check that the untrusted header is from past.
    fn is_header_from_past(
        &self,
        untrusted_header_time: Time,
        clock_drift: Duration,
        now: Time,
    ) -> Result<(), VerificationError> {
        let drifted = (now + clock_drift).map_err(VerificationError::cometbft)?;

        if untrusted_header_time < drifted {
            Ok(())
        } else {
            Err(VerificationError::header_from_the_future(
                untrusted_header_time,
                now,
                clock_drift,
            ))
        }
    }

    /// Check that time passed monotonically between the trusted header and the untrusted one.
    fn is_monotonic_bft_time(
        &self,
        untrusted_header_time: Time,
        trusted_header_time: Time,
    ) -> Result<(), VerificationError> {
        if untrusted_header_time > trusted_header_time {
            Ok(())
        } else {
            Err(VerificationError::non_monotonic_bft_time(
                untrusted_header_time,
                trusted_header_time,
            ))
        }
    }

    /// Check that the height increased between the trusted header and the untrusted one.
    fn is_monotonic_height(
        &self,
        untrusted_height: Height,
        trusted_height: Height,
    ) -> Result<(), VerificationError> {
        if untrusted_height > trusted_height {
            Ok(())
        } else {
            Err(VerificationError::non_increasing_height(
                untrusted_height,
                trusted_height.increment(),
            ))
        }
    }

    /// Check that the chain-ids of the trusted header and the untrusted one are the same
    fn is_matching_chain_id(
        &self,
        untrusted_chain_id: &ChainId,
        trusted_chain_id: &ChainId,
    ) -> Result<(), VerificationError> {
        if untrusted_chain_id == trusted_chain_id {
            Ok(())
        } else {
            Err(VerificationError::chain_id_mismatch(
                untrusted_chain_id.to_string(),
                trusted_chain_id.to_string(),
            ))
        }
    }

    /// Checks that there is enough overlap between validators and the untrusted
    /// signed header.
    fn has_sufficient_validators_and_signers_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        trusted_validators: &ValidatorSet,
        trust_threshold: &TrustThreshold,
        untrusted_validators: &ValidatorSet,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        calculator.check_enough_trust_and_signers(
            untrusted_sh,
            trusted_validators,
            *trust_threshold,
            untrusted_validators,
        )?;
        Ok(())
    }

    /// Check that there is enough signers overlap between the given, untrusted
    /// validator set and the untrusted signed header.
    fn has_sufficient_signers_overlap(
        &self,
        untrusted_sh: &SignedHeader,
        untrusted_validators: &ValidatorSet,
        calculator: &dyn VotingPowerCalculator,
    ) -> Result<(), VerificationError> {
        calculator.check_signers_overlap(untrusted_sh, untrusted_validators)?;
        Ok(())
    }

    /// Check that the hash of the next validator set in the trusted block matches
    /// the hash of the validator set in the untrusted one.
    fn valid_next_validator_set(
        &self,
        untrusted_validators_hash: Hash,
        trusted_next_validators_hash: Hash,
    ) -> Result<(), VerificationError> {
        if trusted_next_validators_hash == untrusted_validators_hash {
            Ok(())
        } else {
            Err(VerificationError::invalid_next_validator_set(
                untrusted_validators_hash,
                trusted_next_validators_hash,
            ))
        }
    }
}
