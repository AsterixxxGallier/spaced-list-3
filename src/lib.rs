#![feature(trait_alias)]
#![feature(int_log)]
#![feature(default_free_fn)]
#![feature(option_get_or_insert_default)]
#![allow(unused)]

use std::default::default;
use std::fmt;
use std::fmt::{Debug, Formatter, Write};
use std::iter::empty;
use std::num::NonZeroU64;
use std::ops::{Add, AddAssign, Index, IndexMut, Sub};
use std::ptr::NonNull;
use indenter::{indented, Indented};
use num_traits::{Zero, zero};

pub trait Spacing = Add<Output=Self> + AddAssign + Sub<Output=Self> + Zero + Ord + Copy;

// region helper functions
fn necessary_link_length_capacity_for_size(size: usize) -> usize {
    match size {
        // never actually happens
        0 => 0,
        1 => 0,
        2 => 1,
        _ => (1 << ((size - 1 - 1).checked_log2().expect("size was 2, can't take logarithm") + 1)) * 2 - 1
    }
}

const fn link_index(node_index: usize, degree: usize) -> usize {
    (((node_index >> degree << 1) + 1) << degree) - 1
}
// endregion

// region link indices above iterator
struct LinkIndicesAbove {
    degree: usize,
    node_index: usize,
}

impl LinkIndicesAbove {
    fn new(node_index: usize) -> Self {
        Self {
            degree: 0,
            node_index,
        }
    }
}

impl Iterator for LinkIndicesAbove {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let result = link_index(self.node_index, self.degree);
        self.node_index &= !(1 << self.degree);
        self.degree += 1;
        Some(result)
    }
}
// endregion

// region traversal result
#[derive(Eq, PartialEq)]
struct TraversalResult<'a, S: Spacing> {
    list: &'a SpacedList<S>,
    position: S,
    index: usize,
}

impl<S: Spacing> Debug for TraversalResult<'_, S>
    where S: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return f.debug_struct("TraversalResult")
            .field("position", &self.position)
            .field("index", &self.index)
            .finish();
    }
}
// endregion

// region spaced list
/// A list that stores the distance between its nodes, but does not store values
///
/// IMPORTANT: There are no empty instances of this list. Upon construction, it already contains a
/// node at position zero, meaning it has a size of 1, not 0.
#[derive(Eq, PartialEq)]
pub struct SpacedList<S: Spacing> {
    size: usize,
    length: S,
    link_lengths: Vec<S>,
    sublists: Vec<Option<SpacedList<S>>>,
}

impl<S: Spacing> Default for SpacedList<S> {
    fn default() -> Self {
        Self {
            size: 1,
            length: zero(),
            link_lengths: vec![],
            sublists: vec![],
        }
    }
}

impl<S: Spacing> SpacedList<S> {
    fn new() -> Self {
        default()
    }

    fn make_space(&mut self) {
        if self.size < 2 {
            return;
        }
        let necessary_capacity = necessary_link_length_capacity_for_size(self.size);
        if self.link_lengths.len() < necessary_capacity {
            self.link_lengths.push(self.length);
            self.link_lengths.extend(vec![zero(); self.link_lengths.len() - 1].iter());
        }
    }

    fn depth(&self) -> usize {
        (self.link_lengths.len() + 1).trailing_zeros() as usize
    }

    /// TODO this documentation is just for thoughts rn, nothing to go onto docs.rs or anything
    ///
    /// A list of size 0 is completely empty. No nodes, no length, no link lengths, no sublists.
    /// This is way too useless and similar to a list of size 1, so it doesn't exist.
    ///
    /// A list of size 1 has one node, chilling at absolute zero, no length, no link lengths.
    /// A single-node list does not have sublists either, because they would *have* to go after the
    /// last node, which does not make sense (if you want an element there, just append it).
    /// This is the kind of list you get from calling SpacedList::new() or SpacedList::default().
    ///
    /// A list of size 2 has two nodes, one at zero, one distanced from that, a length and a single
    /// link length equal to the distance between the two nodes.
    /// It may have a sublist between them, after node zero.
    ///
    /// A list of any higher size has enough nodes to link every node to the next one, and binary
    /// shortcut (higher-degree) links on several degrees up. It has a length equal to the greatest
    /// shortcut link (the one right in the middle of `link_lengths`, at least using the current
    /// system of link length storage, state 035dba816c16b4005a65c3e0132fdaa59cf6bbc7), and a number
    /// of link lengths that is enough to store the distances between nodes, allowing for a total
    /// size up to the next power of 2 + 1. There can be a sublist after every node except for the
    /// last one.
    ///
    ///
    fn append_node(&mut self, distance: S) {
        assert!(distance >= zero());
        // self.size is at least 1 (no empty lists exist), so
        self.size += 1;
        // self.size is greater than one by now
        self.make_space();
        self.length += distance;
        for link_index in LinkIndicesAbove::new(self.size - 1 - 1).take(self.depth()) {
            self.link_lengths[link_index] += distance
        }
        self.sublists.push(None)
    }

