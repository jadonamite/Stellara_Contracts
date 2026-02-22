use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FactoryError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    Paused = 4,
    TemplateNotFound = 5,
    TemplateAlreadyExists = 6,
    TemplateInactive = 7,
    InstanceNotFound = 8,
    InstanceInactive = 9,
    InstanceAlreadyInactive = 10,
    InvalidWasmHash = 11,
}
