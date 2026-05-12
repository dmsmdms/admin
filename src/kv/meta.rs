use crate::kv::kv::KvStore;
use serde::{Deserialize, Serialize};
use std::error::Error;
use strum::EnumDiscriminants;

#[repr(u8)]
#[derive(EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(name(MetaKey))]
pub enum MetaValue {
    LastBackupTs(i64) = 1,
    LastStatusCheckTs(i64) = 2,
}

impl KvStore {
    pub fn get_meta(key: MetaKey) -> Result<Option<MetaValue>, Box<dyn Error>> {
        let kv = Self::global();
        let rtxn = kv.env.read_txn()?;
        let value: Option<MetaValue> = kv.meta.get(&rtxn, &(key as u8))?;
        Ok(value)
    }

    pub fn set_meta(value: MetaValue) -> Result<(), Box<dyn Error>> {
        let kv = Self::global();
        let mut wtxn = kv.env.write_txn()?;
        let key = MetaKey::from(&value);
        kv.meta.put(&mut wtxn, &(key as u8), &value)?;
        wtxn.commit()?;
        Ok(())
    }
}