    /// Returns a mutable reference to the sublist at `index`, creating an empty one if absent
    fn get_sublist_at_index(&mut self, index: usize) -> &mut SpacedList<S> {
        self.sublists[index].get_or_insert_default()
    }

    fn insert(&mut self, position: S) {
        assert!(position >= zero());

        if position >= self.length {
            self.append_node(position - self.length)
        } else {
            // zero() <= position < self.length
            let TraversalResult { list, position: node_position, index } =
                self.node_before_or_at_shallow(position);
            let sublist = self.get_sublist_at_index(index);
            let position_in_sublist = position - node_position;
            sublist.insert(position_in_sublist)
        }
    }

    /// Returns the last node before (the greatest less than) `target_position` in this list, not in
    /// sublists
    fn node_before_shallow(&self, target_position: S) -> TraversalResult<S> {
        assert!(target_position >= zero());

        let mut position = S::zero();
        let mut index = 0usize;
        for degree in (0..self.depth()).rev() {
            let possibly_next_index = index + (1 << degree);
            if possibly_next_index < self.size {
                let possibly_next_position = position + self[(index, degree)];
                if possibly_next_position < target_position {
                    position = possibly_next_position;
                    index = possibly_next_index;
                }
            }
        }

        TraversalResult {
            list: self,
            position,
            index,
        }
    }

    /// Returns the last node before or at (the greatest less than or equal to) `target_position` in
    /// this list, not in sublists
    fn node_before_or_at_shallow(&self, target_position: S) -> TraversalResult<S> {
        assert!(target_position >= zero());

        let mut position = S::zero();
        let mut index = 0usize;
        for degree in (0..self.depth()).rev() {
            let possibly_next_index = index + (1 << degree);
            if possibly_next_index < self.size {
                let possibly_next_position = position + self[(index, degree)];
                if possibly_next_position <= target_position {
                    position = possibly_next_position;
                    index = possibly_next_index;
                }
            }
        }

        TraversalResult {
            list: self,
            position,
            index,
        }
    }

    /// Returns the node at `target_position` in this list, not in sublists, or None if this list
    /// does not contain a node at `target_position`
    fn node_at_shallow(&self, target_position: S) -> Option<TraversalResult<S>> {
        assert!(target_position >= zero());

        let mut position = S::zero();
        let mut index = 0usize;
        for degree in (0..self.depth()).rev() {
            let possibly_next_index = index + (1 << degree);
            if possibly_next_index < self.size {
                let possibly_next_position = position + self[(index, degree)];
                if possibly_next_position <= target_position {
                    position = possibly_next_position;
                    index = possibly_next_index;
                }
            }
        }

        if position == target_position {
            Some(TraversalResult {
                list: self,
                position,
                index,
            })
        } else {
            None
        }
    }
}
// endregion

// region spaced list indexing
impl<S: Spacing> Index<(usize, usize)> for SpacedList<S> {
    type Output = S;

    fn index(&self, (node_index, degree): (usize, usize)) -> &Self::Output {
        &self.link_lengths[link_index(node_index, degree)]
    }
}

impl<S: Spacing> IndexMut<(usize, usize)> for SpacedList<S> {
    fn index_mut(&mut self, (node_index, degree): (usize, usize)) -> &mut Self::Output {
        &mut self.link_lengths[link_index(node_index, degree)]
    }
}
// endregion

// region spaced list debug formatting
const ID_LETTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

fn id_letter(id: usize) -> char {
    ID_LETTERS.chars().cycle().nth(id).unwrap()
}

