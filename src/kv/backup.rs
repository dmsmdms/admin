use crate::kv::kv::KvStore;
use std::error::Error;

impl KvStore {
    pub fn get_last_backup_ts(service: &str) -> Result<i64, Box<dyn Error>> {
        let kv = Self::global();
        let rtxn = kv.env.read_txn()?;
        let value: Option<i64> = kv.backup_ts.get(&rtxn, service)?;
        Ok(value.unwrap_or(0))
    }

    pub fn set_last_backup_ts(service: &str, ts: i64) -> Result<(), Box<dyn Error>> {
        let kv = Self::global();
        let mut wtxn = kv.env.write_txn()?;
        kv.backup_ts.put(&mut wtxn, service, &ts)?;
        wtxn.commit()?;
        Ok(())
    }
}
