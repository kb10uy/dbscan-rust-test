use num_traits::Float;
use std::{borrow::Borrow, cmp::Ordering, fmt::Debug, num::NonZeroUsize};

pub trait KdTreeItem: Debug + Clone {
    type Measurement: Debug + PartialOrd;

    /// 指定されたツリー深度で要素同士を比較する。
    /// 一般的に ```components[depth % N]``` が比較されるように実装される。
    fn cmp_in_depth(&self, rhs: &Self, depth: usize) -> Ordering;

    /// 2 要素間の距離を計算する。距離不等式を満たしていればよい。
    fn distance(&self, other: &Self) -> Self::Measurement;

    /// もう一方の要素の軸との距離を計算する。 distance() と一貫性があればよい。
    fn distance_to_axis(&self, other: &Self, depth: usize) -> Self::Measurement;
}

impl<T: Debug + Float, const N: usize> KdTreeItem for [T; N] {
    type Measurement = T;

    fn cmp_in_depth(&self, rhs: &Self, depth: usize) -> Ordering {
        self[depth % N].partial_cmp(&rhs[depth % N]).expect("not total order")
    }

    fn distance(&self, other: &Self) -> Self::Measurement {
        (0..N)
            .map(|i| (self[i] - other[i]).powi(2))
            .fold(T::zero(), |a, x| a + x)
            .sqrt()
    }

    fn distance_to_axis(&self, other: &Self, depth: usize) -> Self::Measurement {
        let i = depth % N;
        (self[i] - other[i]).abs()
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

        let root_index = construct_part(&mut nodes, &mut items, 0);

        KdTree { nodes, root_index }
    }

    pub fn root(&self) -> Option<&T> {
        self.get_node(self.root_index).map(|n| &n.item)
    }

    pub fn find_nearest<'a>(&'a self, query: &'a T) -> Option<&'a T> {
        let query = query.borrow();
        self.find_nearest_depth(self.get_node(self.root_index), query, 0)
            .map(|n| &n.item)
    }

    fn find_nearest_depth<'a>(&'a self, root: Option<&'a Node<T>>, query: &'a T, depth: usize) -> Option<&'a Node<T>> {
        let root = root?;
        let (left_subtree, right_subtree) = (self.get_node(root.left_index), self.get_node(root.right_index));
        let (first_subtree, second_subtree) = match query.cmp_in_depth(&root.item, depth) {
            Ordering::Less => (left_subtree, right_subtree),
            Ordering::Equal | Ordering::Greater => (right_subtree, left_subtree),
        };

        // query が属する sub-tree の探索
        let first_subtree_nearest = self.find_nearest_depth(first_subtree, query, depth + 1);
        let (first_best, first_best_distance) = select_nearest(query, root, first_subtree_nearest);

        // first_best_distance が現在の分割面を跨いでいなければ打ち切り
        let axis_distance = query.distance_to_axis(&root.item, depth);
        if axis_distance >= first_best_distance {
            return Some(first_best);
        }

        // 逆側の sub-tree の探索
        let second_subtree_nearest = self.find_nearest_depth(second_subtree, query, depth + 1);
        let (second_best, _) = select_nearest(query, first_best, second_subtree_nearest);
        Some(second_best)
    }

    #[inline]
    fn get_node(&self, index: Option<NonZeroUsize>) -> Option<&Node<T>> {
        index.map(|ip1| &self.nodes[ip1.get() - 1])
    }
}

#[derive(Debug)]
struct Node<T> {
    item: T,
    left_index: Option<NonZeroUsize>,
    right_index: Option<NonZeroUsize>,
}

fn construct_part<T: KdTreeItem>(nodes: &mut Vec<Node<T>>, items: &mut [T], depth: usize) -> Option<NonZeroUsize> {
    match items.len() {
        0 => None,
        1 => {
            let index = allocate_node(
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
            let (mid_item, right_slice) = mid_right.split_first_mut().expect("right split must exist");

            let left_index = construct_part(nodes, left_slice, depth + 1);
            let right_index = construct_part(nodes, right_slice, depth + 1);
            let mid_node_index = allocate_node(
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

fn allocate_node<T: KdTreeItem>(nodes: &mut Vec<Node<T>>, node: Node<T>) -> NonZeroUsize {
    nodes.push(node);
    NonZeroUsize::new(nodes.len()).expect("must not be empty")
}

fn select_nearest<'a, T: KdTreeItem>(
    query: &'a T,
    node1: &'a Node<T>,
    node2: Option<&'a Node<T>>,
) -> (&'a Node<T>, T::Measurement) {
    let node1_distance = node1.item.distance(query);

    let Some(node2) = node2 else {
        return (node1, node1_distance);
    };
    let node2_distance = node2.item.distance(query);

    if node1_distance < node2_distance {
        (node1, node1_distance)
    } else {
        (node2, node2_distance)
    }
}
