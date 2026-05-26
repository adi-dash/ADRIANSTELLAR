#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::token::StellarAssetClient as TokenAdminClient;

fn setup_env() -> (Env, Address, Address, Address, Address, TokenClient) {
    let env = Env::default();
    env.mock_all_auths();

    let mill = Address::generate(&env);
    let farmer = Address::generate(&env);
    let inspector = Address::generate(&env);
    let admin = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract(admin.clone());
    let token_client = TokenClient::new(&env, &token_contract);
    let token_admin = TokenAdminClient::new(&env, &token_contract);

    token_admin.mint(&mill, &1000);

    (env, mill, farmer, inspector, token_contract, token_client)
}

#[test]
fn test_successful_escrow_release() {
    let (env, mill, farmer, inspector, token, token_client) = setup_env();
    let contract_id = env.register_contract(None, PalayPayEscrow);
    let client = PalayPayEscrowClient::new(&env, &contract_id);

    client.initialize(&mill, &farmer, &inspector, &token, &500);
    client.deposit();
    
    assert_eq!(token_client.balance(&contract_id), 500);
    
    client.release_funds();
    
    assert_eq!(token_client.balance(&farmer), 500);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "Escrow is no longer active")]
fn test_double_release_fails() {
    let (env, mill, farmer, inspector, token, _) = setup_env();
    let contract_id = env.register_contract(None, PalayPayEscrow);
    let client = PalayPayEscrowClient::new(&env, &contract_id);

    client.initialize(&mill, &farmer, &inspector, &token, &500);
    client.deposit();
    client.release_funds();
    
    // Should panic
    client.release_funds();
}

#[test]
fn test_state_verification_after_init() {
    let (env, mill, farmer, inspector, token, _) = setup_env();
    let contract_id = env.register_contract(None, PalayPayEscrow);
    let client = PalayPayEscrowClient::new(&env, &contract_id);

    client.initialize(&mill, &farmer, &inspector, &token, &500);
    
    // Re-instantiate env to check raw storage if needed, or verify via failure paths
    // Since we don't have getters, we verify state by successfully executing cancel
    client.deposit();
    client.cancel();
    
    // If state was incorrect, cancel would have failed
}

#[test]
fn test_cancel_escrow() {
    let (env, mill, farmer, inspector, token, token_client) = setup_env();
    let contract_id = env.register_contract(None, PalayPayEscrow);
    let client = PalayPayEscrowClient::new(&env, &contract_id);

    client.initialize(&mill, &farmer, &inspector, &token, &500);
    client.deposit();
    
    assert_eq!(token_client.balance(&contract_id), 500);
    assert_eq!(token_client.balance(&mill), 500);
    
    client.cancel();
    
    assert_eq!(token_client.balance(&mill), 1000);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
#[should_panic]
fn test_unauthorized_release() {
    let env = Env::default();
    let mill = Address::generate(&env);
    let farmer = Address::generate(&env);
    let inspector = Address::generate(&env);
    let bad_actor = Address::generate(&env);
    let admin = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract(admin.clone());
    let token_admin = TokenAdminClient::new(&env, &token_contract);
    token_admin.mint(&mill, &1000);

    let contract_id = env.register_contract(None, PalayPayEscrow);
    let client = PalayPayEscrowClient::new(&env, &contract_id);

    client.initialize(&mill, &farmer, &inspector, &token_contract, &500);
    
    // We mock specific auth to simulate failure.
    // If the bad actor tries to invoke release_funds, auth fails.
    env.mock_auths(&[
        soroban_sdk::testutils::MockAuth {
            address: &bad_actor,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "release_funds",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }
    ]);

    client.release_funds();
}