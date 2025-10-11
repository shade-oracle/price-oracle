use near_workspaces::{Account, AccountId, Contract, Worker, network::Sandbox};
use near_workspaces::types::{NearToken};
use serde_json::json;
use anyhow;

// Test constants
const CONTRACT_WASM_PATH: &str = "res/price_oracle.wasm";

// Helper function to create test environment
async fn setup_test_env() -> anyhow::Result<(Worker<Sandbox>, Account, Account, Account, Contract)> {
    // Create a sandbox worker
    let worker = near_workspaces::sandbox().await?;
    
    // Create test accounts
    let owner = worker.dev_create_account().await?;
    let oracle = worker.dev_create_account().await?;
    let user = worker.dev_create_account().await?;
    
    // Deploy the contract
    let wasm = std::fs::read(CONTRACT_WASM_PATH)?;
    let contract = owner.deploy(&wasm).await?.into_result()?;
    
    // Initialize the contract
    let init_result = contract
        .call("new")
        .args_json(json!({
            "recency_duration_sec": 3600,
            "owner_id": owner.id(),
            "near_claim_amount": "1000000000000000000000000" // 1 NEAR
        }))
        .transact()
        .await?;
    
    assert!(init_result.is_success());
    
    Ok((worker, owner, oracle, user, contract))
}

#[tokio::test]
async fn test_contract_initialization() -> anyhow::Result<()> {
    let (_worker, owner, _oracle, _user, contract) = setup_test_env().await?;
    
    // Test getting owner ID
    let owner_id: AccountId = contract
        .call("get_owner_id")
        .view()
        .await?
        .json()?;
    
    assert_eq!(owner_id, *owner.id());
    
    // Test getting version
    let version: String = contract
        .call("get_version")
        .view()
        .await?
        .json()?;
    
    assert_eq!(version, "0.6.0");
    
    Ok(())
}

#[tokio::test]
async fn test_oracle_management() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Add oracle
    let add_result = contract
        .call("add_oracle")
        .args_json(json!({
            "account_id": oracle.id()
        }))
        .deposit(NearToken::from_yoctonear(1)) // 1 yoctoNEAR
        .transact()
        .await?;
    
    assert!(add_result.is_success());
    
    // Get oracle info
    let oracle_info: serde_json::Value = contract
        .call("get_oracle")
        .args_json(json!({
            "account_id": oracle.id()
        }))
        .view()
        .await?
        .json()?;
    
    assert!(oracle_info.is_object());
    
    // List oracles
    let oracles: Vec<(AccountId, serde_json::Value)> = contract
        .call("get_oracles")
        .args_json(json!({
            "from_index": null,
            "limit": null
        }))
        .view()
        .await?
        .json()?;
    
    assert_eq!(oracles.len(), 1);
    assert_eq!(oracles[0].0, *oracle.id());
    
    Ok(())
}

