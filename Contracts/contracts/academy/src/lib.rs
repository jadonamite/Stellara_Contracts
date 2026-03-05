#![no_std]

pub mod vesting;

pub use vesting::{
    AcademyVestingContract, VestingSchedule, GrantEvent, ClaimEvent, RevokeEvent, VestingError,
};
