use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct StoreValue {
    value: String,
    expired_at: Option<u128>,
}

impl StoreValue {
    pub fn new(value: String, expired_at: Option<u128>) -> Self {
        StoreValue { value, expired_at }
    }
}

#[derive(Debug, Clone, PartialEq)]

pub struct Store {
    map: HashMap<String, StoreValue>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: String, options: &SetOptions) -> Option<StoreValue> {
        let current_expired_at = match self.map.get(&key) {
            Some(store_value) => store_value.expired_at,
            None => None,
        };

        let expired_at = match options.expire {
            Some(expire) => Some(
                get_unixtime()
                    .checked_add(Duration::from_millis(expire))
                    .unwrap()
                    .as_millis(),
            ),
            None => current_expired_at,
        };
        let store_value = StoreValue { value, expired_at };
        self.map.insert(key, store_value)
    }

    pub fn get(&self, key: String) -> Option<String> {
        match self.map.get(&key) {
            Some(store_value) => {
                match store_value.expired_at {
                    Some(expired_at) => {
                        let current_unixtime = get_unixtime().as_millis();
                        if expired_at < current_unixtime {
                            // TODO: mapから値を削除する
                            //       使う側でRwLockを使用しているため、可変参照にできない
                            //       jobで定期的に期限切れの値を削除する仕様であれば、ここで削除する必要はないかも？
                            return None;
                        } else {
                            return Some(store_value.value.to_owned());
                        }
                    }
                    None => Some(store_value.value.to_owned()),
                };

                Some(store_value.value.to_owned())
            }
            None => None,
        }
    }
}

pub struct SetOptions {
    expire: Option<u64>,
}

impl SetOptions {
    pub fn new() -> Self {
        SetOptions { expire: None }
    }

    pub fn set_expire(&mut self, expire: u64) -> &Self {
        self.expire = Some(expire);
        self
    }
}

fn get_unixtime() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}
