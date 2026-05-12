use crate::kv::kv::KvStore;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Permission {
    Log = 1,
    Status = 2,
    Backup = 3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatAuth {
    pub chat_id: i64,
    pub perms: Vec<Permission>,
}

impl KvStore {
    pub fn get_chat_auth(chat_id: i64) -> Result<Option<ChatAuth>, Box<dyn Error>> {
        let kv = Self::global();
        let rtxn = kv.env.read_txn()?;
        let value: Option<ChatAuth> = kv.chat_auth.get(&rtxn, &chat_id)?;
        Ok(value)
    }

    pub fn set_chat_auth(chat_id: i64, value: ChatAuth) -> Result<(), Box<dyn Error>> {
        let kv = Self::global();
        let mut wtxn = kv.env.write_txn()?;
        kv.chat_auth.put(&mut wtxn, &chat_id, &value)?;
        wtxn.commit()?;
        Ok(())
    }

    pub fn has_perm(chat_id: i64, perm: Permission) -> Result<bool, Box<dyn Error>> {
        let kv = Self::global();
        let rtxn = kv.env.read_txn()?;
        let value: Option<ChatAuth> = kv.chat_auth.get(&rtxn, &chat_id)?;
        Ok(value.map(|ca| ca.perms.contains(&perm)).unwrap_or(false))
    }
}
