use std::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
};

pub struct UnorderedPair<T> {
    a: T,
    b: T,
}

impl<T: Ord> UnorderedPair<T> {
    pub fn new(a: T, b: T) -> Self {
        if a < b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }

    pub fn ref_a(&self) -> &T {
        &self.a
    }

    pub fn ref_b(&self) -> &T {
        &self.b
    }
}

impl<T: Clone> Clone for UnorderedPair<T> {
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
            b: self.b.clone(),
        }
    }
}

impl<T: Copy> Copy for UnorderedPair<T> {}

impl<T: Debug> Debug for UnorderedPair<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnorderedPair {{ {:?}, {:?} }}", self.a, self.b)
    }
}

impl<T: Default + Ord> Default for UnorderedPair<T> {
    fn default() -> Self {
        Self::new(T::default(), T::default())
    }
}

impl<T: Ord> From<(T, T)> for UnorderedPair<T> {
    fn from(t: (T, T)) -> Self {
        Self::new(t.0, t.1)
    }
}

impl<T: PartialEq<T>> PartialEq<UnorderedPair<T>> for UnorderedPair<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.a == rhs.a && self.b == rhs.b
    }
}

impl<T: Eq> Eq for UnorderedPair<T> {}

impl<T: PartialOrd> PartialOrd for UnorderedPair<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match self.a.partial_cmp(&rhs.a) {
            Some(Ordering::Equal) => self.b.partial_cmp(&rhs.b),
            v => v,
        }
    }
}

impl<T: Ord> Ord for UnorderedPair<T> {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        match self.a.cmp(&rhs.a) {
            Ordering::Equal => self.b.cmp(&rhs.b),
            v => v,
        }
    }
}

impl<T: Hash> Hash for UnorderedPair<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.a.hash(hasher);
        self.b.hash(hasher);
    }
}
