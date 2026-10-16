use std::collections::HashMap;

/// Simple intern vec.
#[derive(Clone)]
pub struct InternVec<T> {
    objects: Vec<T>,
    intern: HashMap<T, u32>,
}

impl<T: std::fmt::Debug> std::fmt::Debug for InternVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.objects.fmt(f)
    }
}

impl<T> Default for InternVec<T> {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            intern: HashMap::new(),
        }
    }
}

impl<T: Clone + Eq + std::hash::Hash> InternVec<T> {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an object into the container, returns the ID.
    pub fn insert(&mut self, object: T) -> u32 {
        if let Some(idx) = self.intern.get(&object) {
            return *idx;
        }
        let idx = self.objects.len() as _;
        self.objects.push(object.clone());
        self.intern.insert(object, idx);
        idx
    }

    /// Get an object by ID.
    pub fn get(&self, idx: u32) -> Option<&T> {
        self.objects.get(idx as usize)
    }

    /// Get access to internal object storage
    pub fn slice(&self) -> &[T] {
        &self.objects
    }

    /// Update the values, rebuild the intern index.
    pub fn update<U>(&mut self, callback: impl FnOnce(&mut Vec<T>) -> U) -> U {
        let retval = callback(&mut self.objects);
        self.intern.clear();
        for (idx, obj) in self.objects.iter().enumerate() {
            self.intern.insert(obj.clone(), idx as _);
        }

        retval
    }

    /// Get the number of elements in the collection.
    pub fn size(&self) -> u32 {
        self.objects.len() as _
    }
}

// Traits
impl<T: PartialEq> PartialEq for InternVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.objects == other.objects
    }
}

impl<T: Eq> Eq for InternVec<T> {}

impl<T: PartialOrd> PartialOrd for InternVec<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.objects.partial_cmp(&other.objects)
    }
}

impl<T: Ord> Ord for InternVec<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.objects.cmp(&other.objects)
    }
}
