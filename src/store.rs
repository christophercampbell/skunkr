use std::fs;
use std::path::Path;
use std::time::Duration;
use libmdbx::*;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

pub trait KeyValueStore {
    fn get(&self, table: Option<String>, key: Vec<u8>) -> Option<Vec<u8>>;
    fn set(&self, table: Option<String>, key: Vec<u8>, value: Vec<u8>) -> bool;
    fn scan(&self, from: Receiver<(Option<String>, Option<Vec<u8>>)>, results: Sender<(Vec<u8>, Vec<u8>)>);
}

pub const SEND_TIMEOUT: Duration = Duration::new(3, 0);
pub const MAX_TABLES: usize  = 10;

#[derive(Debug)]
pub struct MdbxStore {
    db: std::sync::Arc<Database<NoWriteMap>>, // does there need to be a WriteMap? tests are ambiguous
}

impl MdbxStore {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).expect("Could not create data path");
        MdbxStore {
            db: std::sync::Arc::new(
                Database::new()
                    .set_max_tables(MAX_TABLES)
                    .open(Path::new(path))
                    .unwrap()
            )
        }
    }
}

/// READ_WRITE create table if does not exist
fn ensure_table<'a>(tx: &'a Transaction<'a, RW,NoWriteMap>, table_name:Option<&str>) {
    //let table_name = table_name.as_ref().map(|x| &**x);
    if tx.open_table(table_name).is_err() {
        match tx.create_table(table_name, Default::default()) {
            Ok(_) => log::info!("created table {}", table_name.unwrap()),
            Err(e) => log::error!("error creating table: {}", e)
        }
    }
}

/// READ_ONLY check if table exists
fn table_exists(tx: &Transaction<RO, NoWriteMap>, table_name:Option<&str>) -> bool {
    return table_name.is_some() && !tx.open_table(table_name).is_err();
}

impl KeyValueStore for MdbxStore {
    fn get(&self, table_name: Option<String>, key: Vec<u8>) -> Option<Vec<u8>> {
        let ro = self.db.begin_ro_txn().unwrap();
        let table_name = table_name.as_ref().map(|x| &**x);
        if !table_exists(&ro, table_name) {
            log::warn!("table {:?} does not exist", table_name);
            return None;
        }
        let table = ro.open_table(table_name).unwrap();
        match ro.get::<Vec<u8>>(&table, key.as_ref()).unwrap() {
            Some(value_bytes) => {
                Some(value_bytes)
            }
            _ => None,
        }
    }

    fn set(&self, table_name: Option<String>, key: Vec<u8>, value: Vec<u8>) -> bool {
        let rw = self.db.begin_rw_txn().unwrap();
        let table_name = table_name.as_ref().map(|x| &**x);

        ensure_table(&rw, table_name);
        let table = rw.open_table(table_name).unwrap();

        match rw.put(&table, key, value, WriteFlags::UPSERT) {
            Ok(_) => {
                rw.commit().unwrap();
                true
            }
            _ => false,
        }
    }

    fn scan(&self, mut receiver: Receiver<(Option<String>, Option<Vec<u8>>)>, results: Sender<(Vec<u8>, Vec<u8>)>) {

        let database = self.db.clone();

        tokio::spawn(async move {
            let (maybe_table, from) = receiver.try_recv().unwrap();

            let ro = database.begin_ro_txn().unwrap();
            let table_name = maybe_table.as_ref().map(|x| &**x);

            if !table_exists(&ro, table_name) {
                log::warn!("table {:?} does not exist, cannot scan", table_name);
                return;
            }

            let table = ro.open_table(table_name).unwrap();
            let mut cursor = ro.cursor(&table).unwrap();

            let mut iterator = match from.is_some() {
                true => cursor.iter_from::<Vec<u8>, Vec<u8>>(from.unwrap().as_slice()),
                false => cursor.iter_start::<Vec<u8>, Vec<u8>>()
            };

            while let Some(row) = iterator.next() {
                match row {
                    Ok((key_bytes, value_bytes)) => {
                        // results blocks when full, if client does not read, it will timeout
                        if let Err(e) = results.send_timeout((key_bytes, value_bytes), SEND_TIMEOUT).await {
                            log::error!("send error: #{:?}", e);
                            break;
                        }
                    }
                    _ => break
                }
            }
            drop(results);
        });
    }
}

/*
pub struct LevelDbStore {
    // Add any necessary fields specific to the LevelDB implementation
}

impl KeyValueStore for LevelDbStore {
    fn get(&self, key: &str) -> Option<String> {
        // Implement the get method for the LevelDB store
        // Retrieve the value associated with the key from the LevelDB database
        // Return Some(value) if found, otherwise return None
        None
    }

    fn set(&self, key: &str, value: &str) -> bool {
        // Implement the set method for the LevelDB store
        // Set the value associated with the key in the LevelDB database
        // Return true if the operation is successful, otherwise return false
        false
    }


    fn scan(&self, start_key: Option<&str>) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)>> {
        Box::new(iter::empty())
    }
}
*/
