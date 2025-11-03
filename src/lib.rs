mod asset;
mod collateral;
mod ema;
mod legacy;
mod oracle;
mod owner;
mod upgrade;
mod utils;

pub use crate::asset::*;
pub use crate::ema::*;
use crate::legacy::*;
pub use crate::oracle::*;
pub use crate::utils::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::{UnorderedMap, IterableSet, IterableMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near, require, AccountId, NearToken, Gas, BorshStorageKey,
    Duration, Promise, Timestamp,
};
use near_sdk_macros::NearSchema;
use hex::{decode, encode};
use dcap_qvl::verify;

const NO_DEPOSIT: NearToken = NearToken::from_yoctonear(0);

const GAS_FOR_PROMISE: Gas = Gas::from_tgas(10);

const NEAR_CLAIM_DURATION: Duration = 24 * 60 * 60 * 10u64.pow(9);
// This is a safety margin in NEAR for to cover potential extra storage.
const SAFETY_MARGIN_NEAR_CLAIM: NearToken = NearToken::from_near(1);

pub type DurationSec = u32;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Oracles,
    Assets,
    ApprovedCodehashes,
}

#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Worker {
    checksum: String,
    codehash: String,
}

#[near(contract_state)]
pub struct Contract {
    pub oracles: UnorderedMap<AccountId, VOracle>,

    pub assets: UnorderedMap<AssetId, VAsset>,

    pub recency_duration_sec: DurationSec,

    pub owner_id: AccountId,

    pub near_claim_amount: NearToken,

    pub approved_codehashes: IterableSet<String>,

    pub worker_by_account_id: IterableMap<AccountId, Worker>,
}

#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct PriceData {
    pub timestamp: Timestamp,
    pub recency_duration_sec: DurationSec,

    pub prices: Vec<AssetOptionalPrice>,
}

#[ext_contract]
pub trait ExtPriceReceiver {
    fn oracle_on_call(&mut self, sender_id: AccountId, data: PriceData, msg: String);
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(
        //recency_duration_sec: DurationSec,
        owner_id: AccountId,
        //near_claim_amount: U128,
    ) -> Self {
        Self {
            oracles: UnorderedMap::new(StorageKey::Oracles),
            assets: UnorderedMap::new(StorageKey::Assets),
            recency_duration_sec: 3600,
            owner_id,
            near_claim_amount: NearToken::from_yoctonear(1000000000000000000000000),
            approved_codehashes: IterableSet::new(b"a"),
            worker_by_account_id: IterableMap::new(b"b"),
        }
    }

    /// Remove price data from removed oracle.
    pub fn clean_oracle_data(&mut self, account_id: AccountId, asset_ids: Vec<AssetId>) {
        assert!(self.internal_get_oracle(&account_id).is_none());
        for asset_id in asset_ids {
            let mut asset = self.internal_get_asset(&asset_id).expect("Unknown asset");
            if asset.remove_report(&account_id) {
                self.internal_set_asset(&asset_id, asset);
            }
        }
    }

    pub fn get_oracle(&self, account_id: AccountId) -> Option<Oracle> {
        self.internal_get_oracle(&account_id)
    }

