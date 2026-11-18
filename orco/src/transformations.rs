mod monomorphization;
mod name_anonymous;

/// Generates a new unique ID that can be a key to the map.
fn create_id<K, V>(map: &std::collections::HashMap<K, V>) -> K
where
    K: Copy + std::hash::Hash + Eq + From<usize>,
{
    // Uh-oh...
    let mut id = 0;
    for key in map.keys().take(100) {
        use std::hash::BuildHasher;
        id ^= map.hasher().hash_one(key);
    }

    loop {
        let key = K::from(id as _);
        if !map.contains_key(&key) {
            return key;
        }

        id += 1;
    }
}
