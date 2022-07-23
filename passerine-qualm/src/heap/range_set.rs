use std::collections::{BTreeMap, BTreeSet};

use crate::heap::pointer::{Pointer, PointerIdx};

// Needs to do a few simple things:
// Returns all ranges greater than or equal to a given size
// when a range is added, merges neighboring ranges together
// when a range is removed, splits neighboring ranges

/// Keeps track of unallocated ranges of slots.
/// When a pointer is freed, it's range is merged with other ranges.
/// We use a pair of BTreeMaps to keep this snappy under the hood.
#[derive(Debug)]
pub(super) struct RangeSet {
    pub(super) capacity: usize,
    // slots before, length of range
    pub(super) ranges: BTreeMap<PointerIdx, usize>,
    // length -> start of range
    // if an entry size is present in the map, the pointer set must be non-empty.
    pub(super) free: BTreeMap<usize, BTreeSet<PointerIdx>>,
}

impl RangeSet {
    /// Create a new RangeSet with no capacity
    pub fn new() -> RangeSet {
        RangeSet {
            capacity: 0,
            ranges:   BTreeMap::new(),
            free:     BTreeMap::new(),
        }
    }

    /// Adds some capacity to the heap.
    pub fn add_free_capacity(&mut self, slots: usize) {
        self.free(Pointer::new(PointerIdx::new(self.capacity as u64)), slots);
        self.capacity += slots;
    }

    /// Create a new rangeset with the capacity of a pre-allocated heap.
    pub fn new_with_free_capacity(slots: usize) -> RangeSet {
        let mut empty = RangeSet::new();
        empty.add_free_capacity(slots);
        empty
    }

    /// Returns a pointer and the size to increase the allocation by.
    /// The backing allocation size must be increased according to the returned size.
    /// Do not call `add_free_capacity` with the returned size of this method,
    /// because the allocation is used, not free.
    pub fn mark_first(&mut self, slots: usize) -> (Pointer, usize) {
        // try filling the smallest earliest gap possible.
        if let Some((_size, potential)) = self.free.range(slots..).next() {
            let pointer = *potential.iter().next().unwrap();
            self.mark_smaller(pointer, slots);
            return (Pointer::new(pointer), 0);
        }

        // if the last range is a tail range, try extending it
        if let Some((tail, size)) = self.ranges.iter().rev().next() {
            // copy to please the borrow checker gods
            let (tail, size) = (*tail, *size);
            // this free range goes right up to the end
            if tail.to_usize() + size == self.capacity {
                self.mark(tail);
                let remaining = slots - size;
                self.capacity += remaining;
                return (Pointer::new(tail), remaining)
            }
        }

        let pointer = Pointer::new(PointerIdx::new(self.capacity as u64));
        self.capacity += slots;
        return (pointer, slots);
    }

    /// Mark a pointer for use reserving a certain number of slots,
    /// returns the extra free space to the heap.
    pub fn mark_smaller(&mut self, pointer: PointerIdx, slots: usize) {
        // grab the full allocation
        let size = self.mark(pointer);
        if size == slots { return; }

        // let go of the end; may cause minor fragmentation
        assert!(slots < size);
        self.free(Pointer::new(pointer.into()).add(slots as u64), size - slots);
    }

    /// Mark a pointer for use, returns the size of the full allocation
    fn mark(&mut self, pointer: PointerIdx) -> usize {
        // remove it from the ranges set, getting the size of the pointer
        let size = self.ranges.remove(&pointer).unwrap();
        // remove it from the free set, by inverse looking up by size
        assert!(self.free.get_mut(&size).unwrap().remove(&pointer));
        // if the pointer was the last of a given size, remove the entry from the map
        if self.free.get(&size).unwrap().is_empty() {
            self.free.remove_entry(&size);
        }
        // return the size of the marked pointer
        return size;
    }

    // TODO: tail allocations.
    /// Returns whether a pointer of a given size is free at a given point.
    /// Used to determine whether reallocation in place is possible.
    pub fn is_free(&self, pointer: Pointer, slots: usize) -> bool {
        let pointer: PointerIdx = pointer.to_idx();

        // get the first pointer before or at the one specified.
        if let Some((p, free_range)) = self.ranges.range(..=pointer).rev().next() {
            // check that the free range covers the range of the pointer in question
            let p_end = p.to_usize() + free_range;
            let pointer_end = pointer.to_usize() + slots;

            // for a pointer to be free it must be in the range!
            if p_end >= pointer_end {
                return true;
            }

            // TODO: tail allocations; need to increase the allocation size.
            // || self.free.capacity == p_end
            //     && self.free.capacity < pointer_end
        }

        false
    }

    /// Returns capacity that the heap can be shrunk by if freeing a tail allocation
    pub fn free(&mut self, pointer: Pointer, mut slots: usize) -> usize {
        let mut pointer: PointerIdx = pointer.into();

        // merge it with any other nearby ranges
        // start with the range before
        if let Some((pointer_before, size)) = self.ranges.range(..pointer).rev().next() {
            let (pointer_before, size) = (*pointer_before, *size);

            // if the free ranges are back-to-back, we merge them by extending the old range
            if pointer_before + size as u64 == pointer {
                // use the new combined pointer
                self.mark(pointer_before);
                pointer = pointer_before;
                slots = size + slots;
            }
        }

        // then do the range after
        // pointer has not yet beed added to the ranges map,
        // so `pointer..` is technically exclusive
        if let Some((pointer_after, size)) = self.ranges.range(pointer..).next() {
            let (pointer_after, size) = (*pointer_after, *size);
            if pointer + size as u64 == pointer_after {
                // extend the pointer to be longer
                self.mark(pointer_after);
                slots += size;
            }
        }

        // if this is a tail free, reduce the size of the heap
        if pointer.to_usize() + slots== self.capacity {
            self.capacity -= slots;
            return slots;
        }

        // add the pointer with its new size in the free map
        // add the pointer with its new size to the ranges map
        if let Some(s) = self.free.get_mut(&slots) {
            s.insert(pointer);
        } else {
            let mut pointers = BTreeSet::new();
            pointers.insert(pointer);
            self.free.insert(slots, pointers);
        }

        // not a tail free, there still may be an allocation after this one
        // return 0 to keep slots
        assert!(self.ranges.insert(pointer, slots).is_none());
        return 0;
    }
}
