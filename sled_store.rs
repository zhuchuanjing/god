use ldk_node::lightning::util::persist::KVStore;

use sled::{Db, Tree};
use std::io::{Error, ErrorKind};
pub struct SledTree {
    tree: Tree,
}

impl SledTree {
    pub fn new(db: &Db, name: &str)-> Self {
        Self{tree: db.open_tree(name).unwrap()}
    }
    pub fn read(&self, key: &str) -> Result<Vec<u8>, std::io::Error> {
        match self.tree.get(key) {
            Ok(Some(v))=> Ok(v.to_vec()),
            Ok(None)=> Err(std::io::Error::new(std::io::ErrorKind::NotFound, key)),
            _=> Err(Error::new(ErrorKind::Other, "read error!"))
        }
    }
    pub fn write(&self, key: &str, buf: &[u8]) -> Result<(), std::io::Error> {
        match self.tree.insert(key, buf) {
            Ok(_)=> Ok(()),
            _=>  Err(Error::new(ErrorKind::Other, "write error!"))
        }
    }
    pub fn remove(&self, key: &str, lazy: bool) -> Result<(), std::io::Error> {
        match self.tree.remove(key) {
            Ok(_)=> {
                if !lazy { self.tree.flush().unwrap(); }
                Ok(())
            }
            _=> Err(Error::new(ErrorKind::Other, "remove error!"))
        }
    }
    pub fn list(&self) -> Result<Vec<String>, std::io::Error> {
        Ok(self.tree.iter().filter_map(|kv| kv.ok()).filter_map(|kv| String::from_utf8(kv.0.to_vec()).ok() ).collect())
    }
}

use std::sync::{RwLock, Arc};
use std::collections::BTreeMap;
pub struct SledRoot {
    db: Db,
    trees: BTreeMap<String, Arc<SledTree>>,
}

impl SledRoot {
    pub fn new(path: &str)-> Self {
        let db = sled::open(path).unwrap();
        Self{db, trees: BTreeMap::new()}
    }

    pub fn get(&mut self, dir: &str)-> std::sync::RwLock<Arc<SledTree>> {
        if !self.trees.contains_key(dir) {
            let tree: SledTree = SledTree::new(&self.db, dir);
            self.trees.insert(dir.into(), Arc::new(tree));
        }
        RwLock::new(self.trees.get(dir).unwrap().clone())
    }
}

use once_cell::sync::OnceCell;
pub static SLED: OnceCell<RwLock<SledRoot>> = OnceCell::new();

pub fn get_sled_tree(dir: &str)-> std::sync::RwLock<Arc<SledTree>> {
    SLED.get().unwrap().write().unwrap().get(dir)
}

pub struct SledStore{
    dir: String
}
unsafe impl Send for SledStore{}
unsafe impl Sync for SledStore{}

impl SledStore {
    pub fn get_dir(&self, primary_namespace: &str, secondary_namespace: &str)-> String {
        format!("{}/{}/{}/", self.dir, primary_namespace, secondary_namespace)
    }
    pub fn new(dir: &str)-> Self {
        Self{dir: dir.into()}
    }
}

impl KVStore for SledStore {
    fn read(&self, primary_namespace: &str, secondary_namespace: &str, key: &str) -> Result<Vec<u8>, std::io::Error> {
        get_sled_tree(&self.get_dir(primary_namespace, secondary_namespace)).read().unwrap().read(key)
    }

    fn write(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, buf: &[u8]) -> Result<(), std::io::Error> {
        let sled_tree = get_sled_tree(&self.get_dir(primary_namespace, secondary_namespace));
        let tree = sled_tree.write().unwrap();
        let _ = tree.write(key, buf);
        let _ = tree.tree.flush();
        Ok(())
    }

    fn remove(&self, primary_namespace: &str, secondary_namespace: &str, key: &str, lazy: bool) -> Result<(), std::io::Error> {
        get_sled_tree(&self.get_dir(primary_namespace, secondary_namespace)).write().unwrap().remove(key, lazy)
    }

    fn list(&self, primary_namespace: &str, secondary_namespace: &str) -> Result<Vec<String>, std::io::Error> {
        get_sled_tree(&self.get_dir(primary_namespace, secondary_namespace)).read().unwrap().list()
    }
}
