use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

use crate::errors::FactoryError;
use crate::types::{DataKey, DeployedInstance, FactoryStats, Template};

pub fn register_template(env: &Env, template: Template) -> Result<(), FactoryError> {
    if env
        .storage()
        .persistent()
        .get::<DataKey, Template>(&DataKey::Template(template.name.clone()))
        .is_some()
    {
        return Err(FactoryError::TemplateAlreadyExists);
    }

    let mut names: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&DataKey::TemplateNames)
        .unwrap_or_else(|| Vec::new(env));

    names.push_back(template.name.clone());
    env.storage()
        .persistent()
        .set(&DataKey::Template(template.name.clone()), &template);
    env.storage().instance().set(&DataKey::TemplateNames, &names);
    Ok(())
}

pub fn get_template(env: &Env, name: &Symbol) -> Result<Template, FactoryError> {
    env.storage()
        .persistent()
        .get(&DataKey::Template(name.clone()))
        .ok_or(FactoryError::TemplateNotFound)
}

pub fn update_template(env: &Env, name: &Symbol, new_wasm_hash: BytesN<32>) -> Result<Template, FactoryError> {
    let mut template = get_template(env, name)?;
    template.wasm_hash = new_wasm_hash;
    template.version += 1;
    env.storage()
        .persistent()
        .set(&DataKey::Template(name.clone()), &template);
    Ok(template)
}

pub fn set_template_active(env: &Env, name: &Symbol, active: bool) -> Result<(), FactoryError> {
    let mut template = get_template(env, name)?;
    template.active = active;
    env.storage()
        .persistent()
        .set(&DataKey::Template(name.clone()), &template);
    Ok(())
}

pub fn list_templates(env: &Env) -> Vec<Template> {
    let names: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&DataKey::TemplateNames)
        .unwrap_or_else(|| Vec::new(env));

    let mut templates: Vec<Template> = Vec::new(env);
    for name in names.iter() {
        if let Some(t) = env
            .storage()
            .persistent()
            .get::<DataKey, Template>(&DataKey::Template(name))
        {
            templates.push_back(t);
        }
    }
    templates
}

pub fn record_instance(env: &Env, instance: DeployedInstance) {
    let owner = instance.owner.clone();
    let addr = instance.address.clone();

    env.storage()
        .persistent()
        .set(&DataKey::Instance(addr.clone()), &instance);

    let mut owner_list: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerInstances(owner.clone()))
        .unwrap_or_else(|| Vec::new(env));

    owner_list.push_back(addr);
    env.storage()
        .persistent()
        .set(&DataKey::OwnerInstances(owner), &owner_list);

    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::TotalDeployed)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&DataKey::TotalDeployed, &(count + 1));
}

pub fn get_instance(env: &Env, address: &Address) -> Result<DeployedInstance, FactoryError> {
    env.storage()
        .persistent()
        .get(&DataKey::Instance(address.clone()))
        .ok_or(FactoryError::InstanceNotFound)
}

pub fn set_instance_active(env: &Env, address: &Address, active: bool) -> Result<(), FactoryError> {
    let mut instance = get_instance(env, address)?;
    if !instance.active && !active {
        return Err(FactoryError::InstanceAlreadyInactive);
    }
    instance.active = active;
    env.storage()
        .persistent()
        .set(&DataKey::Instance(address.clone()), &instance);
    Ok(())
}

pub fn get_owner_instances(env: &Env, owner: &Address) -> Vec<Address> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerInstances(owner.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn get_stats(env: &Env) -> FactoryStats {
    let total_deployed: u64 = env
        .storage()
        .instance()
        .get(&DataKey::TotalDeployed)
        .unwrap_or(0);

    let names: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&DataKey::TemplateNames)
        .unwrap_or_else(|| Vec::new(env));

    FactoryStats {
        total_deployed,
        total_templates: names.len(),
    }
}