impl<S: Spacing> Debug for SpacedList<S>
    where S: Into<usize> {
    /// Prints this list in a human-readable format, including sublists.
    ///
    /// # Panics
    ///
    /// This function panics when a link length of zero is encountered.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let depth = self.depth();
        for degree in (0..depth).rev() {
            f.write_str(" ")?;
            for index in (0..self.size - 1).step_by(1 << degree) {
                let link_length = self[(index, degree)];
                if link_length.is_zero() {
                    panic!("Encountered link length of zero while debug formatting spaced list")
                }
                f.write_str("‾".repeat(link_length.into() * 2 - 1).as_str())?;
                f.write_str("\\")?;
            }
            f.write_char('\n')?;
        }
        f.write_char('0')?;
        let mut position: S = zero();
        for index in 1..self.size {
            let link_length = self[(index - 1, 0)];
            if link_length.is_zero() {
                panic!("Encountered link length of zero while debug formatting spaced list")
            }
            position += link_length;
            f.write_str(" ".repeat(link_length.into() - 1).as_str())?;
            write!(f, "{:2}", position.into());
        }
        f.write_char('\n')?;
        f.write_char(' ')?;
        let mut sublists = vec![];
        for index in 0..self.size - 1 {
            let link_length = self[(index, 0)];
            if link_length.is_zero() {
                panic!("Encountered link length of zero while debug formatting spaced list")
            }
            let sublist = &self.sublists[index];
            if let Some(sublist) = sublist {
                f.write_char('^')?;
                f.write_char(id_letter(sublists.len()))?;
                sublists.push(sublist);
            } else {
                f.write_str("  ")?;
            }
            f.write_str(" ".repeat(link_length.into() - 1).as_str())?;
        }
        f.write_char('\n')?;
        for (id, &sublist) in sublists.iter().enumerate() {
            write!(indented(f).with_str("| "), "sublist {}: \n{:?}", id_letter(id), sublist);
        }
        Ok(())
    }
}
// endregion

#[cfg(test)]
mod tests {
    use std::default::default;
    use crate::{link_index, LinkIndicesAbove, necessary_link_length_capacity_for_size, SpacedList, TraversalResult};

    #[test]
    fn test_spaced_list_necessary_capacity() {
        assert_eq!(necessary_link_length_capacity_for_size(0), 0);
        assert_eq!(necessary_link_length_capacity_for_size(1), 0);
        assert_eq!(necessary_link_length_capacity_for_size(2), 1);
        assert_eq!(necessary_link_length_capacity_for_size(3), 3);
        assert_eq!(necessary_link_length_capacity_for_size(4), 7);
        assert_eq!(necessary_link_length_capacity_for_size(5), 7);
        assert_eq!(necessary_link_length_capacity_for_size(6), 15);
        assert_eq!(necessary_link_length_capacity_for_size(7), 15);
        assert_eq!(necessary_link_length_capacity_for_size(8), 15);
        assert_eq!(necessary_link_length_capacity_for_size(9), 15);
        assert_eq!(necessary_link_length_capacity_for_size(10), 31);
        assert_eq!(necessary_link_length_capacity_for_size(11), 31);
        assert_eq!(necessary_link_length_capacity_for_size(12), 31);
        assert_eq!(necessary_link_length_capacity_for_size(13), 31);
        assert_eq!(necessary_link_length_capacity_for_size(14), 31);
        assert_eq!(necessary_link_length_capacity_for_size(15), 31);
        assert_eq!(necessary_link_length_capacity_for_size(16), 31);
        assert_eq!(necessary_link_length_capacity_for_size(17), 31);
    }

    #[test]
    fn test_link_index() {
        assert_eq!(link_index(0, 0), 0b0000);
        assert_eq!(link_index(1, 0), 0b0010);
        assert_eq!(link_index(2, 0), 0b0100);
        assert_eq!(link_index(3, 0), 0b0110);
        assert_eq!(link_index(4, 0), 0b1000);
        assert_eq!(link_index(5, 0), 0b1010);
        assert_eq!(link_index(6, 0), 0b1100);
        assert_eq!(link_index(7, 0), 0b1110);
        assert_eq!(link_index(0, 1), 0b0001);
        assert_eq!(link_index(1, 1), 0b0001);
        assert_eq!(link_index(2, 1), 0b0101);
        assert_eq!(link_index(3, 1), 0b0101);
        assert_eq!(link_index(4, 1), 0b1001);
        assert_eq!(link_index(5, 1), 0b1001);
        assert_eq!(link_index(6, 1), 0b1101);
        assert_eq!(link_index(7, 1), 0b1101);
        assert_eq!(link_index(0, 2), 0b0011);
        assert_eq!(link_index(1, 2), 0b0011);
        assert_eq!(link_index(2, 2), 0b0011);
        assert_eq!(link_index(3, 2), 0b0011);
        assert_eq!(link_index(4, 2), 0b1011);
        assert_eq!(link_index(5, 2), 0b1011);
        assert_eq!(link_index(6, 2), 0b1011);
        assert_eq!(link_index(7, 2), 0b1011);
    }

