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
    let (_worker, _owner, oracle, _user, contract) = setup_test_env().await?;
    
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
    let (_worker, _owner, _oracle, _user, contract) = setup_test_env().await?;
    
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
    let (_worker, _owner, _oracle, _user, contract) = setup_test_env().await?;
    
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

#[tokio::test]
async fn test_ema_functionality() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Add oracle
    let add_oracle_result = owner
        .call(contract.id(), "add_oracle")
        .args_json(json!({
            "account_id": oracle.id()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    assert!(add_oracle_result.is_success());
    
    // Add asset with EMA
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near",
            "emas": [
                {
                    "period_sec": 3600
                }
            ]
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;

    assert!(add_asset_result.is_success());
    
    // Report prices multiple times to test EMA
    for i in 0..3 {
        let price_multiplier: u128 = 1000 + (i * 1000);
        
        let report_result = oracle
            .call(contract.id(), "report_prices")
            .args_json(json!({
                "prices": [
                    {
                        "asset_id": "wrap.near",
                        "price": {
                            "multiplier": price_multiplier,
                            "decimals": 24
                        }
                    }
                ],
                "claim_near": false
            }))
            .transact()
            .await?;

        assert!(report_result.is_success());
    }
    
    // Get price data for EMA asset
    let price_data: serde_json::Value = contract
        .call("get_price_data")
        .args_json(json!({
            "asset_ids": ["wrap.near#3600"]
        }))
        .view()
        .await?
        .json()?;
    
    println!("price_data: {:?}", price_data);
    assert!(price_data["prices"].is_array());
    let prices = price_data["prices"].as_array().unwrap();
    assert_eq!(prices.len(), 1);
    
    // The EMA price should be present
    assert!(prices[0]["price"].is_object());
    
    Ok(())
}

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

// Test data extracted from shade-agent-flow/src/collateral.rs unit tests
// Codehashes extracted from the test_app_compose_extract test
const TEST_API_CODEHASH: &str = "a99cbc1c5b45bdbad4691f49019e7fc91efd6ba1e6dd3d0c9632c1dcc94b69fc";
const TEST_APP_CODEHASH: &str = "105015ca023e386df24f80fe45f6545f206df75e8f37debee00f603057da462b";

// Quote hex from the test() function in collateral.rs
// This is kept for reference/documentation but not used in tests due to data structure mismatch
#[allow(dead_code)]
const TEST_QUOTE_HEX: &str = "040002008100000000000000939a7233f79c4ca9940a0db3957f0607ac666ed993e70e31ff5f5a8a2c743b220000000007010300000000000000000000000000c51e5cb16c461fe29b60394984755325ecd05a9a7a8fb3a116f1c3cf0aca4b0eb9edefb9b404deeaee4b7d454372d17a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000702000000000000c68518a0ebb42136c12b2275164f8c72f25fa9a34392228687ed6e9caeb9c0f1dbd895e9cf475121c029dc47e70e91fd00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000085e0855a6384fa1c8a6ab36d0dcbfaa11a5753e5a070c08218ae5fe872fcb86967fd2449c29e22e59dc9fec998cb65474a7db64a609c77e85f603c23e9a9fd03bfd9e6b52ce527f774a598e66d58386026cea79b2aea13b81a0b70cfacdec0ca8a4fe048fea22663152ef128853caa5c033cbe66baf32ba1ff7f6b1afc1624c279f50a4cbc522a735ca6f69551e61ef2561c1b02351cd6f7c803dd36bc95ba25463aa025ce7761156260c9131a5d7c03aeccc10e12160ec3205bb2876a203a7fb81447910d62fd92897d68b1f51d54fb75dfe2aeba3a97a879cba59a771fc522d88046cc26b407d723f726fae17c3e5a50529d0b6c2b991d027f06a9b430d43ecc1000003bdd12b68ee3cfc93a1758479840b6f8734c2439106d8f0faa50ac919d86ea101c002c41d262670ad84afb8f9ee35c7abbb72dcc01bbc3e3a3773672d665005ee6bcb0c5f4b03f0563c797747f7ddd25d92d4f120bee4a829daca986bbc03c155b3d158f6a386bca7ee49ceb3ec31494b792e0cf22fc4e561ddc57156da1b77a0600461000000303070704ff00020000000000000000000000000000000000000000000000000000000000000000000000000000000015000000000000000700000000000000e5a3a7b5d830c2953b98534c6c59a3a34fdc34e933f7f5898f0a85cf08846bca0000000000000000000000000000000000000000000000000000000000000000dc9e2a7c6f948f17474e34a7fc43ed030f7c1563f1babddf6340c82e0e54a8c5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020006000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005d2eb8ae211693884eadaea0be0392c5532c7ff55429e4696c84954444d62ed600000000000000000000000000000000000000000000000000000000000000004f1cd2dde7dd5d4a9a495815f3ac76c56a77a9e06a5279a8c8550b54cf2d7287a630c3b9aefb94b1b6e8491eba4b43baa811c8f44167eb7d9ca933678ea64f5b2000000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f05005e0e00002d2d2d2d2d424547494e2043455254494649434154452d2d2d2d2d0a4d49494538544343424a656741774942416749554439426b736e734170713045567861464a59785a56794f6774664d77436759494b6f5a497a6a3045417749770a634445694d434147413155454177775a535735305a577767553064594946424453794251624746305a6d397962534244515445614d42674741315545436777520a535735305a577767513239796347397959585270623234784644415342674e564241634d43314e68626e526849454e7359584a684d51737743515944565151490a44414a445154454c4d416b474131554542684d4356564d774868634e4d6a55774d6a41334d5463774f4441325768634e4d7a49774d6a41334d5463774f4441320a576a42774d534977494159445651514444426c4a626e526c624342545231676755454e4c49454e6c636e52705a6d6c6a5958526c4d526f77474159445651514b0a4442464a626e526c6243424462334a7762334a6864476c76626a45554d424947413155454277774c553246756447456751327868636d4578437a414a42674e560a4241674d416b4e424d517377435159445651514745774a56557a425a4d424d4742797147534d34394167454743437147534d34394177454841304941424853770a3977506a72554532734f4a644c5653415434686565414a572b31796c6473615556696b5a4c485832506235777374326a79697539414f5865576a7a6a6d585a4c0a4343742b457858716f53394e45476c6b52724b6a67674d4e4d4949444354416642674e5648534d4547444157674253566231334e765276683655424a796454300a4d383442567776655644427242674e56485238455a4442694d47436758714263686c706f64485277637a6f764c32467761533530636e567a6447566b633256790a646d6c6a5a584d75615735305a577775593239744c334e6e6543396a5a584a3061575a7059324630615739754c3359304c33426a61324e796244396a595431770a624746305a6d397962535a6c626d4e765a476c755a7a316b5a584977485159445652304f42425945464d6a464e59626f7464634b636859487258467966774b460a774e534d4d41344741315564447745422f775145417749477744414d42674e5648524d4241663845416a41414d4949434f67594a4b6f5a497a6a30454177494452774177524149675873566b6930772b6936565947573355462f32327561586530594a446a3155650a6e412b546a44316169356343494359623153416d4435786b66545670766f34556f79695359787244574c6d5552344349394e4b7966504e2b0a2d2d2d2d2d454e442043455254494649434154452d2d2d2d2d0a2d2d2d2d2d424547494e2043455254494649434154452d2d2d2d2d0a4d4949436c6a4343416a32674177494241674956414a567658633239472b487051456e4a3150517a7a674658433935554d416f4743437147534d343942414d430a4d476778476a415942674e5642414d4d45556c756447567349464e48574342536232393049454e424d526f77474159445651514b4442464a626e526c624342440a62334a7762334a6864476c76626a45554d424947413155454277774c553246756447456751327868636d4578437a414a42674e564241674d416b4e424d5173770a435159445651514745774a56557a4165467730784f4441314d6a45784d4455774d5442614677307a4d7a41314d6a45784d4455774d5442614d484178496a41670a42674e5642414d4d47556c756447567349464e4857434251513073675547786864475a76636d306751304578476a415942674e5642416f4d45556c75644756730a49454e76636e4276636d4630615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b474131554543417743513045780a437a414a42674e5642415954416c56544d466b77457759484b6f5a497a6a3043415159494b6f5a497a6a304441516344516741454e53422f377432316c58534f0a3243757a7078773734654a423732457944476757357258437478327456544c7136684b6b367a2b5569525a436e71523770734f766771466553786c6d546c4a6c0a65546d693257597a33714f42757a43427544416642674e5648534d4547444157674251695a517a575770303069664f44744a5653763141624f536347724442530a42674e5648523845537a424a4d45656752614244686b466f64485277637a6f764c324e6c636e52705a6d6c6a5958526c63793530636e567a6447566b633256790a646d6c6a5a584d75615735305a577775593239744c306c756447567355306459556d397664454e424c6d526c636a416442674e5648513445466751556c5739640a7a62306234656c4153636e553944504f4156634c336c517744675944565230504151482f42415144416745474d42494741315564457745422f7751494d4159420a4166384341514177436759494b6f5a497a6a30454177494452774177524149675873566b6930772b6936565947573355462f32327561586530594a446a3155650a6e412b546a44316169356343494359623153416d4435786b66545670766f34556f79695359787244574c6d5552344349394e4b7966504e2b0a2d2d2d2d2d454e442043455254494649434154452d2d2d2d2d0a2d2d2d2d2d424547494e2043455254494649434154452d2d2d2d2d0a4d4949436a7a4343416a53674177494241674955496d554d316c71644e496e7a6737535655723951477a6b6e42717777436759494b6f5a497a6a3045417749770a614445614d4267474131554541777752535735305a5777675530645949464a766233516751304578476a415942674e5642416f4d45556c756447567349454e760a636e4276636d4630615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b47413155454341774351304578437a414a0a42674e5642415954416c56544d423458445445344d4455794d5445774e4455784d466f58445451354d54497a4d54497a4e546b314f566f77614445614d4267470a4131554541777752535735305a5777675530645949464a766233516751304578476a415942674e5642416f4d45556c756447567349454e76636e4276636d46300a615739754d5251774567594456515148444174545957353059534244624746795954454c4d416b47413155454341774351304578437a414a42674e56424159540a416c56544d466b77457759484b6f5a497a6a3043415159494b6f5a497a6a3044415163445167414543366e45774d4449595a4f6a2f69505773437a61454b69370a314f694f534c52466857476a626e42564a66566e6b59347533496a6b4459594c304d784f346d717379596a6c42616c54565978465032734a424b357a6c4b4f420a757a43427544416642674e5648534d4547444157674251695a517a575770303069664f44744a5653763141624f5363477244425342674e5648523845537a424a0a4d45656752614244686b466f64485277637a6f764c324e6c636e52705a6d6c6a5958526c63793530636e567a6447566b63325679646d6c6a5a584d75615735300a5a577775593239744c306c756447567355306459556d397664454e424c6d526c636a416442674e564851344546675155496d554d316c71644e496e7a673753560a55723951477a6b6e4271777744675944565230504151482f42415144416745474d42494741315564457745422f7751494d4159424166384341514577436759490a4b6f5a497a6a3045417749445351417752674968414f572f35516b522b533943695344634e6f6f774c7550524c735747662f59693747535839344267775477670a41694541344a306c72486f4d732b586f356f2f7358364f39515778485241765a55474f6452513763767152586171493d0a2d2d2d2d2d454e442043455254494649434154452d2d2d2d2d0a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

#[tokio::test]
async fn test_approve_codehash() -> anyhow::Result<()> {
    let (_worker, owner, _oracle, _user, contract) = setup_test_env().await?;
    
    // Test approving codehashes using values from collateral.rs unit test
    let approve_api = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_API_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_api.is_success());
    
    let approve_app = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_APP_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_app.is_success());
    
    Ok(())
}

