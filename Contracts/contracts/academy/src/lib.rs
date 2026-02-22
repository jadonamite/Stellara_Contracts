#![no_std]
#![allow(unexpected_cfgs)]

pub mod vesting;
pub mod storage;

pub use vesting::{
    AcademyVestingContract,
    // V1 types
    VestingSchedule,
    BatchVestingRequest,
    BatchVestingResult,
    BatchClaimRequest,
    BatchClaimResult,
    BatchVestingOperation,
    GrantEvent,
    ClaimEvent,
    RevokeEvent,
    VestingError,
    // V2 types (issue #184)
    VestingScheduleV2,
    PerformanceTrigger,
    ConditionType,
    ScheduleModification,
};