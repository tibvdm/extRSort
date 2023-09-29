#[derive(PartialEq, Eq, Debug)]
pub struct Leaf<T: Ord> {
    pub value: Option<T>
}

impl<T: Ord> Leaf<T> {
    pub fn new(value: T) -> Self {
        Self { value: Some(value) }
    }

    pub fn take_value(&mut self) -> Option<T> {
        self.value.take()
    }

    pub fn fill_value(&mut self, value: T) {
        self.value = Some(value);
    }
}

impl<T: Ord> Default for Leaf<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T: Ord> PartialOrd for Leaf<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Leaf<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.value, &other.value) {
            (Some(value1), Some(value2)) => value1.cmp(value2),
            (Some(_), None)              => std::cmp::Ordering::Greater,
            (None, Some(_))              => std::cmp::Ordering::Less,
            (None, None)                 => std::cmp::Ordering::Equal
        }
    }
}

pub struct WinnerHeap<T: Ord> {
    pub internal: Vec<Option<usize>>,
    pub internal_size: usize,

    pub leaves: Vec<Leaf<T>>,

    pub last_pop: Option<usize>
}

impl<T: Ord> WinnerHeap<T> {
    pub fn new(items: Vec<T>) -> Self {
        if items.len() == 0 {
            return Self::default();
        }

        let leaves: Vec<Leaf<T>> = items.into_iter().map(Leaf::new).collect();

        let mut internal = init_internal_structure(leaves.len());

        // Keep track of the amount of internal nodes.
        let internal_size = internal.len();

        let mut lower_bound = internal_size / 2;
        let mut upper_bound = internal_size;

        // Populate the lowest depth of the internal structure.
        for i in lower_bound..upper_bound {
            let left_index = 2 * i + 1 - internal_size;
            let right_index = 2 * i + 2 - internal_size;

            let left_leaf = leaves.get(left_index);
            let right_leaf = leaves.get(right_index);

            internal[i] = match (left_leaf, right_leaf) {
                (Some(leaf1), Some(leaf2)) => {
                    if leaf1 > leaf2 { Some(left_index) } else { Some(right_index) }
                },
                (Some(_), None)            => Some(left_index),
                (None, Some(_))            => Some(right_index),
                (None, None)               => None
            };
        }

        // Keep track of the boundaries on each level of the internal structure.
        upper_bound = lower_bound;
        lower_bound = lower_bound / 2;

        // Populate the other levels of the internal structure.
        while upper_bound > 0 {
            for i in lower_bound..upper_bound {
                let left = internal[2 * i + 1];
                let right = internal[2 * i + 2];

                internal[i] = match (left, right) {
                    (Some(leaf1), Some(leaf2)) => {
                        if leaves[leaf1] > leaves[leaf2] { left } else { right }
                    },
                    (Some(_), None)            => left,
                    (None, Some(_))            => right,
                    (None, None)               => None
                };
            }

            upper_bound = lower_bound;
            lower_bound = lower_bound / 2;
        }

        Self { internal, internal_size, leaves, last_pop: None }
    }

    pub fn peek(&self) -> Option<&T> {
        if let Some(pos) = self.internal[0] {
            return self.leaves[pos].value.as_ref();
        }

        None
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(pos) = self.last_pop {
            self.update(pos);
        }

        if let Some(pos) = self.internal[0] {
            self.last_pop = Some(pos);
            return self.leaves[pos].take_value();
        }

        None
    }

    pub fn push(&mut self, value: T) {
        let leaf_position = self.last_pop.expect("You cannot push to an already empty tree");

        self.leaves[leaf_position].fill_value(value);

        self.update(leaf_position);
    }

    pub fn pop_push(&mut self, value: T) -> Option<T> {
        if let Some(leaf_position) = self.internal[0] {
            if let Some(leaf) = self.leaves[leaf_position].take_value() {
                if leaf <= value {
                    self.leaves[leaf_position].fill_value(value);
                } else {
                    self.update(leaf_position);
                }
    
                return Some(leaf);
            }
        }

        None
    }

