use num_traits::Float;
use std::{cmp::Ordering, collections::BinaryHeap, fmt::Debug, num::NonZeroUsize};

/// KdTree に格納する要素が実装しなければいけないトレイト。
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

/// k-d tree を表す。
pub struct KdTree<T> {
    nodes: Vec<Node<T>>,
    root_index: Option<NonZeroUsize>,
}

#[derive(Debug)]
struct Node<T> {
    item: T,
    left_index: Option<NonZeroUsize>,
    right_index: Option<NonZeroUsize>,
}

#[derive(Debug)]
struct NeighborCandidate<'a, T: KdTreeItem>(&'a T, T::Measurement);

impl<T: KdTreeItem> PartialEq for NeighborCandidate<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<T: KdTreeItem> Eq for NeighborCandidate<'_, T> {}

impl<T: KdTreeItem> PartialOrd for NeighborCandidate<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: KdTreeItem> Ord for NeighborCandidate<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.partial_cmp(&other.1).expect("not total order")
    }
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
        self.find_nearest_n(query, 1).into_iter().next()
    }

    pub fn find_nearest_n<'a>(&'a self, query: &'a T, max_count: usize) -> Vec<&'a T> {
        let mut candidates = BinaryHeap::with_capacity(max_count);
        self.find_nearest_n_depth(&mut candidates, max_count, self.get_node(self.root_index), query, 0);
        candidates.iter().rev().map(|c| c.0).collect()
    }

    fn find_nearest_n_depth<'a>(
        &'a self,
        candidates: &mut BinaryHeap<NeighborCandidate<'a, T>>,
        max_candidates: usize,
        root: Option<&'a Node<T>>,
        query: &'a T,
        depth: usize,
    ) {
        let Some(root) = root else {
            return;
        };

        // root が candidates に入るなら入れる
        let root_distance = query.distance(&root.item);
        if candidates.len() < max_candidates {
            candidates.push(NeighborCandidate(&root.item, root_distance));
        } else if root_distance < candidates.peek().expect("must exist").1 {
            candidates.pop();
            candidates.push(NeighborCandidate(&root.item, root_distance));
        }

        let (left_subtree, right_subtree) = (self.get_node(root.left_index), self.get_node(root.right_index));
        let (first_subtree, second_subtree) = match query.cmp_in_depth(&root.item, depth) {
            Ordering::Less => (left_subtree, right_subtree),
            Ordering::Equal | Ordering::Greater => (right_subtree, left_subtree),
        };

        // query が属する sub-tree の探索
        self.find_nearest_n_depth(candidates, max_candidates, first_subtree, query, depth + 1);

        if candidates.len() < max_candidates {
            // max_candidate に達してない場合は無条件で逆側も探索
            self.find_nearest_n_depth(candidates, max_candidates, second_subtree, query, depth + 1);
        } else {
            let axis_distance = query.distance_to_axis(&root.item, depth);
            let max_candidate_distance = &candidates.peek().expect("must exist").1;
            // candidate の最遠半径が現在の分割面を跨いでいれば逆側も探索
            if axis_distance < *max_candidate_distance {
                self.find_nearest_n_depth(candidates, max_candidates, second_subtree, query, depth + 1);
            }
        }
    }

    #[inline]
    fn get_node(&self, index: Option<NonZeroUsize>) -> Option<&Node<T>> {
        index.map(|ip1| &self.nodes[ip1.get() - 1])
    }
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
