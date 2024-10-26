use alloc::vec::Vec;
use core::iter;
use smallvec::SmallVec;

pub trait Hash {
    fn hash(&self) -> usize;
}

impl<T: AsRef<str>> Hash for T {
    fn hash(&self) -> usize {
        const LEN_USIZE: usize = size_of::<usize>();

        let mut bytes = self.as_ref()
            .bytes()
            .chain(iter::repeat(0x00))
            .array_chunks::<LEN_USIZE>();

        let mut index = bytes.next().unwrap_or_default();

        for (a, mut i) in bytes.enumerate() {
            i.rotate_left(a % LEN_USIZE);

            let bytes = index.into_iter()
                .zip(i.into_iter())
                .map(|(i, j)| i ^ j)
                .collect::<SmallVec<_, LEN_USIZE>>();

            
        }

        usize::from_be_bytes(index)
    }
}

/// This hashmap uses a custom algorithm which assumes the keys used are relatively short
pub struct HashMap<K: Hash, V> {
    buckets: Vec<Vec<(K, V)>>,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> HashMap<K, V> {
        HashMap {
            buckets: Vec::new()
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {}
}