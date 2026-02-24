use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BridgeError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    Paused = 4,
    MessageAlreadyProcessed = 5,
    InvalidSignatureCount = 6,
    InvalidSignerIndex = 7,
    ThresholdNotMet = 8,
    InvalidDestinationChain = 9,
    WrappedAssetNotFound = 10,
    ProposalNotFound = 11,
    ProposalExpired = 12,
    ProposalAlreadyExecuted = 13,
    AlreadyVoted = 14,
    ProposalNotApproved = 15,
    ValidatorNotFound = 16,
    InvalidThreshold = 17,
    DuplicateSigner = 18,
    VotingStillActive = 19,
}