    #[test]
    fn test_link_indices_above() {
        let mut iterator = LinkIndicesAbove::new(0);
        assert_eq!(iterator.next(), Some(link_index(0, 0)));
        assert_eq!(iterator.next(), Some(link_index(0, 1)));
        assert_eq!(iterator.next(), Some(link_index(0, 2)));
        assert_eq!(iterator.next(), Some(link_index(0, 3)));

        let mut iterator = LinkIndicesAbove::new(1);
        assert_eq!(iterator.next(), Some(link_index(1, 0)));
        assert_eq!(iterator.next(), Some(link_index(0, 1)));
        assert_eq!(iterator.next(), Some(link_index(0, 2)));
        assert_eq!(iterator.next(), Some(link_index(0, 3)));

        let mut iterator = LinkIndicesAbove::new(2);
        assert_eq!(iterator.next(), Some(link_index(2, 0)));
        assert_eq!(iterator.next(), Some(link_index(2, 1)));
        assert_eq!(iterator.next(), Some(link_index(0, 2)));
        assert_eq!(iterator.next(), Some(link_index(0, 3)));

        let mut iterator = LinkIndicesAbove::new(3);
        assert_eq!(iterator.next(), Some(link_index(3, 0)));
        assert_eq!(iterator.next(), Some(link_index(2, 1)));
        assert_eq!(iterator.next(), Some(link_index(0, 2)));
        assert_eq!(iterator.next(), Some(link_index(0, 3)));

        let mut iterator = LinkIndicesAbove::new(4);
        assert_eq!(iterator.next(), Some(link_index(4, 0)));
        assert_eq!(iterator.next(), Some(link_index(4, 1)));
        assert_eq!(iterator.next(), Some(link_index(4, 2)));
        assert_eq!(iterator.next(), Some(link_index(0, 3)));

        let mut iterator = LinkIndicesAbove::new(5);
        assert_eq!(iterator.next(), Some(link_index(5, 0)));
        assert_eq!(iterator.next(), Some(link_index(4, 1)));
        assert_eq!(iterator.next(), Some(link_index(4, 2)));
        assert_eq!(iterator.next(), Some(link_index(0, 3)));
    }

