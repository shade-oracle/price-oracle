use crate::*;
use near_sdk_macros::NearSchema;

#[derive(BorshSerialize, BorshDeserialize, Clone, NearSchema)]
pub struct AssetV0 {
    pub reports: Vec<Report>,
}

impl From<AssetV0> for Asset {
    fn from(v: AssetV0) -> Self {
        Asset {
            reports: v.reports,
            emas: vec![],
        }
    }
}
