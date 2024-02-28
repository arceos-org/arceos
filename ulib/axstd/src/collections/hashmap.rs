extern crate alloc;

const N: usize = 128;

struct Entry<K, V> {
    key: K,
    value: V,
}

// As a Rust newbie, I found it's very hard to me to port the hash map implementation from std library which contains too many things
// I generated the initial code by ChatGPT, and fixed the code issues with the help of ChatGPT prompts and compiler hints
// I am glad to pass the unit tests in the end
pub struct HashMap<K, V> {
    storage: [Option<Entry<K, V>>; N],
}

pub struct HashMapIter<'a, K, V> {
    hashmap: &'a HashMap<K, V>,
    index: usize,
}

impl<K, V> HashMap<K, V>
where
    K: PartialEq,
{
    const ARRAY_REPEAT_VALUE: Option<Entry<K, V>> = None;

    pub fn new() -> Self {
        HashMap {
            storage: [Self::ARRAY_REPEAT_VALUE; N],
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let index = self.hash(&key) % N;
        for i in 0..N {
            let idx = (index + i) % N;
            match self.storage[idx] {
                Some(ref entry) if entry.key == key => {
                    let old_value =
                        core::mem::replace(&mut self.storage[idx].as_mut().unwrap().value, value);
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
            match self.storage[idx] {
                Some(ref entry) if entry.key == key => return Some(&entry.value),
                None => return None,
                _ => continue,
            }
        }
        None
    }

    // A very simple hash function for demonstration purposes.
    // You should use a better hash function for real applications.
    fn hash(&self, key: &K) -> usize {
        let key_ptr = key as *const K as usize;
        key_ptr % N
    }

    pub fn iter(&self) -> HashMapIter<K, V> {
        HashMapIter {
            hashmap: self,
            index: 0,
        }
    }
}

impl<'a, K, V> Iterator for HashMapIter<'a, K, V>
where
    K: PartialEq,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < N {
            match self.hashmap.storage[self.index] {
                Some(ref entry) => {
                    self.index += 1; // Move to the next index for the next call to next()
                    return Some((&entry.key, &entry.value));
                }
                None => {
                    self.index += 1; // Current index is empty, move to the next one
                }
            }
        }
        None // All entries have been traversed
    }
}
