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
/// A list that stores non-zero distance between its nodes, but does not store values.
///
/// IMPORTANT: New and empty instances of this list contain one node, fixed at position zero, and
/// thereby have a size of 1, even though they are empty.
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
        assert!(distance > zero());
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

    /// Returns a reference to the sublist at `index`, or None if there is no sublist at
    /// `index` or that sublist is empty
    fn get_not_empty_sublist_at_index(&self, index: usize) -> Option<&SpacedList<S>> {
        let sublist = (&self.sublists[index]).as_ref()?;
        if sublist.is_empty() {
            None
        } else {
            Some(sublist)
        }
    }

    /// Returns a mutable reference to the sublist at `index`, or None if there is no sublist at
    /// `index` or that sublist is empty
    fn get_not_empty_sublist_at_index_mut(&mut self, index: usize) -> Option<&mut SpacedList<S>> {
        let sublist = (&mut self.sublists[index]).as_mut()?;
        if sublist.is_empty() {
            None
        } else {
            Some(sublist)
        }
    }

    fn insert(&mut self, position: S) {
        assert!(position > zero());

        if position >= self.length {
            self.append_node(position - self.length)
        } else {
            // zero() < position < self.length
            let TraversalResult { list, position: node_position, index } =
                self.node_before_or_at_shallow(position).unwrap();
            let sublist = self.get_sublist_at_index(index);
            let position_in_sublist = position - node_position;
            assert!(position_in_sublist > zero());
            sublist.insert(position_in_sublist)
        }
    }

    fn is_empty(&self) -> bool {
        self.size == 1
    }
}

impl<S: Spacing> SpacedList<S> {
    /// Returns the last node before (the greatest less than) `target_position` in this list, not in
    /// sublists, or None if `target_position` is negative.
    fn node_before_shallow(&self, target_position: S) -> Option<TraversalResult<S>> {
        if target_position < zero() {
            return None;
        }

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

        Some(TraversalResult {
            list: self,
            position,
            index,
        })
    }

    /// Returns the last node before or at (the greatest less than or equal to) `target_position` in
    /// this list, not in sublists, or None if `target_position` is negative.
    fn node_before_or_at_shallow(&self, target_position: S) -> Option<TraversalResult<S>> {
        if target_position < zero() {
            return None;
        }

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

        Some(TraversalResult {
            list: self,
            position,
            index,
        })
    }

    /// Returns the node at `target_position` in this list, not in sublists, or None if this list
    /// does not contain a node at `target_position`.
    fn node_at_shallow(&self, target_position: S) -> Option<TraversalResult<S>> {
        if target_position < zero() {
            return None;
        }

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

    /// Returns the first node after or at (the least greater than or equal to) `target_position` in
    /// this list, not in sublists, or None if `target_position > self.length`.
    fn node_after_or_at_shallow(&self, target_position: S) -> Option<TraversalResult<S>> {
        if target_position < zero() {
            return Some(TraversalResult {
                list: self,
                position: zero(),
                index: 0,
            });
        }

        if target_position > self.length {
            return None;
        }

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
            // target_position < self.length
            // therefore, we can safely assume there is a node after position and index
            Some(TraversalResult {
                list: self,
                position: position + self[(index, 0)],
                index: index + 1,
            })
        }
    }

    /// Returns the first node after (the least greater than) `target_position` in this list, not in
    /// sublists, or None if `target_position > self.length`.
    fn node_after_shallow(&self, target_position: S) -> Option<TraversalResult<S>> {
        if target_position < zero() {
            return Some(TraversalResult {
                list: self,
                position: zero(),
                index: 0,
            });
        }

        if target_position >= self.length {
            return None;
        }

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

        // target_position < self.length
        // therefore, we can safely assume there is a node after position and index
        Some(TraversalResult {
            list: self,
            position: position + self[(index, 0)],
            index: index + 1,
        })
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let depth = self.depth();
        for degree in (0..depth).rev() {
            f.write_str(" ")?;
            for index in (0..self.size - 1).step_by(1 << degree) {
                let link_length = self[(index, degree)];
                f.write_str("‾".repeat(link_length.into() * 2 - 1).as_str())?;
                f.write_str("\\")?;
            }
            f.write_char('\n')?;
        }
        f.write_char('0')?;
        let mut position: S = zero();
        for index in 1..self.size {
            let link_length = self[(index - 1, 0)];
            position += link_length;
            f.write_str(" ".repeat(link_length.into() - 1).as_str())?;
            write!(f, "{:2}", position.into());
        }
        f.write_char('\n')?;
        f.write_char(' ')?;
        let mut sublists = vec![];
        for index in 0..self.size - 1 {
            let link_length = self[(index, 0)];
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
mod tests;
