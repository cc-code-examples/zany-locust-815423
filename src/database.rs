use std::{collections::HashMap, sync::{Arc, Mutex}, time::Instant};

#[derive(Clone, Default)]
pub struct DatabaseRef {
    map: Arc<Mutex<HashMap<String, Entry>>>,
    lists: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

pub struct Entry {
    value: String,
    expire_at: Option<std::time::Instant>,
}

impl DatabaseRef {

    pub fn new() -> Self {
        DatabaseRef { 
            map: Arc::new(Mutex::new(HashMap::new())),
            lists: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&self, key: String, value: String, expire_at: Option<Instant>) -> anyhow::Result<()> {
        match self.map.lock() {
            Ok(mut map) => {
                map.insert(key, Entry { value, expire_at });
                Ok(())
            },
            Err(err) => todo!("{}", err),
        }
    }

    pub fn get(&self, key: String) -> Option<String> {
        match self.map.lock() {
            Ok(map) => {
                if let Some(entry) = map.get(&key) {
                    if let Some(expire_at) = entry.expire_at && Instant::now() > expire_at {
                        return None;
                    }
                    Some(entry.value.clone())
                } else {
                    None
                }
            },
            Err(err) => todo!("{}", err),
        }
    }



    pub fn append_to_list(&self, key: String, value: String) -> anyhow::Result<usize> {
        match self.lists.lock() {
            Ok(mut lists) => {
                if let Some(list) = lists.get_mut(&key) {
                    list.push(value);
                    Ok(list.len())
                } else {
                    lists.insert(key, vec![value]);
                    Ok(1)
                }
            },
            Err(err) => todo!("{}", err),
        }
    }

    pub fn prepend_to_list(&self, key: String, value: String) -> anyhow::Result<usize> {
        match self.lists.lock() {
            Ok(mut lists) => {
                if let Some(list) = lists.get_mut(&key) {
                    list.insert(0, value);
                    Ok(list.len())
                } else {
                    lists.insert(key, vec![value]);
                    Ok(1)
                }
            },
            Err(err) => todo!("{}", err),
        }
    }


    pub fn list_range(&self, key: String, start: isize, stop: isize) -> anyhow::Result<Vec<String>> {
        match self.lists.lock() {
            Ok(lists) => {
                if let Some(list) = lists.get(&key) {
                    let len = list.len() as isize;
                    let start = if start < 0 { len + start } else { start };
                    let stop = if stop < 0 { len + stop } else { stop };

                    let start = start.max(0) as usize;
                    let stop = stop.min(len - 1) as usize;

                    Ok(list[start..=stop].to_vec())
                } else {
                    Ok(vec![])
                }
            },
            Err(err) => todo!("{}", err),
        }
    }

    pub fn list_len(&self, key: String) -> anyhow::Result<usize> {
        match self.lists.lock() {
            Ok(lists) => {
                if let Some(list) = lists.get(&key) {
                    Ok(list.len())
                } else {
                    Ok(0)
                }
            },
            Err(err) => todo!("{}", err),
        }
    }

    pub fn left_pop(&self, key: String) -> anyhow::Result<Option<String>> {
        match self.lists.lock() {
            Ok(mut lists) => {
                if let Some(list) = lists.get_mut(&key) {
                    Ok((!list.is_empty()).then(|| list.remove(0)))
                } else {
                    Ok(None)
                }
            },
            Err(err) => todo!("{}", err),
        }
    }

    pub fn left_pop_multiple(&self, key: String, count: usize) -> anyhow::Result<Vec<String>> {
        match self.lists.lock() {
            Ok(mut lists) => {
                if let Some(list) = lists.get_mut(&key) {
                    let pop_count = count.min(list.len());
                    let mut result = Vec::with_capacity(pop_count);
                    for _ in 0..pop_count {
                        result.push(list.remove(0));
                    }
                    Ok(result)
                } else {
                    Ok(vec![])
                }
            },
            Err(err) => todo!("{}", err),
        }
    }
}