    #[test]
    fn test_make_space() {
        let mut list = SpacedList::<usize>::new();
        assert_eq!(list.link_lengths.len(), 0);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 1);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 3);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 7);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 7);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 15);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 15);

        list.size += 1;
        list.make_space();
        assert_eq!(list.link_lengths.len(), 15);
    }

    #[test]
    fn test_append_node() {
        let mut list = SpacedList::<usize>::new();
        assert_eq!(list.size, 1);
        assert_eq!(list.length, 0);
        assert_eq!(list.link_lengths, vec![]);

        list.append_node(1);
        assert_eq!(list.size, 2);
        assert_eq!(list.length, 1);
        assert_eq!(list.link_lengths, vec![1]);

        list.append_node(2);
        assert_eq!(list.size, 3);
        assert_eq!(list.length, 3);
        assert_eq!(list.link_lengths, vec![1, 3, 2]);

        list.append_node(3);
        assert_eq!(list.size, 4);
        assert_eq!(list.length, 6);
        assert_eq!(list.link_lengths, vec![1, 3, 2, 6, 3, 3, 0]);

        list.append_node(2);
        assert_eq!(list.size, 5);
        assert_eq!(list.length, 8);
        assert_eq!(list.link_lengths, vec![1, 3, 2, 8, 3, 5, 2]);

        list.append_node(2);
        assert_eq!(list.size, 6);
        assert_eq!(list.length, 10);
        assert_eq!(list.link_lengths, vec![1, 3, 2, 8, 3, 5, 2, 10, 2, 2, 0, 2, 0, 0, 0]);

        list.append_node(3);
        assert_eq!(list.size, 7);
        assert_eq!(list.length, 13);
        assert_eq!(list.link_lengths, vec![1, 3, 2, 8, 3, 5, 2, 13, 2, 5, 3, 5, 0, 0, 0]);

        list.append_node(1);
        assert_eq!(list.size, 8);
        assert_eq!(list.length, 14);
        assert_eq!(list.link_lengths, vec![1, 3, 2, 8, 3, 5, 2, 14, 2, 5, 3, 6, 1, 1, 0]);
    }

    #[test]
    fn test_shallow_traversal() {
        let mut list = SpacedList::<isize>::new();
        list.append_node(2);
        list.append_node(3);
        list.append_node(4);
        list.append_node(1);

        assert_eq!(list.node_before_shallow(0),
                   TraversalResult { list: &list, position: 0, index: 0 });
        assert_eq!(list.node_before_or_at_shallow(0),
                   TraversalResult { list: &list, position: 0, index: 0 });
        assert_eq!(list.node_at_shallow(0),
                   Some(TraversalResult { list: &list, position: 0, index: 0 }));

        assert_eq!(list.node_before_shallow(1),
                   TraversalResult { list: &list, position: 0, index: 0 });
        assert_eq!(list.node_before_or_at_shallow(1),
                   TraversalResult { list: &list, position: 0, index: 0 });
        assert_eq!(list.node_at_shallow(1), None);

        assert_eq!(list.node_before_shallow(2),
                   TraversalResult { list: &list, position: 0, index: 0 });
        assert_eq!(list.node_before_or_at_shallow(2),
                   TraversalResult { list: &list, position: 2, index: 1 });
        assert_eq!(list.node_at_shallow(2),
                   Some(TraversalResult { list: &list, position: 2, index: 1 }));

        assert_eq!(list.node_before_shallow(3),
                   TraversalResult { list: &list, position: 2, index: 1 });
        assert_eq!(list.node_before_or_at_shallow(3),
                   TraversalResult { list: &list, position: 2, index: 1 });
        assert_eq!(list.node_at_shallow(3), None);

        assert_eq!(list.node_before_shallow(4),
                   TraversalResult { list: &list, position: 2, index: 1 });
        assert_eq!(list.node_before_or_at_shallow(4),
                   TraversalResult { list: &list, position: 2, index: 1 });
        assert_eq!(list.node_at_shallow(4), None);

        assert_eq!(list.node_before_shallow(5),
                   TraversalResult { list: &list, position: 2, index: 1 });
        assert_eq!(list.node_before_or_at_shallow(5),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_at_shallow(5),
                   Some(TraversalResult { list: &list, position: 5, index: 2 }));

        assert_eq!(list.node_before_shallow(6),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_before_or_at_shallow(6),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_at_shallow(6), None);

        assert_eq!(list.node_before_shallow(7),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_before_or_at_shallow(7),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_at_shallow(7), None);

        assert_eq!(list.node_before_shallow(8),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_before_or_at_shallow(8),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_at_shallow(8), None);

        assert_eq!(list.node_before_shallow(9),
                   TraversalResult { list: &list, position: 5, index: 2 });
        assert_eq!(list.node_before_or_at_shallow(9),
                   TraversalResult { list: &list, position: 9, index: 3 });
        assert_eq!(list.node_at_shallow(9),
                   Some(TraversalResult { list: &list, position: 9, index: 3 }));

        assert_eq!(list.node_before_shallow(10),
                   TraversalResult { list: &list, position: 9, index: 3 });
        assert_eq!(list.node_before_or_at_shallow(10),
                   TraversalResult { list: &list, position: 10, index: 4 });
        assert_eq!(list.node_at_shallow(10),
                   Some(TraversalResult { list: &list, position: 10, index: 4 }));

        assert_eq!(list.node_before_shallow(11),
                   TraversalResult { list: &list, position: 10, index: 4 });
        assert_eq!(list.node_before_or_at_shallow(11),
                   TraversalResult { list: &list, position: 10, index: 4 });
        assert_eq!(list.node_at_shallow(11), None);
    }

    #[test]
    fn test_insert() {
        let mut list = SpacedList::<usize>::new();
        list.insert(2);
        list.insert(6);
        list.insert(3);
        list.insert(5);
        list.insert(4);
        list.insert(7);
        list.insert(9);
        list.insert(8);

        println!("{:?}", list);
    }
}
