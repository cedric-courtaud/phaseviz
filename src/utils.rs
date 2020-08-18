use std::collections::btree_set::BTreeSet;

pub fn set_from<A: Ord + Copy, B: AsRef<[A]>> (xs: B) -> BTreeSet<A> {
    let mut ret = BTreeSet::new();

    for x in xs.as_ref() {
        ret.insert(*x);
    }

    ret
}

#[macro_export]
macro_rules! bt_set {
    ($($x:expr),*) => ( $crate::utils::set_from([$($x),*]));
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    #[test]
    fn test_btree_set_macro() {
        let mut s1: BTreeSet<u32> = bt_set!();
        let mut s2: BTreeSet<u32> = BTreeSet::new();

        assert_eq!(s1, s2);
        
        s1 = bt_set![1, 2, 3];
        s2.insert(1);
        s2.insert(2);
        s2.insert(3);
        
        assert_eq!(s1, s2);
    }
}