    fn update(&mut self, position: usize) {
        let mut parent = (position + self.internal_size - 1) / 2;

        let left_index = 2 * parent + 1 - self.internal_size;
        let right_index = 2 * parent + 2 - self.internal_size;

        let left = self.leaves.get(left_index);
        let right = self.leaves.get(right_index);

        self.internal[parent] = match (left, right) {
            (Some(leaf1), Some(leaf2)) => Some({
                if leaf1 > leaf2 { left_index } else { right_index }
            }),
            (Some(_), None)            => Some(left_index),
            (None, Some(_))            => Some(right_index),
            (None, None)               => None
        };

        while parent > 0 {
            parent = (parent - 1) / 2;

            let left = self.internal[2 * parent + 1];
            let right = self.internal[2 * parent + 2];

            self.internal[parent] = match (left, right) {
                (Some(leaf1), Some(leaf2)) => {
                    if self.leaves[leaf1] > self.leaves[leaf2] { left } else { right }
                },
                (Some(_), None)            => left,
                (None, Some(_))            => right,
                (None, None)               => None
            };
        }

        self.last_pop = None;
    }
}

impl<T: Ord> Default for WinnerHeap<T> {
    fn default() -> Self {
        WinnerHeap {
            internal: Vec::new(),
            internal_size: 0,
            leaves: Vec::new(),
            last_pop: None
        }
    }
}

