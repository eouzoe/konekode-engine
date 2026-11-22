use std::fmt;

const BITS: usize = 64;

#[derive(Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Set {
    words: Vec<u64>,
}

impl Set {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, item: usize) {
        let word = item / BITS;
        if self.words.len() <= word {
            self.words.resize(word + 1, 0);
        }
        self.words[word] |= 1_u64 << (item % BITS);
    }

    pub fn remove(&mut self, item: usize) {
        if let Some(word) = self.words.get_mut(item / BITS) {
            *word &= !(1_u64 << (item % BITS));
            self.trim();
        }
    }

    #[must_use]
    pub fn contains(&self, item: usize) -> bool {
        self.words
            .get(item / BITS)
            .is_some_and(|word| word & (1_u64 << (item % BITS)) != 0)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.words
            .iter()
            .map(|word| usize::try_from(word.count_ones()).unwrap_or(BITS))
            .sum()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    #[must_use]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.words
            .iter()
            .enumerate()
            .all(|(at, word)| word & !other.word(at) == 0)
    }

    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let size = self.words.len().max(other.words.len());
        Self {
            words: (0..size).map(|at| self.word(at) | other.word(at)).collect(),
        }
    }

    #[must_use]
    pub fn intersection(&self, other: &Self) -> Self {
        let mut set = Self {
            words: (0..self.words.len())
                .map(|at| self.word(at) & other.word(at))
                .collect(),
        };
        set.trim();
        set
    }

    #[must_use]
    pub fn difference(&self, other: &Self) -> Self {
        let mut set = Self {
            words: (0..self.words.len())
                .map(|at| self.word(at) & !other.word(at))
                .collect(),
        };
        set.trim();
        set
    }

    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.words.iter().enumerate().flat_map(|(at, &word)| {
            (0..BITS)
                .filter(move |bit| word & (1_u64 << bit) != 0)
                .map(move |bit| at * BITS + bit)
        })
    }

    fn word(&self, at: usize) -> u64 {
        self.words.get(at).copied().unwrap_or(0)
    }

    fn trim(&mut self) {
        while self.words.last() == Some(&0) {
            self.words.pop();
        }
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl FromIterator<usize> for Set {
    fn from_iter<I: IntoIterator<Item = usize>>(items: I) -> Self {
        let mut set = Self::new();
        for item in items {
            set.insert(item);
        }
        set
    }
}

#[cfg(kani)]
mod proofs {
    use super::{BITS, Set};

    const ITEMS: usize = 2 * BITS;

    fn any_set() -> Set {
        let mut set = Set {
            words: vec![kani::any(), kani::any()],
        };
        set.trim();
        set
    }

    fn any_item() -> usize {
        let item: usize = kani::any();
        kani::assume(item < ITEMS);
        item
    }

    fn trimmed(set: &Set) -> bool {
        set.words.last() != Some(&0)
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn union_holds_each_side() {
        let (left, right, item) = (any_set(), any_set(), any_item());
        let both = left.union(&right);
        assert!(trimmed(&both));
        assert_eq!(
            both.contains(item),
            left.contains(item) || right.contains(item)
        );
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn intersection_holds_both_sides() {
        let (left, right, item) = (any_set(), any_set(), any_item());
        let common = left.intersection(&right);
        assert!(trimmed(&common));
        assert_eq!(
            common.contains(item),
            left.contains(item) && right.contains(item)
        );
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn difference_holds_the_left_side_only() {
        let (left, right, item) = (any_set(), any_set(), any_item());
        let rest = left.difference(&right);
        assert!(trimmed(&rest));
        assert_eq!(
            rest.contains(item),
            left.contains(item) && !right.contains(item)
        );
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn subset_means_nothing_is_left_over() {
        let (left, right, item) = (any_set(), any_set(), any_item());
        assert_eq!(left.is_subset(&right), left.difference(&right).is_empty());
        if left.is_subset(&right) && left.contains(item) {
            assert!(right.contains(item));
        }
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn equal_members_mean_equal_sets() {
        let (left, right) = (any_set(), any_set());
        if left.is_subset(&right) && right.is_subset(&left) {
            assert!(left == right);
        }
    }

    #[kani::proof]
    #[kani::unwind(4)]
    fn insert_and_remove_touch_one_item() {
        let (start, item, other) = (any_set(), any_item(), any_item());
        let mut grown = start.clone();
        grown.insert(item);
        assert!(trimmed(&grown));
        assert!(grown.contains(item));
        let mut shrunk = start.clone();
        shrunk.remove(item);
        assert!(trimmed(&shrunk));
        assert!(!shrunk.contains(item));
        if other != item {
            assert_eq!(grown.contains(other), start.contains(other));
            assert_eq!(shrunk.contains(other), start.contains(other));
        }
    }
}
