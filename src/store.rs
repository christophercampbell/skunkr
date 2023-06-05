use std::fs;
use std::path::Path;
use std::time::Duration;
use libmdbx::*;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

pub trait KeyValueStore {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str) -> bool;
    fn scan(&self, from: Receiver<Option<String>>, results: Sender<(String, String)>);
}

pub const SEND_TIMEOUT: Duration = Duration::new(3, 0);

#[derive(Debug)]
pub struct MdbxStore {
    db: std::sync::Arc<Database<NoWriteMap>>, // does there need to be a WriteMap? tests are ambiguous
}

impl MdbxStore {
    pub fn new(path: &str) -> Self {
        fs::create_dir_all(path).expect("Could not create data path");
        MdbxStore {
            db: std::sync::Arc::new(Database::new().open(Path::new(path)).unwrap())
        }
    }
}

impl KeyValueStore for MdbxStore {
    fn get(&self, key: &str) -> Option<String> {
        let tx = self.db.begin_ro_txn().unwrap();
        let table = tx.open_table(None).unwrap();
        match tx.get::<Vec<u8>>(&table, key.as_bytes()).unwrap() {
            Some(value_bytes) => {
                let value = String::from_utf8_lossy(value_bytes.as_slice());
                Some(value.into_owned())
            }
            _ => None,
        }
    }

    fn set(&self, key: &str, value: &str) -> bool {
        let tx = self.db.begin_rw_txn().unwrap();
        let table = tx.open_table(None).unwrap();
        match tx.put(&table, key.as_bytes(), value.as_bytes(), WriteFlags::UPSERT) {
            Ok(_) => {
                tx.commit().unwrap();
                true
            }
            _ => false,
        }
    }

    fn scan(&self, mut filter: Receiver<Option<String>>, results: Sender<(String, String)>) {
        let database = self.db.clone();

        tokio::spawn(async move {
            let tx = database.begin_ro_txn().unwrap();
            let table = tx.open_table(None).unwrap();
            let mut cursor = tx.cursor(&table).unwrap();

            let from = filter.try_recv().unwrap();

            let mut iterator = match from.is_some() {
                true => cursor.iter_from::<Vec<u8>, Vec<u8>>(from.unwrap().as_bytes()),
                false => cursor.iter_start::<Vec<u8>, Vec<u8>>()
            };

            while let Some(row) = iterator.next() {
                match row {
                    Ok((key_bytes, value_bytes)) => {
                        let key = String::from_utf8_lossy(&key_bytes).into_owned();
                        let value = String::from_utf8_lossy(&value_bytes).into_owned();
                        // results blocks when full, if client does not read, it will timeout
                        if let Err(e) = results.send_timeout((key, value), SEND_TIMEOUT).await {
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