pub fn init_internal_structure(amount_of_items: usize) -> Vec<Option<usize>> {
    match amount_of_items {
        0 => vec![],
        1 => vec![None],
        n => vec![None; n.next_power_of_two() as usize - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_new(items: Vec<u32>, expected_internal: Vec<Option<usize>>, expected_leaves: Vec<Leaf<u32>>) {
        let winner_tree = WinnerHeap::new(items);

        assert_eq!(winner_tree.internal, expected_internal);
        assert_eq!(winner_tree.leaves, expected_leaves);
        assert_eq!(winner_tree.last_pop, None);
    }

    #[test]
    fn test_new_0() {
        test_new(
            vec![], 
            vec![], 
            vec![]
        );
    }

    #[test]
    fn test_new_1() {
        test_new(
            vec![ 4 ], 
            vec![ Some(0) ], 
            vec![ Leaf::new(4) ]
        );
    }

    #[test]
    fn test_new_2() {
        test_new(
            vec![ 4, 7 ], 
            vec![ Some(1) ], 
            vec![ Leaf::new(4), Leaf::new(7) ]
        );
    }

    #[test]
    fn test_new_3() {
        test_new(
            vec![ 4, 7, 2 ], 
            vec![ Some(1), Some(1), Some(2) ], 
            vec![ Leaf::new(4), Leaf::new(7), Leaf::new(2) ]
        );
    }

    #[test]
    fn test_new_4() {
        test_new(
            vec![ 4, 7, 2, 8 ], 
            vec![ Some(3), Some(1), Some(3) ], 
            vec![ Leaf::new(4), Leaf::new(7), Leaf::new(2), Leaf::new(8) ]
        );
    }

    #[test]
    fn test_new_5() {
        test_new(
            vec![ 4, 7, 2, 8, 13 ], 
            vec![ Some(4), Some(3), Some(4), Some(1), Some(3), Some(4), None ],
            vec![ 
                Leaf::new(4), Leaf::new(7), Leaf::new(2), Leaf::new(8), 
                Leaf::new(13) 
            ]
        );
    }

    #[test]
    fn test_new_6() {
        test_new(
            vec![ 4, 7, 2, 8, 13, 1 ], 
            vec![ Some(4), Some(3), Some(4), Some(1), Some(3), Some(4), None ],
            vec![ 
                Leaf::new(4), Leaf::new(7), Leaf::new(2), Leaf::new(8), 
                Leaf::new(13), Leaf::new(1)
            ]
        );
    }

    #[test]
    fn test_new_7() {
        test_new(
            vec![ 4, 7, 2, 8, 13, 1, 5 ], 
            vec![ Some(4), Some(3), Some(4), Some(1), Some(3), Some(4), Some(6) ],
            vec![ 
                Leaf::new(4), Leaf::new(7), Leaf::new(2), Leaf::new(8), 
                Leaf::new(13), Leaf::new(1), Leaf::new(5)
            ]
        );
    }

    #[test]
    fn test_new_8() {
        test_new(
            vec![ 4, 7, 2, 8, 13, 1, 5, 23 ], 
            vec![ Some(7), Some(3), Some(7), Some(1), Some(3), Some(4), Some(7) ],
            vec![ 
                Leaf::new(4), Leaf::new(7), Leaf::new(2), Leaf::new(8), 
                Leaf::new(13), Leaf::new(1), Leaf::new(5), Leaf::new(23)
            ]
        );
    }

    // #[test]
    // fn test_pop_0() {
    //     let mut winner_tree = WinnerHeap::<u32>::new(vec![]);
    //     assert_eq!(winner_tree.pop(), None);
    // }

    #[test]
    fn test_pop_1() {
        let mut winner_tree = WinnerHeap::new(vec![ 4 ]);

        assert_eq!(winner_tree.pop(), Some(4));
        assert_eq!(winner_tree.pop(), None);
    }

    #[test]
    fn test_pop_2() {
        let mut winner_tree = WinnerHeap::new(vec![ 4, 2 ]);
        
        assert_eq!(winner_tree.pop(), Some(4));
        assert_eq!(winner_tree.pop(), Some(2));
        assert_eq!(winner_tree.pop(), None);
    }

    #[test]
    fn test_pop_8() {
        let mut winner_tree = WinnerHeap::new(vec![ 4, 7, 2, 8, 13, 1, 5, 23 ]);

        assert_eq!(winner_tree.pop(), Some(23));
        assert_eq!(winner_tree.pop(), Some(13));
        assert_eq!(winner_tree.pop(), Some(8));
        assert_eq!(winner_tree.pop(), Some(7));
        assert_eq!(winner_tree.pop(), Some(5));
        assert_eq!(winner_tree.pop(), Some(4));
        assert_eq!(winner_tree.pop(), Some(2));
        assert_eq!(winner_tree.pop(), Some(1));

        assert_eq!(winner_tree.pop(), None);
    }

    #[test]
    #[should_panic]
    fn test_push_empty_0() {
        let mut winner_tree = WinnerHeap::<u32>::new(vec![]);

        winner_tree.push(4);
    }

    // #[test]
    // #[should_panic]
    // fn test_push_empty_1() {
    //     let mut winner_tree = WinnerHeap::new(vec![ 4 ]);

    //     winner_tree.pop();
    //     winner_tree.pop();
    //     winner_tree.push(4);
    // }

    #[test]
    fn test_push_pop() {
        let mut winner_tree = WinnerHeap::new(vec![ 4, 7, 2 ]);

        assert_eq!(winner_tree.pop(), Some(7));
        winner_tree.push(13);
        assert_eq!(winner_tree.pop(), Some(13));
        assert_eq!(winner_tree.pop(), Some(4));
        winner_tree.push(1);
        assert_eq!(winner_tree.pop(), Some(2));
        assert_eq!(winner_tree.pop(), Some(1));
        winner_tree.push(5);
        assert_eq!(winner_tree.pop(), Some(5));
        assert_eq!(winner_tree.pop(), None);
    }

    #[test]
    fn test_pop_push() {
        let mut winner_tree = WinnerHeap::new(vec![ 4, 7, 2 ]);

        assert_eq!(winner_tree.pop_push(14), Some(7));
        assert_eq!(winner_tree.pop_push(3), Some(14));
        assert_eq!(winner_tree.pop_push(4), Some(4));
    }
}
