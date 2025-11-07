use anyhow::Result;
use colored::Colorize;
use rocksdb::DB;
use rocksdb::IteratorMode;
use rocksdb::Options;
use rocksdb::SliceTransform;

use crate::display::print_key_value;
use crate::utility::highlight_pattern;
#[derive(Debug)]
pub struct DBHelper {
    pub db: rocksdb::DB,
    pub path: String,
    pub current_cf: String,
    pub cf_list: Vec<String>,
}

impl DBHelper {
    pub fn new(path: &str, readonly: Option<bool>) -> Self {
        let mut db_opts = Options::default();
        let prefix_extractor = SliceTransform::create_fixed_prefix(4);
        db_opts.set_prefix_extractor(prefix_extractor);
        let cf_list = match DB::list_cf(&db_opts, path) {
            Ok(cfs) => cfs,
            Err(e) => {
                eprintln!("Error listing column families: {}", e);
                std::process::exit(1);
            }
        };
        println!("{:?}", cf_list);
        let db;
        if readonly.is_some() && readonly.unwrap() {
            db = DBHelper::new_readonly_db(path, db_opts, &cf_list);
        } else {
            db = DBHelper::new_writable_db(path, &mut db_opts, &cf_list);
        }
        DBHelper {
            db,
            path: path.to_string(),
            current_cf: if cf_list.is_empty() {
                String::new()
            } else {
                cf_list[0].clone()
            },
            cf_list,
        }
    }

    fn new_readonly_db(path: &str, db_opts: Options, cf_list: &Vec<String>) -> rocksdb::DB {
        DB::open_cf_for_read_only(&db_opts, path, cf_list, false).unwrap()
    }

    fn new_writable_db(path: &str, db_opts: &mut Options, cf_list: &Vec<String>) -> rocksdb::DB {
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        DB::open_cf(&db_opts, path, cf_list.iter()).unwrap()
    }

    pub fn get_cfs_names(&self) -> Vec<String> {
        self.cf_list.clone()
    }

    fn get_cf_handle(&self, name: &str) -> Option<&rocksdb::ColumnFamily> {
        self.db.cf_handle(name)
    }

    pub fn get(&self, key: &str, as_json: bool) -> Result<()> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        match self.db.get_cf(cf, key)? {
            Some(value) => {
                if as_json {
                    let value_str = String::from_utf8_lossy(&value);
                    match serde_json::from_str::<String>(&value_str) {
                        Ok(json_val) => print_key_value(key.as_bytes(), json_val.as_bytes()),
                        Err(_) => println!("{}", value_str),
                    }
                } else {
                    print_key_value(key.as_bytes(), &value);
                }
            }
            None => println!("Key not found"),
        }
        Ok(())
    }

    pub fn get_keys(&self, limit: usize) -> Result<Vec<String>> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        let mut keys = Vec::with_capacity(limit);
        let mut iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        while let Some(key_values) = iter.next() {
            match key_values {
                Ok((key, _)) => {
                    keys.push(String::from_utf8_lossy(&key).to_string());
                    if keys.len() >= limit {
                        break;
                    }
                }
                Err(_) => {
                    println!("Error occurred while iterating over keys");
                }
            }
        }
        Ok(keys)
    }

    pub fn put(&self, key: &str, value: &str) -> Result<()> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        self.db.put_cf(cf, key, value)?;
        println!(
            "Successfully put {} {}",
            key.bright_green(),
            value.bright_green()
        );
        Ok(())
    }

    pub fn prefix(
        &self,
        prefix: &str,
        highlight_matched: bool,
    ) -> Result<impl Iterator<Item = (Vec<u8>, Vec<u8>)>> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        let iter = self.db.prefix_iterator_cf(cf, prefix);
        let key_values = iter.filter_map(|kv| kv.ok()).map(move |(key, value)| {
            if highlight_matched {
                let highlighted_key: Vec<u8> = highlight_pattern(prefix, key.into_vec());
                (highlighted_key, value.into_vec())
            } else {
                (key.into_vec(), value.into_vec())
            }
        });
        Ok(key_values)
    }

    pub fn contains_stringkey(&self, key: &str) -> bool {
        if let Some(cf) = self.get_cf_handle(&self.current_cf) {
            if let Ok(result) = self.db.get_pinned_cf(cf, key) {
                if let Some(_) = result { true } else { false }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn search_key(
        &self,
        pattern: &str,
        highlight_matched: bool,
    ) -> Result<impl Iterator<Item = (Vec<u8>, Vec<u8>)>> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        let results = iter
            .filter_map(|item| item.ok())
            .filter(|value| {
                value
                    .0
                    .windows(pattern.len())
                    .any(|window| window == pattern.as_bytes())
            })
            .map(move |(key, value)| {
                if highlight_matched {
                    let highlighted_value = highlight_pattern(pattern, value.into_vec());
                    (key.to_vec(), highlighted_value)
                } else {
                    (key.to_vec(), value.to_vec())
                }
            });
        Ok(results)
    }

    pub fn search_value(
        &self,
        pattern: &str,
        highlight_matched: bool,
    ) -> Result<impl Iterator<Item = (Vec<u8>, Vec<u8>)>> {
        // let mut results = Vec::with_capacity(limit);
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        let results = iter
            .filter_map(|item| item.ok())
            .filter(|value| {
                value
                    .1
                    .windows(pattern.len())
                    .any(|window| window == pattern.as_bytes())
            })
            .map(move |(key, value)| {
                if highlight_matched {
                    let highlighted_value = highlight_pattern(pattern, value.into_vec());
                    (key.to_vec(), highlighted_value)
                } else {
                    (key.to_vec(), value.to_vec())
                }
            });
        Ok(results)
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        match self.db.delete_cf(cf, key) {
            Ok(_) => println!("Key deleted successfully"),
            Err(_) => println!("Key not found"),
        }
        Ok(())
    }

    pub fn scan(
        &self,
        start: Option<&str>,
        end: Option<&str>,
        reverse: bool,
    ) -> Result<impl Iterator<Item = (Vec<u8>, Vec<u8>)>> {
        let cf = self.get_cf_handle(&self.current_cf).unwrap();
        let mut options = rocksdb::ReadOptions::default();
        if let Some(start) = start {
            options.set_iterate_lower_bound(start.as_bytes());
        }
        if let Some(end) = end {
            options.set_iterate_upper_bound(end.as_bytes());
        }
        let iter = self.db.iterator_cf_opt(
            cf,
            options,
            if reverse {
                IteratorMode::End
            } else {
                IteratorMode::Start
            },
        );
        let key_values = iter
            .filter_map(|kv| kv.ok())
            .map(|(key, value)| (key.into(), value.into()));
        Ok(key_values)
    }
}
