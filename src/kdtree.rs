use std::{cmp::Ordering, fmt::Debug, num::NonZeroUsize};

pub trait KdTreeItem: Debug + Clone {
    fn cmp_in_depth(&self, rhs: &Self, depth: usize) -> Ordering;
}

impl<T: Debug + Clone + PartialOrd, const N: usize> KdTreeItem for [T; N] {
    fn cmp_in_depth(&self, rhs: &Self, depth: usize) -> Ordering {
        self[depth % N]
            .partial_cmp(&rhs[depth % N])
            .expect("not total order")
    }
}

pub struct KdTree<T> {
    nodes: Vec<Node<T>>,
    root_index: Option<NonZeroUsize>,
}

impl<T: KdTreeItem> KdTree<T> {
    pub fn construct(items: impl Into<Vec<T>>) -> KdTree<T> {
        let mut items: Vec<_> = items.into();
        let mut nodes = Vec::with_capacity(items.len());

        let root_index = Self::construct_part(&mut nodes, &mut items, 0);

        KdTree { nodes, root_index }
    }

    pub fn root(&self) -> Option<&T> {
        self.root_index.map(|ip1| &self.nodes[ip1.get() - 1].item)
    }

    fn construct_part(
        nodes: &mut Vec<Node<T>>,
        items: &mut [T],
        depth: usize,
    ) -> Option<NonZeroUsize> {
        match items.len() {
            0 => None,
            1 => {
                let index = Self::allocate_node(
                    nodes,
                    Node {
                        item: items[0].clone(),
                        left_index: None,
                        right_index: None,
                    },
                );
                Some(index)
            }
            _ => {
                items.sort_unstable_by(|lhs, rhs| lhs.cmp_in_depth(rhs, depth));

                let mid = items.len() / 2;
                let (left_slice, mid_right) = items.split_at_mut(mid);
                let (mid_item, right_slice) =
                    mid_right.split_first_mut().expect("right split must exist");

                let left_index = Self::construct_part(nodes, left_slice, depth + 1);
                let right_index = Self::construct_part(nodes, right_slice, depth + 1);
                let mid_node_index = Self::allocate_node(
                    nodes,
                    Node {
                        item: mid_item.clone(),
                        left_index,
                        right_index,
                    },
                );

                Some(mid_node_index)
            }
        }
    }

    fn allocate_node(nodes: &mut Vec<Node<T>>, node: Node<T>) -> NonZeroUsize {
        nodes.push(node);
        NonZeroUsize::new(nodes.len()).expect("must not be empty")
    }
}

struct Node<T> {
    item: T,
    left_index: Option<NonZeroUsize>,
    right_index: Option<NonZeroUsize>,
}
