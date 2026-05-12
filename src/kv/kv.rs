use crate::{
    base::config::Config,
    kv::{bot::ChatAuth, meta::MetaValue},
};
use heed::{
    Database, Env, EnvFlags, EnvOpenOptions, RwTxn, WithoutTls,
    byteorder::BE,
    types::{I64, SerdeBincode, Str, U8},
};
use std::{error::Error, fs, sync::OnceLock};

pub struct KvStore {
    pub env: Env<WithoutTls>,
    pub meta: Database<U8, SerdeBincode<MetaValue>>,
    pub chat_auth: Database<I64<BE>, SerdeBincode<ChatAuth>>,
    pub backup_ts: Database<Str, SerdeBincode<i64>>,
}

static KV_STORE: OnceLock<KvStore> = OnceLock::new();

impl KvStore {
    pub fn global() -> &'static Self {
        KV_STORE.get_or_init(|| {
            Self::open(3, 32).unwrap_or_else(|err| panic!("Failed to open KV store: {err}"))
        })
    }

    fn open(db_count: u32, size_mb: usize) -> Result<Self, Box<dyn Error>> {
        let config = Config::get();

        // Ensure the KV directory exists
        if !config.kv_path.exists() {
            fs::create_dir_all(&config.kv_path)?;
        }

        // Open the LMDB environment
        let env = unsafe {
            EnvOpenOptions::new()
                .read_txn_without_tls()
                .map_size(size_mb * 1024 * 1024)
                .max_dbs(db_count)
                .flags(EnvFlags::NO_SYNC | EnvFlags::NO_META_SYNC)
                .open(&config.kv_path)?
        };

        // Create databases
        let mut wtxn = env.write_txn()?;
        let meta = Self::create_db(&env, &mut wtxn, "meta")?;
        let chat_auth = Self::create_db(&env, &mut wtxn, "chat_auth")?;
        let backup_ts = Self::create_db(&env, &mut wtxn, "backup_ts")?;
        wtxn.commit()?;

        Ok(Self {
            env,
            meta,
            chat_auth,
            backup_ts,
        })
    }

    fn create_db<K: 'static, V: 'static>(
        env: &Env<WithoutTls>,
        wtxn: &mut RwTxn,
        name: &str,
    ) -> Result<Database<K, V>, Box<dyn Error>> {
        let db = env
            .database_options()
            .name(name)
            .types::<K, V>() // Указываем типы здесь
            .create(wtxn)?;
        Ok(db)
    }
}
