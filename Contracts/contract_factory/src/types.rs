use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Template {
    pub name: Symbol,
    pub wasm_hash: BytesN<32>,
    pub version: u32,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DeployedInstance {
    pub address: Address,
    pub template_name: Symbol,
    pub template_version: u32,
    pub owner: Address,
    pub deployed_at: u32,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct FactoryStats {
    pub total_deployed: u64,
    pub total_templates: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Paused,
    TotalDeployed,
    TemplateNames,
    Template(Symbol),
    Instance(Address),
    OwnerInstances(Address),
}