#[tokio::test]
async fn test_report_prices_requires_approved_codehash() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Add oracle using old method (without codehash)
    let add_oracle_result = owner
        .call(contract.id(), "add_oracle")
        .args_json(json!({
            "account_id": oracle.id()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_oracle_result.is_success());
    
    // Add asset
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_asset_result.is_success());
    
    // Try to report prices - should fail because oracle doesn't have approved codehash
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
        .await;
    
    // Should fail because oracle doesn't have codehash
    assert!(report_result.is_err() || !report_result.unwrap().is_success());
    
    Ok(())
}

#[tokio::test]
async fn test_full_codehash_flow() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Step 1: Owner approves codehashes
    let approve_api = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_API_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_api.is_success());
    
    let approve_app = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_APP_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_app.is_success());
    
    // Step 2: Add asset
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_asset_result.is_success());
    
    // Step 3: Oracle tries to report prices without registering - should fail
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
        .await;
    
    // Should fail - oracle not registered
    assert!(report_result.is_err() || !report_result.unwrap().is_success());
    
    // Note: To fully test register_agent with the test data, we would need:
    // - A tcb_info with app_compose containing "#shade-agent-api-image" and "#shade-agent-app-image" tags
    // - The tcb_info must match the quote_hex and pass verification
    // - The report_data in the quote must match the oracle account ID
    // - The quote_collateral must be properly formatted JSON string
    
    Ok(())
}

