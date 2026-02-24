#![no_std]

mod deployer;
mod errors;
mod registry;
mod types;

use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Symbol, Val, Vec};

use errors::FactoryError;
use types::{DataKey, DeployedInstance, FactoryStats, Template};

#[contract]
pub struct ContractFactory;

fn get_admin(env: &Env) -> Result<Address, FactoryError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(FactoryError::NotInitialized)
}

fn require_admin(env: &Env) -> Result<(), FactoryError> {
    let admin = get_admin(env)?;
    admin.require_auth();
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), FactoryError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused {
        Err(FactoryError::Paused)
    } else {
        Ok(())
    }
}

#[contractimpl]
impl ContractFactory {
    pub fn initialize(env: Env, admin: Address) -> Result<(), FactoryError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(FactoryError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().set(&DataKey::TotalDeployed, &0u64);
        Ok(())
    }

    // Registers a new contract template (WASM hash) under a name.
    pub fn register_template(
        env: Env,
        name: Symbol,
        wasm_hash: BytesN<32>,
    ) -> Result<(), FactoryError> {
        require_admin(&env)?;
        let template = Template {
            name: name.clone(),
            wasm_hash,
            version: 1,
            active: true,
        };
        registry::register_template(&env, template)?;
        env.events()
            .publish((symbol_short!("tmpl_add"), name), ());
        Ok(())
    }

    // Updates an existing template to a new WASM hash and bumps its version.
    pub fn update_template(
        env: Env,
        name: Symbol,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), FactoryError> {
        require_admin(&env)?;
        let updated = registry::update_template(&env, &name, new_wasm_hash)?;
        env.events()
            .publish((symbol_short!("tmpl_upd"), name), updated.version);
        Ok(())
    }

    pub fn deactivate_template(env: Env, name: Symbol) -> Result<(), FactoryError> {
        require_admin(&env)?;
        registry::set_template_active(&env, &name, false)?;
        env.events()
            .publish((symbol_short!("tmpl_off"), name), ());
        Ok(())
    }

    pub fn reactivate_template(env: Env, name: Symbol) -> Result<(), FactoryError> {
        require_admin(&env)?;
        registry::set_template_active(&env, &name, true)?;
        Ok(())
    }

    // Deploys a new contract instance from a registered template.
    // init_fn is called on the new contract if init_args is non-empty.
    pub fn deploy(
        env: Env,
        template_name: Symbol,
        owner: Address,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> Result<Address, FactoryError> {
        require_not_paused(&env)?;
        owner.require_auth();

        let template = registry::get_template(&env, &template_name)?;
        if !template.active {
            return Err(FactoryError::TemplateInactive);
        }

        let nonce: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalDeployed)
            .unwrap_or(0);

        let salt = deployer::compute_salt(&env, &owner, nonce);
        let deployed_addr =
            deployer::deploy(&env, template.wasm_hash, salt, &init_fn, init_args);

        let instance = DeployedInstance {
            address: deployed_addr.clone(),
            template_name: template_name.clone(),
            template_version: template.version,
            owner: owner.clone(),
            deployed_at: env.ledger().sequence(),
            active: true,
        };

        registry::record_instance(&env, instance);

        env.events().publish(
            (symbol_short!("deployed"), deployed_addr.clone()),
            (template_name, owner),
        );

        Ok(deployed_addr)
    }

    // Upgrades a deployed instance to the current wasm_hash of its template.
    pub fn upgrade_instance(
        env: Env,
        caller: Address,
        instance_addr: Address,
    ) -> Result<(), FactoryError> {
        require_not_paused(&env)?;
        caller.require_auth();

        let instance = registry::get_instance(&env, &instance_addr)?;
        if !instance.active {
            return Err(FactoryError::InstanceInactive);
        }

        let admin = get_admin(&env)?;
        if instance.owner != caller && admin != caller {
            return Err(FactoryError::Unauthorized);
        }

        let template = registry::get_template(&env, &instance.template_name)?;
        deployer::upgrade_instance(&env, &instance_addr, template.wasm_hash);

        env.events().publish(
            (symbol_short!("upgraded"), instance_addr),
            template.version,
        );
        Ok(())
    }

    pub fn deactivate_instance(
        env: Env,
        caller: Address,
        instance_addr: Address,
    ) -> Result<(), FactoryError> {
        caller.require_auth();

        let instance = registry::get_instance(&env, &instance_addr)?;
        let admin = get_admin(&env)?;
        if instance.owner != caller && admin != caller {
            return Err(FactoryError::Unauthorized);
        }

        registry::set_instance_active(&env, &instance_addr, false)?;
        env.events()
            .publish((symbol_short!("inst_off"), instance_addr), ());
        Ok(())
    }

    pub fn pause(env: Env) -> Result<(), FactoryError> {
        require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), FactoryError> {
        require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), FactoryError> {
        require_admin(&env)?;
        new_admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Ok(())
    }

    // --- View functions ---

    pub fn get_template(env: Env, name: Symbol) -> Result<Template, FactoryError> {
        registry::get_template(&env, &name)
    }

    pub fn list_templates(env: Env) -> Vec<Template> {
        registry::list_templates(&env)
    }

    pub fn get_instance(env: Env, address: Address) -> Result<DeployedInstance, FactoryError> {
        registry::get_instance(&env, &address)
    }

    pub fn get_owner_instances(env: Env, owner: Address) -> Vec<Address> {
        registry::get_owner_instances(&env, &owner)
    }

    pub fn get_stats(env: Env) -> FactoryStats {
        registry::get_stats(&env)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }
}
