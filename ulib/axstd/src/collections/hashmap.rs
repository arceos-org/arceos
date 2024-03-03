extern crate alloc;

use core::hash::Hasher;
use core::hash::Hash;

const N: usize = 128;

struct Entry<K, V> {
    key: K,
    value: V,
}

pub struct RandomState;


impl Clone for RandomState {
    fn clone(&self) -> Self {
        RandomState
    }
}

impl Default for RandomState {
    fn default() -> Self {
        RandomState
    }
}

pub struct SimpleHasher(u64);

impl core::hash::Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
        }
    }
}

impl core::hash::BuildHasher for RandomState {
    type Hasher = SimpleHasher;
    fn build_hasher(&self) -> Self::Hasher {
        SimpleHasher(0)
    }
}

pub type HashMap<K, V> = HashMapWithHasher<K, V, RandomState>;

pub struct HashMapWithHasher<K, V, S = RandomState> {
    storage: [Option<Entry<K, V>>; N],
    hash_builder: S,
}

pub struct HashMapIter<'a, K, V, S> {
    hashmap: &'a HashMapWithHasher<K, V, S>,
    index: usize,
}

impl<'a, K, V, S> Iterator for HashMapIter<'a, K, V, S>
where
    K: PartialEq,
    S: core::hash::BuildHasher,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N {
            if let Some(entry) = &self.hashmap.storage[self.index] {
                self.index += 1;
                return Some((&entry.key, &entry.value));
            } else {
                self.index += 1;
            }
        }
        None
    }
}

impl<K, V, S> HashMapWithHasher<K, V, S>
where
    K: PartialEq + Hash,
    S: core::hash::BuildHasher + Default + Clone,
{
    const ARRAY_REPEAT_VALUE: Option<Entry<K, V>> = None;

    pub fn new() -> Self {
        HashMapWithHasher {
            storage: [Self::ARRAY_REPEAT_VALUE; N],
            hash_builder: S::default(),
        }
    }

    pub fn with_hasher(hash_builder: S) -> Self {
        HashMapWithHasher {
            storage: [Self::ARRAY_REPEAT_VALUE; N],
            hash_builder,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let index = self.hash(&key) % N;
        for i in 0..N {
            let idx = (index + i) % N;
            match &mut self.storage[idx] {
                Some(entry) if entry.key == key => {
                    let old_value = core::mem::replace(&mut entry.value, value);
                    return Some(old_value);
                }
                None => {
                    self.storage[idx] = Some(Entry { key, value });
                    return None;
                }
                _ => continue,
            }
        }
        None
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let index = self.hash(&key) % N;
        for i in 0..N {
            let idx = (index + i) % N;
            match &self.storage[idx] {
                Some(entry) if entry.key == key => return Some(&entry.value),
                None => return None,
                _ => continue,
            }
        }
        None
    }

    fn hash(&self, key: &K) -> usize {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        hasher.finish() as usize % N
    }

    pub fn iter(&self) -> HashMapIter<K, V, S> {
        HashMapIter {
            hashmap: self,
            index: 0,
        }
    }
}