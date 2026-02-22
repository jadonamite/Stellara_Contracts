use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol, Val, Vec};

pub fn compute_salt(env: &Env, owner: &Address, nonce: u64) -> BytesN<32> {
    let mut data = Bytes::new(env);
    let owner_bytes: Bytes = owner.clone().into();
    data.append(&owner_bytes);
    data.append(&Bytes::from_slice(env, &nonce.to_be_bytes()));
    env.crypto().sha256(&data)
}

pub fn deploy(
    env: &Env,
    wasm_hash: BytesN<32>,
    salt: BytesN<32>,
    init_fn: &Symbol,
    init_args: Vec<Val>,
) -> Address {
    let deployed_addr = env
        .deployer()
        .with_current_contract(salt)
        .deploy(wasm_hash);

    if !init_args.is_empty() {
        let _: Val = env.invoke_contract(&deployed_addr, init_fn, init_args);
    }

    deployed_addr
}

pub fn upgrade_instance(env: &Env, instance: &Address, new_wasm_hash: BytesN<32>) {
    let upgrade_fn = Symbol::new(env, "upgrade");
    let mut args: Vec<Val> = Vec::new(env);
    args.push_back(new_wasm_hash.into());
    let _: Val = env.invoke_contract(instance, &upgrade_fn, args);
}
