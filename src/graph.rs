pub trait Graph {
    type K: Clone + Eq;
    type V;

    fn find_node(&self, k: &Self::K) -> Option<&Self::V>;
    fn find_edges_direct(&self, v: &Self::V) -> Vec<Self::K>;
    fn find_edges_ordered_by_depth(&self, root_key: &Self::K) -> Vec<Self::K> {
        let mut back = vec![];
        back.push(root_key.clone());
        let mut visited = 0;
        while visited < back.len() {
            let k = back.get(visited).expect("should be present");
            if let Some(v) = self.find_node(k) {
                for child in self.find_edges_direct(v) {
                    if !back.contains(&child) {
                        back.push(child.clone())
                    }
                }
            }
            visited += 1;
        }
        back
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;
    use std::collections::BTreeMap;

    struct MyGraph {
        datas: BTreeMap<String, Vec<String>>,
    }

    impl Graph for MyGraph {
        type K = String;
        type V = Vec<String>;
        fn find_node(&self, k: &Self::K) -> Option<&Self::V> {
            self.datas.get(k)
        }
        fn find_edges_direct(&self, v: &Self::V) -> Vec<Self::K> {
            v.clone()
        }
    }
    #[test]
    fn test_find_edges_ordered_by_depth() {
        let mut datas = BTreeMap::new();
        datas.insert("k1".to_owned(), vec!["k1.1".to_owned(), "k1.2".to_owned()]);
        datas.insert(
            "k1.1".to_owned(),
            vec![
                "k1.1.1".to_owned(),
                "k1.1.2".to_owned(),
                "k1.1.3".to_owned(),
            ],
        );
        datas.insert(
            "k1.2".to_owned(),
            vec![
                "k1.2.1".to_owned(),
                "k1.2.2".to_owned(),
                "k1.2.2".to_owned(),
            ],
        );
        datas.insert("k1.1.2".to_owned(), vec!["k1.1.2.1".to_owned()]);
        let g = MyGraph { datas };
        let expected = vec![
            "k1".to_owned(),
            "k1.1".to_owned(),
            "k1.2".to_owned(),
            "k1.1.1".to_owned(),
            "k1.1.2".to_owned(),
            "k1.1.3".to_owned(),
            "k1.2.1".to_owned(),
            "k1.2.2".to_owned(),
            "k1.1.2.1".to_owned(),
        ];
        assert_that!(&(g.find_edges_ordered_by_depth(&"k1".to_owned()))).is_equal_to(&expected);
    }

}