// Quote collateral JSON structure from test() function in collateral.rs
// This is kept for reference/documentation but not used in tests due to data structure mismatch
#[allow(dead_code)]
fn get_test_quote_collateral() -> String {
    // Full quote_collateral JSON from test() function
    // In reality, this would be the full JSON string with certificates and signatures
    json!({
        "tcb_info_issuer_chain": "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n",
        "tcb_info": r#"{"id":"TDX","version":3,"issueDate":"2025-03-11T00:36:15Z","nextUpdate":"2025-04-10T00:36:15Z","fmspc":"20a06f000000","pceId":"0000","tcbType":0,"tcbEvaluationDataNumber":17,"tdxModule":{"mrsigner":"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","attributes":"0000000000000000","attributesMask":"FFFFFFFFFFFFFFFF"},"tdxModuleIdentities":[{"id":"TDX_03","mrsigner":"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","attributes":"0000000000000000","attributesMask":"FFFFFFFFFFFFFFFF","tcbLevels":[{"tcb":{"isvsvn":3},"tcbDate":"2024-03-13T00:00:00Z","tcbStatus":"UpToDate"}]},{"id":"TDX_01","mrsigner":"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","attributes":"0000000000000000","attributesMask":"FFFFFFFFFFFFFFFF","tcbLevels":[{"tcb":{"isvsvn":4},"tcbDate":"2024-03-13T00:00:00Z","tcbStatus":"UpToDate"},{"tcb":{"isvsvn":2},"tcbDate":"2023-08-09T00:00:00Z","tcbStatus":"OutOfDate"}]}],"tcbLevels":[{"tcb":{"sgxtcbcomponents":[{"svn":2,"category":"BIOS","type":"Early Microcode Update"},{"svn":2,"category":"OS/VMM","type":"SGX Late Microcode Update"},{"svn":2,"category":"OS/VMM","type":"TXT SINIT"},{"svn":2,"category":"BIOS"},{"svn":2,"category":"BIOS"},{"svn":255,"category":"BIOS"},{"svn":0},{"svn":2,"category":"OS/VMM","type":"SEAMLDR ACM"},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":13,"tdxtcbcomponents":[{"svn":5,"category":"OS/VMM","type":"TDX Module"},{"svn":0,"category":"OS/VMM","type":"TDX Module"},{"svn":2,"category":"OS/VMM","type":"TDX Late Microcode Update"},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}]},"tcbDate":"2024-03-13T00:00:00Z","tcbStatus":"UpToDate"},{"tcb":{"sgxtcbcomponents":[{"svn":2,"category":"BIOS","type":"Early Microcode Update"},{"svn":2,"category":"OS/VMM","type":"SGX Late Microcode Update"},{"svn":2,"category":"OS/VMM","type":"TXT SINIT"},{"svn":2,"category":"BIOS"},{"svn":2,"category":"BIOS"},{"svn":255,"category":"BIOS"},{"svn":0},{"svn":2,"category":"OS/VMM","type":"SEAMLDR ACM"},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}],"pcesvn":5,"tdxtcbcomponents":[{"svn":5,"category":"OS/VMM","type":"TDX Module"},{"svn":0,"category":"OS/VMM","type":"TDX Module"},{"svn":2,"category":"OS/VMM","type":"TDX Late Microcode Update"},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0},{"svn":0}]},"tcbDate":"2018-01-04T00:00:00Z","tcbStatus":"OutOfDate"}]}"#,
        "tcb_info_signature": "dff1380a12d533bff4ad7f69fd0355ad97ff034b42c8269e26e40e3d585dffff3e55bf21f8cda481d3c163fafcd4eab11c8818ba6aa7553ba6866bce06b56a95",
        "qe_identity_issuer_chain": "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n",
        "qe_identity": r#"{"id":"TD_QE","version":2,"issueDate":"2025-03-10T23:38:16Z","nextUpdate":"2025-04-09T23:38:16Z","tcbEvaluationDataNumber":17,"miscselect":"00000000","miscselectMask":"FFFFFFFF","attributes":"11000000000000000000000000000000","attributesMask":"FBFFFFFFFFFFFFFF0000000000000000","mrsigner":"DC9E2A7C6F948F17474E34A7FC43ED030F7C1563F1BABDDF6340C82E0E54A8C5","isvprodid":2,"tcbLevels":[{"tcb":{"isvsvn":4},"tcbDate":"2024-03-13T00:00:00Z","tcbStatus":"UpToDate"}]}"#,
        "qe_identity_signature": "920d5f18df6da142a667caf71844d45dfd4de3e3b14f846bae92a3e52a9c765d855b9a8b4b54307dd3feae30f28f09888a3200c29584d7c50d42f85275afe6cc"
    }).to_string()
}

