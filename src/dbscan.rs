use std::{cmp::Ordering, collections::VecDeque, num::NonZeroUsize};

use crate::kdtree::{KdTree, KdTreeItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DbscanLabel {
    Cluster(NonZeroUsize),
    Noize,
}

#[derive(Debug, Clone)]
struct Indexed<'a, T>(usize, &'a T);

impl<T: KdTreeItem> KdTreeItem for Indexed<'_, T> {
    type Measurement = T::Measurement;

    fn cmp_in_depth(&self, rhs: &Self, depth: usize) -> Ordering {
        self.1.cmp_in_depth(rhs.1, depth)
    }

    fn distance(&self, other: &Self) -> Self::Measurement {
        self.1.distance(other.1)
    }

    fn distance_to_axis(&self, other: &Self, depth: usize) -> Self::Measurement {
        self.1.distance_to_axis(other.1, depth)
    }
}

pub fn dbscan<T: KdTreeItem>(items: impl Into<Vec<T>>, epsilon: T::Measurement, min_items: usize) -> Vec<DbscanLabel> {
    let items = items.into();
    let indexed_items: Vec<_> = items.iter().enumerate().map(|(i, item)| Indexed(i, item)).collect();

    let kdtree = KdTree::construct(indexed_items.clone());
    let mut core_neighbor_groups = VecDeque::with_capacity(indexed_items.len() / min_items);

    let mut cluster_id = NonZeroUsize::new(1).expect("must be 1");
    let mut labels = Vec::with_capacity(indexed_items.len());
    let mut visited = Vec::with_capacity(indexed_items.len());
    labels.resize(indexed_items.len(), DbscanLabel::Noize);
    visited.resize(indexed_items.len(), false);

    for item in &indexed_items {
        if visited[item.0] {
            continue;
        }

        visited[item.0] = true;
        let neighbors = kdtree.find_range_n(item, &epsilon);

        // コア点であればクラスターを生成
        if neighbors.len() >= min_items {
            let cluster_label = DbscanLabel::Cluster(cluster_id);
            labels[item.0] = cluster_label;

            // コア点候補は VecDeque で先頭から探索する
            core_neighbor_groups.push_back(neighbors);
            while let Some(neighbors) = core_neighbor_groups.pop_front() {
                for neighbor in neighbors {
                    if !visited[neighbor.0] {
                        visited[neighbor.0] = true;
                        labels[neighbor.0] = cluster_label;

                        let sub_neighbors = kdtree.find_range_n(neighbor, &epsilon);
                        if sub_neighbors.len() >= min_items {
                            core_neighbor_groups.push_back(sub_neighbors);
                        }
                    }

                    if labels[neighbor.0] == DbscanLabel::Noize {
                        labels[neighbor.0] = cluster_label;
                    }
                }
            }

            cluster_id = cluster_id.saturating_add(1);
        }
    }

    labels
}
