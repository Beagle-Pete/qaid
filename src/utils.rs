use std::collections::HashMap;
use std::hash::Hash;

pub fn get_duplicates<T: Eq + PartialEq + Hash + Clone>(data: &[T]) -> Vec<T>{
    let mut map: HashMap<T, bool> = HashMap::with_capacity(data.len());

    for el in data.iter().cloned() {
        map.entry(el)
            .and_modify(|e| *e = true)
            .or_insert(false);
    }

    map.into_iter()
        .filter(|el| el.1)
        .map(|el| el.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_duplicates_test() {
        let vec = vec![1, 2, 3];
        assert_eq!(get_duplicates(&vec), []);

        let vec = vec![1, 2, 3, 3];
        assert_eq!(get_duplicates(&vec), [3]);

        let vec = vec![1, 2, 3, 3, 3, 4, 4];
        assert!(get_duplicates(&vec).contains(&3));
        assert!(get_duplicates(&vec).contains(&4));

        let vec = vec!["one", "two", "three", "three", "three", "four"];
        assert_eq!(get_duplicates(&vec), ["three"]);
    }
}