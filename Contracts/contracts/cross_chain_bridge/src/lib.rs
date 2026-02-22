#![no_std]
#![allow(unexpected_cfgs)]

pub mod bridge;
pub mod storage;
pub mod types;
pub mod security;

pub use bridge::CrossChainBridgeContract;

pub use types::{
    // Core
    BridgeRequest,
    BridgeRequestStatus,
    BridgeDirection,
    ExternalChain,
    WrappedAsset,
    ValidatorSet,
    ValidatorSignature,
    ChainConfig,
    BridgeStats,
    PendingValidatorUpgrade,
    // Events
    BridgeInitiatedEvent,
    BridgeCompletedEvent,
    BridgeRejectedEvent,
    AssetRegisteredEvent,
    ValidatorAddedEvent,
    ValidatorRemovedEvent,
    ValidatorVoteEvent,
    EmergencyPauseEvent,
    ValidatorUpgradeProposedEvent,
    ValidatorUpgradeAppliedEvent,
    // Errors
    BridgeError,
};