use std::collections::HashMap;

pub struct Interner<Id> {
    name_to_id: HashMap<Box<str>, Id>,
    id_to_name: Vec<Box<str>>,
}

impl<Id> Default for Interner<Id> {
    fn default() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
        }
    }
}

impl<Id> Interner<Id>
where
    Id: From<u32> + Into<u32> + Copy,
{
    pub fn intern(&mut self, name: &str) -> Id {
        if let Some(id) = self.lookup(name) {
            return id;
        }
        let id = (self.id_to_name.len() as u32).into();
        let name: Box<str> = name.into();
        self.id_to_name.push(name.clone());
        self.name_to_id.insert(name, id);
        id
    }
    pub fn lookup(&self, name: &str) -> Option<Id> {
        self.name_to_id.get(name).copied()
    }
    pub fn name(&self, id: Id) -> &str {
        let id: u32 = id.into();
        &self.id_to_name[id as usize]
    }
    pub fn len(&self) -> usize {
        self.id_to_name.len()
    }
    pub fn is_empty(&self) -> bool {
        self.id_to_name.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let interner = Interner::<u32>::default();
        assert_eq!(interner.len(), 0);
        assert_eq!(interner.lookup("/a"), None);
    }

    #[test]
    fn ids_are_sequential_in_arrival_order() {
        let mut interner = Interner::<u32>::default();
        let ids = ["/lidar", "/camera", "/tf", "/map"].map(|n| interner.intern(n));
        assert_eq!(ids, [0, 1, 2, 3]);
        assert_eq!(interner.len(), 4);
    }

    #[test]
    fn same_name_same_id() {
        let mut interner = Interner::<u32>::default();
        let a = interner.intern("/a");
        interner.intern("/b");
        assert_eq!(interner.intern("/a"), a);
        assert_eq!(interner.len(), 2);
        assert_eq!(interner.intern("/c"), 2);
    }

    #[test]
    fn name_round_trips() {
        let mut interner = Interner::<u32>::default();
        for name in ["/a", "/b/c", "", "кадр", "/a/"] {
            let id = interner.intern(name);
            assert_eq!(interner.name(id), name);
        }
    }

    #[test]
    fn lookup_does_not_intern() {
        let mut interner = Interner::<u32>::default();
        interner.intern("/a");
        assert_eq!(interner.lookup("/b"), None);
        assert_eq!(interner.len(), 1);
        assert_eq!(interner.lookup("/a"), Some(0));
        assert_eq!(interner.intern("/b"), 1);
    }

    #[test]
    fn similar_names_are_distinct() {
        let mut interner = Interner::<u32>::default();
        let ids = ["/a", "/A", "/a/", "a", " /a"].map(|n| interner.intern(n));
        assert_eq!(ids, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn newtype_id() {
        #[derive(Clone, Copy, PartialEq, Debug)]
        struct Id(u32);
        impl From<u32> for Id {
            fn from(v: u32) -> Self {
                Self(v)
            }
        }
        impl From<Id> for u32 {
            fn from(id: Id) -> Self {
                id.0
            }
        }

        let mut interner = Interner::<Id>::default();
        assert_eq!(interner.intern("/a"), Id(0));
        assert_eq!(interner.intern("/b"), Id(1));
        assert_eq!(interner.lookup("/a"), Some(Id(0)));
        assert_eq!(interner.name(Id(1)), "/b");
    }

    #[test]
    #[should_panic]
    fn name_of_unknown_id_panics() {
        let mut interner = Interner::<u32>::default();
        interner.intern("/a");
        interner.name(1);
    }
}
