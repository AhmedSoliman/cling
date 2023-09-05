use std::any::{type_name, Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

// Copied/inspired from Axum's extensions.
// With TypeIds as keys, there's no need to hash them. They are already hashes
// themselves, coming from the compiler. The IdHasher just holds the u64 of
// the TypeId, and then returns it.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Holds values of different types and accessed by TypeId.
#[derive(Default)]
pub struct AnyMap {
    // Key is TypeId, value is heap-allocated Box<dyn Any + Send + Sync>
    map: HashMap<
        TypeId,
        Box<dyn Any + Send + Sync>,
        BuildHasherDefault<IdHasher>,
    >,
    known_types: HashSet<String>,
}

impl AnyMap {
    /// Returns a reference to value of type T if exists.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            // downcast_ref returns a reference to the boxed value if it is of
            // type T.
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    /// Returns a mutable reference to value of type T if exists.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map.get_mut(&TypeId::of::<T>()).and_then(|boxed| {
            (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut()
        })
    }

    /// Inserts a value into the collected arguments. If the value already
    /// exists, it will be returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.known_types.insert(type_name::<T>().to_string());
        self.map
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            })
    }

    pub fn known_types(&self) -> Vec<String> {
        self.known_types.iter().cloned().collect()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_anymap() {
        #[derive(Debug, PartialEq)]
        struct MyType(i32);

        let mut map = AnyMap::default();
        assert_eq!(0, map.len());

        assert_eq!(None, map.insert(1u32));
        let x = map.get::<u32>().unwrap();
        assert_eq!(1, *x);

        map.insert("hello".to_string());
        assert_eq!(2, map.len());
        assert_eq!("hello", *map.get::<String>().unwrap());

        assert_eq!(None, map.get::<MyType>());

        map.insert(MyType(42));
        assert_eq!(3, map.len());
        assert_eq!(MyType(42), *map.get::<MyType>().unwrap());

        assert_eq!(MyType(42), map.insert(MyType(43)).unwrap());

        map.clear();

        assert_eq!(0, map.len());
        assert!(map.is_empty());
        assert_eq!(None, map.insert(MyType(43)));
    }
}
