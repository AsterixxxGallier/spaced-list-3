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