#[tokio::test]
async fn test_price_reporting() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Add oracle first
    let add_oracle_result = contract
        .call("add_oracle")
        .args_json(json!({
            "account_id": oracle.id()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    assert!(add_oracle_result.is_success());
    
    // Add asset (call from oracle account)
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near",
            "emas": []
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_asset_result.is_success());
    
    // Report prices (call from oracle account)
    let report_result = oracle
        .call(contract.id(), "report_prices")
        .args_json(json!({
            "prices": [
                {
                    "asset_id": "wrap.near",
                    "price": {
                        "multiplier": 1000,
                        "decimals": 24
                    }
                }
            ],
            "claim_near": false
        }))
        .transact()
        .await?;
    
    assert!(report_result.is_success());
    
    // Get price data
    let price_data: serde_json::Value = contract
        .call("get_price_data")
        .args_json(json!({
            "asset_ids": ["wrap.near"]
        }))
        .view()
        .await?
        .json()?;
    
    assert!(price_data["prices"].is_array());
    assert_eq!(price_data["prices"].as_array().unwrap().len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_asset_management() -> anyhow::Result<()> {
    let (_worker, owner, _oracle, _user, contract) = setup_test_env().await?;
    
    // Add asset
    let add_result = contract
        .call("add_asset")
        .args_json(json!({
            "asset_id": "wrap.near",
            "emas": []
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_result.is_success());
    
    // Get asset
    let asset: serde_json::Value = contract
        .call("get_asset")
        .args_json(json!({
            "asset_id": "wrap.near"
        }))
        .view()
        .await?
        .json()?;
    
    assert!(asset.is_object());
    
    // List assets
    let assets: Vec<(String, serde_json::Value)> = contract
        .call("get_assets")
        .args_json(json!({
            "from_index": null,
            "limit": null
        }))
        .view()
        .await?
        .json()?;
    
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].0, "wrap.near");
    
    Ok(())
}

#[tokio::test]
async fn test_owner_functions() -> anyhow::Result<()> {
    let (_worker, owner, _oracle, _user, contract) = setup_test_env().await?;
    
    // Test updating recency duration
    let update_result = contract
        .call("set_recency_duration_sec")
        .args_json(json!({
            "recency_duration_sec": 7200
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(update_result.is_success());
    
    // Test updating near claim amount
    let claim_update_result = contract
        .call("update_near_claim_amount")
        .args_json(json!({
            "near_claim_amount": "2000000000000000000000000" // 2 NEAR
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(claim_update_result.is_success());
    
    // Verify the update
    let claim_amount: String = contract
        .call("get_near_claim_amount")
        .view()
        .await?
        .json()?;
    
    assert_eq!(claim_amount, "2000000000000000000000000");
    
    Ok(())
}

// #[tokio::test]
// async fn test_ema_functionality() -> anyhow::Result<()> {
//     let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
//     // Add oracle
//     contract
//         .call("add_oracle")
//         .args_json(json!({
//             "account_id": oracle.id()
//         }))
//         .deposit(NearToken::from_yoctonear(1))
//         .transact()
//         .await?;
    
//     // Add asset with EMA
//     contract
//         .call("add_asset")
//         .args_json(json!({
//             "asset_id": "wrap.near",
//             "emas": [
//                 {
//                     "period_sec": 3600
//                 }
//             ]
//         }))
//         .deposit(NearToken::from_yoctonear(1))
//         .transact()
//         .await?;
    
//     // Report prices multiple times to test EMA
//     for i in 0..3 {
//         let price_multiplier = 1000000000000000000000000000 + (i * 100000000000000000000000000);
        
//         contract
//             .call("report_prices")
//             .args_json(json!({
//                 "prices": [
//                     {
//                         "asset_id": "wrap.near",
//                         "price": {
//                             "multiplier": price_multiplier.to_string(),
//                             "decimals": 24
//                         }
//                     }
//                 ],
//                 "claim_near": false
//             }))
//             .transact()
//             .await?;
//     }
    
//     // Get price data for EMA asset
//     let price_data: serde_json::Value = contract
//         .call("get_price_data")
//         .args_json(json!({
//             "asset_ids": ["wrap.near#3600"]
//         }))
//         .view()
//         .await?
//         .json()?;
    
//     assert!(price_data["prices"].is_array());
//     let prices = price_data["prices"].as_array().unwrap();
//     assert_eq!(prices.len(), 1);
    
//     // The EMA price should be present
//     assert!(prices[0]["price"].is_object());
    
//     Ok(())
// }

#[tokio::test]
async fn test_migration_compatibility() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Test that the migrated contract maintains compatibility with existing functionality
    
    // Add oracle and asset
    let add_oracle_result = contract
        .call("add_oracle")
        .args_json(json!({
            "account_id": oracle.id()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    assert!(add_oracle_result.is_success());
    
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near",
            "emas": []
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    assert!(add_asset_result.is_success());
    
    // Report prices
    let report_result = oracle
        .call(contract.id(), "report_prices")
        .args_json(json!({
            "prices": [
                {
                    "asset_id": "wrap.near",
                    "price": {
                        "multiplier": 1000,
                        "decimals": 24
                    }
                }
            ],
            "claim_near": false
        }))
        .transact()
        .await?;

    println!("report_result: {:?}", report_result);
    assert!(report_result.is_success());
    
    // Test oracle-specific price data
    let oracle_price_data: serde_json::Value = contract
        .call("get_oracle_price_data")
        .args_json(json!({
            "account_id": oracle.id(),
            "asset_ids": ["wrap.near"],
            "recency_duration_sec": 3600
        }))
        .view()
        .await?
        .json()?;
    
    assert!(oracle_price_data["prices"].is_array());
    let prices = oracle_price_data["prices"].as_array().unwrap();
    assert_eq!(prices.len(), 1);
    assert_eq!(prices[0]["asset_id"], "wrap.near");
    
    Ok(())
}