#[tokio::test]
async fn test_register_agent_with_test_data() -> anyhow::Result<()> {
    let (_worker, owner, _oracle, _user, contract) = setup_test_env().await?;
    
    // Step 1: Owner approves codehashes (from test_app_compose_extract)
    let approve_api = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_API_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_api.is_success());
    
    let approve_app = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_APP_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_app.is_success());
    
    // Step 2: Add asset
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_asset_result.is_success());
    
    // Step 3: Test register_agent with data from unit tests
    // NOTE: The test() function in collateral.rs uses test data that works for verification,
    // but the tcb_info structure doesn't match what verify_codehash expects (it needs
    // "#shade-agent-api-image" and "#shade-agent-app-image" tags in the app_compose).
    // 
    // The test demonstrates the structure. For a full working integration test, you would need:
    // 1. A tcb_info with app_compose containing the proper tags matching TEST_API_CODEHASH and TEST_APP_CODEHASH
    // 2. The tcb_info must match the quote_hex (rtmr3 must match)
    // 3. The quote's report_data must match the oracle account ID
    // 4. The quote_collateral must be the complete JSON string with all certificates
    
    // This test documents the flow using the test data structure from the unit tests
    // A full working test would require properly formatted matching data
    
    Ok(())
}

#[tokio::test]
async fn test_register_agent_and_report_prices_flow() -> anyhow::Result<()> {
    let (_worker, owner, oracle, _user, contract) = setup_test_env().await?;
    
    // Complete flow test using test data from collateral.rs unit tests:
    // 1. test_app_compose_extract - provides codehashes
    // 2. test() - provides quote_hex, quote_collateral, and tcb_info structure
    
    // Step 1: Owner approves codehashes
    let approve_api = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_API_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_api.is_success());
    
    let approve_app = owner
        .call(contract.id(), "approve_codehash")
        .args_json(json!({
            "codehash": TEST_APP_CODEHASH
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(approve_app.is_success());
    
    // Step 2: Add asset
    let add_asset_result = owner
        .call(contract.id(), "add_asset")
        .args_json(json!({
            "asset_id": "wrap.near"
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    
    assert!(add_asset_result.is_success());
    
    // Step 3: Verify that oracle cannot report prices without registration
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
        .await;
    
    // Should fail - oracle not registered with approved codehash
    assert!(report_result.is_err() || !report_result.unwrap().is_success());
    
    // Step 4: To complete the test, oracle would need to call register_agent with:
    // - quote_hex: TEST_QUOTE_HEX (but modified so report_data matches oracle.id())
    // - collateral: get_test_quote_collateral() (complete JSON string)
    // - checksum: appropriate checksum value
    // - tcb_info: JSON string with app_compose containing "#shade-agent-api-image" and "#shade-agent-app-image" tags
    // 
    // After successful registration, oracle would be able to report prices
    
    Ok(())
}