    pub fn get_oracles(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<(AccountId, Oracle)> {
        unordered_map_pagination(&self.oracles, from_index, limit)
    }

    pub fn get_assets(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<(AssetId, Asset)> {
        unordered_map_pagination(&self.assets, from_index, limit)
    }

    pub fn get_asset(&self, asset_id: AssetId) -> Option<Asset> {
        self.internal_get_asset(&asset_id)
    }

    pub fn get_price_data(&self, asset_ids: Option<Vec<AssetId>>) -> PriceData {
        let asset_ids = asset_ids.unwrap_or_else(|| self.assets.keys().cloned().collect());
        let timestamp = env::block_timestamp();
        let timestamp_cut = timestamp.saturating_sub(to_nano(self.recency_duration_sec));
        let min_num_recent_reports = std::cmp::max(1, (self.oracles.len() + 1) / 2) as usize;

        PriceData {
            timestamp,
            recency_duration_sec: self.recency_duration_sec,
            prices: asset_ids
                .into_iter()
                .map(|asset_id| {
                    // EMA for a specific asset, e.g. wrap.near#3600 is 1 hour EMA for wrap.near
                    if let Some((base_asset_id, period_sec)) = asset_id.split_once('#') {
                        let period_sec: DurationSec =
                            period_sec.parse().expect("Failed to parse EMA period");
                        let asset = self.internal_get_asset(&base_asset_id.to_string());
                        AssetOptionalPrice {
                            asset_id,
                            price: asset.and_then(|asset| {
                                asset
                                    .emas
                                    .into_iter()
                                    .find(|ema| ema.period_sec == period_sec)
                                    .filter(|ema| ema.timestamp >= timestamp_cut)
                                    .and_then(|ema| ema.price)
                            }),
                        }
                    } else {
                        let asset = self.internal_get_asset(&asset_id);
                        AssetOptionalPrice {
                            asset_id,
                            price: asset.and_then(|asset| {
                                asset.median_price(timestamp_cut, min_num_recent_reports)
                            }),
                        }
                    }
                })
                .collect(),
        }
    }

    /// Returns price data for a given oracle ID and given list of asset IDs.
    /// If recency_duration_sec is given, then it uses the given duration instead of the one from
    /// the contract config.
    pub fn get_oracle_price_data(
        &self,
        account_id: AccountId,
        asset_ids: Option<Vec<AssetId>>,
        recency_duration_sec: Option<DurationSec>,
    ) -> PriceData {
        let asset_ids = asset_ids.unwrap_or_else(|| self.assets.keys().cloned().collect());
        let timestamp = env::block_timestamp();
        let recency_duration_sec = recency_duration_sec.unwrap_or(self.recency_duration_sec);
        let timestamp_cut = timestamp.saturating_sub(to_nano(recency_duration_sec));

        let oracle_id: AccountId = account_id.into();
        PriceData {
            timestamp,
            recency_duration_sec,
            prices: asset_ids
                .into_iter()
                .map(|asset_id| {
                    let asset = self.internal_get_asset(&asset_id);
                    AssetOptionalPrice {
                        asset_id,
                        price: asset.and_then(|asset| {
                            asset
                                .reports
                                .into_iter()
                                .find(|report| report.oracle_id == oracle_id)
                                .filter(|report| report.timestamp >= timestamp_cut)
                                .map(|report| report.price)
                        }),
                    }
                })
                .collect(),
        }
    }

    pub fn report_prices(&mut self, prices: Vec<AssetPrice>, claim_near: Option<bool>) {
        assert!(!prices.is_empty());
        let oracle_id = env::predecessor_account_id();
        let timestamp = env::block_timestamp();

        // Oracle stats
        let mut oracle = self.internal_get_oracle(&oracle_id).expect("Not an oracle");
        
        // Require approved codehash for price reporting
        self.require_approved_codehash(&oracle_id, &oracle);
        oracle.last_report = timestamp;
        oracle.price_reports += prices.len() as u64;

        if claim_near.unwrap_or(false) && oracle.last_near_claim + NEAR_CLAIM_DURATION <= timestamp
        {
            let liquid_balance = env::account_balance().as_yoctonear() + env::account_locked_balance().as_yoctonear()
                - env::storage_byte_cost().as_yoctonear() * u128::from(env::storage_usage());
            if liquid_balance > (self.near_claim_amount.as_yoctonear() + SAFETY_MARGIN_NEAR_CLAIM.as_yoctonear()) {
                oracle.last_near_claim = timestamp;
                Promise::new(oracle_id.clone()).transfer(self.near_claim_amount);
            }
        }

        self.internal_set_oracle(&oracle_id, oracle);

        // Updating prices
        for AssetPrice { asset_id, price } in prices {
            price.assert_valid();
            if let Some(mut asset) = self.internal_get_asset(&asset_id) {
                asset.remove_report(&oracle_id);
                asset.add_report(Report {
                    oracle_id: oracle_id.clone(),
                    timestamp,
                    price,
                });
                if !asset.emas.is_empty() {
                    let timestamp_cut =
                        timestamp.saturating_sub(to_nano(self.recency_duration_sec));
                    let min_num_recent_reports =
                        std::cmp::max(1, (self.oracles.len() + 1) / 2) as usize;
                    if let Some(median_price) =
                        asset.median_price(timestamp_cut, min_num_recent_reports)
                    {
                        for ema in asset.emas.iter_mut() {
                            ema.recompute(median_price, timestamp);
                        }
                    }
                }
                self.internal_set_asset(&asset_id, asset);
            } else {
                log!("Warning! Unknown asset ID: {}", asset_id);
            }
        }
    }

    pub fn register_agent(
        &mut self,
        quote_hex: String,
        collateral: String,
        checksum: String,
        tcb_info: String,
    ) -> bool {
        let collateral_data = crate::collateral::get_collateral(collateral);
        let quote = decode(quote_hex).unwrap();
        let now = env::block_timestamp() / 1000000000;
        let result = verify::verify(&quote, &collateral_data, now).expect("report is not verified");
        let report = result.report.as_td10().unwrap();
        let report_data = format!("{}", String::from_utf8_lossy(&report.report_data));

        // verify the predecessor matches the report data
        require!(
            env::predecessor_account_id() == report_data,
            format!("predecessor_account_id != report_data: {}", report_data)
        );

        let rtmr3 = encode(report.rt_mr3.to_vec());
        let (shade_agent_api_image, shade_agent_app_image) =
            crate::collateral::verify_codehash(tcb_info, rtmr3);

        // verify the code hashes are approved
        require!(self.approved_codehashes.contains(&shade_agent_api_image));
        require!(self.approved_codehashes.contains(&shade_agent_app_image));

        let predecessor = env::predecessor_account_id();
        
        // Check if oracle already exists
        assert!(self.internal_get_oracle(&predecessor).is_none(), "Oracle already exists");
        
        // Create oracle with codehash information
        let mut oracle = Oracle::new();
        oracle.codehash = Some(shade_agent_app_image);
        oracle.checksum = Some(checksum);
        
        self.internal_set_oracle(&predecessor, oracle);

        true
    }

    pub fn get_agent(&self, account_id: AccountId) -> Worker {
        self.worker_by_account_id
            .get(&account_id)
            .expect("no worker found")
            .to_owned()
    }
    
    #[payable]
    pub fn oracle_call(
        &mut self,
        receiver_id: AccountId,
        asset_ids: Option<Vec<AssetId>>,
        msg: String,
    ) -> Promise {
        self.assert_well_paid();

        let sender_id = env::predecessor_account_id();
        let price_data = self.get_price_data(asset_ids);
        let remaining_gas = env::prepaid_gas().as_gas() - env::used_gas().as_gas();
        assert!(remaining_gas >= GAS_FOR_PROMISE.as_gas());

        Promise::new(receiver_id)
            .function_call(
                "oracle_on_call".to_string(),
                serde_json::to_vec(&(sender_id, price_data, msg)).unwrap(),
                NO_DEPOSIT,
                Gas::from_gas(remaining_gas - GAS_FOR_PROMISE.as_gas()),
            )
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            oracles: UnorderedMap::new(StorageKey::Oracles),
            assets: UnorderedMap::new(StorageKey::Assets),
            recency_duration_sec: 0,
            owner_id: "".parse().unwrap(),
            near_claim_amount: NearToken::from_yoctonear(0),
            approved_codehashes: IterableSet::new(StorageKey::ApprovedCodehashes),
            worker_by_account_id: IterableMap::new(b"b"),
        }
    }
}

impl Contract {
    pub fn assert_well_paid(&self) {
        assert_one_yocto();
    }

    /// Will throw if oracle is not registered with a codehash in self.approved_codehashes
    fn require_approved_codehash(&self, oracle_id: &AccountId, oracle: &Oracle) {
        let codehash = oracle.codehash.as_ref().expect("Oracle must have approved codehash to report prices");
        require!(
            self.approved_codehashes.contains(codehash),
            format!("Oracle {} codehash {} is not approved", oracle_id, codehash)
        );
    }
}
