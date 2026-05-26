#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};
use soroban_sdk::token::Client as TokenClient;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Mill,
    Farmer,
    Inspector,
    Token,
    Amount,
    IsActive,
}

#[contract]
pub struct PalayPayEscrow;

#[contractimpl]
impl PalayPayEscrow {
    /// Initializes the escrow with the involved parties and the payment token.
    pub fn initialize(
        env: Env,
        mill: Address,
        farmer: Address,
        inspector: Address,
        token: Address,
        amount: i128,
    ) {
        mill.require_auth();
        
        env.storage().instance().set(&DataKey::Mill, &mill);
        env.storage().instance().set(&DataKey::Farmer, &farmer);
        env.storage().instance().set(&DataKey::Inspector, &inspector);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::IsActive, &true);
    }

    /// Mill deposits funds into the contract. Must be called after initialization.
    pub fn deposit(env: Env) {
        let mill: Address = env.storage().instance().get(&DataKey::Mill).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();
        
        mill.require_auth();

        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&mill, &env.current_contract_address(), &amount);
    }

    /// Inspector validates the harvest and releases funds to the farmer.
    pub fn release_funds(env: Env) {
        let inspector: Address = env.storage().instance().get(&DataKey::Inspector).unwrap();
        inspector.require_auth();

        let is_active: bool = env.storage().instance().get(&DataKey::IsActive).unwrap();
        assert!(is_active, "Escrow is no longer active");

        let farmer: Address = env.storage().instance().get(&DataKey::Farmer).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        // Mark as inactive to prevent double spend
        env.storage().instance().set(&DataKey::IsActive, &false);

        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &farmer, &amount);
    }

    /// Mill can cancel and refund the escrow if delivery fails (requires mill auth).
    pub fn cancel(env: Env) {
        let mill: Address = env.storage().instance().get(&DataKey::Mill).unwrap();
        mill.require_auth();

        let is_active: bool = env.storage().instance().get(&DataKey::IsActive).unwrap();
        assert!(is_active, "Escrow is no longer active");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        env.storage().instance().set(&DataKey::IsActive, &false);

        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &mill, &amount);
    }
}