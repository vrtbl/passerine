use crate::Slot;

pub mod pointer;
pub mod range_set;

pub use pointer::Pointer;
use range_set::RangeSet;

#[derive(Debug)]
pub struct Heap {
    data: Vec<Slot>,
    free: RangeSet,
}

impl Heap {
    /// Constructs new empty heap.
    pub fn new() -> Heap {
        Heap {
            data: vec![],
            free: RangeSet::new(),
        }
    }

    /// Dumps a representation of the heap to stdout.
    /// Useful for general debugging.
    pub fn draw_free(&self) {
        print!("|");
        let mut old = 0;
        let mut unused = 0;
        for (key, value) in self.free.ranges.iter() {
            print!("{}", "_".repeat(key.to_usize() - old));
            print!("{}", "X".repeat(*value));
            unused += value;
            old = key.to_usize();
        }
        print!("{}", "_".repeat(self.free.capacity - old));
        println!("|");

        println!("==== INFO ====");
        println!("heap size:       {} bytes", self.data.len() * 8);
        println!("total slots:     {} slots", self.data.len());
        println!("disjoint ranges: {} slots", self.free.ranges.len());
        let pct = (unused as f64 / self.free.capacity as f64) * 100.0;
        println!(
            "fragmentation:   {} / {} = {:.2}%",
            unused, self.free.capacity, pct
        );
    }

    /// Allocate a pointer of a given size.
    /// Returns the smallest first allocation that will fit the pointer.
    ///
    /// # Safety
    /// The allocated pointer will point to garbage data.
    /// This call must immediately be followed by a call to [`Heap::write`].
    pub unsafe fn alloc(&mut self, slots: usize) -> Pointer {
        let (pointer, extra_capacity) = self.free.mark_first(slots);

        // increase the size of the allocation if needed.
        self.data
            .extend((0..extra_capacity).map(|_| unsafe { Slot::zero() }));
        return pointer;
    }

    /// Reallocates an allocation to a larger size
    /// Tries to reallocate in place, but moves the allocation if needed.
    ///
    /// # Safety
    /// All previously allocated data will be present at the start of the new
    /// allocation. For example, if you have an allocation `*x = ABC` of len
    /// 3, and reallocate to len 5, the new allocation will be `*x = ABC__`.
    ///
    /// Likewise, if you decrease the size of the allocation, data will be
    /// truncated. So `*x = ABC` of len 3 reallocated to len 1 would be `*x
    /// = A`.
    ///
    /// Like when using `Heap::alloc`, this call must be immediately be followed
    /// by a call to [`Heap::write`] to fill the uninitialized portion of
    /// the new array.
    pub unsafe fn realloc(
        &mut self,
        pointer: Pointer,
        old: usize,
        new: usize,
    ) -> Pointer {
        assert!(pointer.is_owned());

        if new > old {
            // try allocation continiously
            let tail = pointer.add(old as u64);
            if self.free.is_free(tail, new - old) {
                // increase the size of the current allocation
                self.free.mark_smaller(tail.to_idx(), new - old);
                return pointer;
            }

            // TODO: free before reallocation might open up space before?
            // reallocate new larger allocation, copy over data.
            let new_pointer = self.alloc(new);
            for slot in 0..old {
                self.data.swap(
                    new_pointer.to_idx().to_usize() + slot,
                    pointer.to_idx().to_usize() + slot,
                );
            }
            // and free old small allocation
            self.free(pointer, old);
            return new_pointer;
        } else if old > new {
            // free back half of allocation
            self.free(pointer.add(new as u64), old - new);
        }

        // they're equal, so do nothing
        return pointer;
    }

    // Reads a single slot relative to a pointer.
    pub fn read_slot(&self, pointer: Pointer, slot: usize) -> &Slot {
        &self.data[pointer.to_idx().to_usize() + slot]
    }

    // Reads a range of data.
    pub fn read(&self, pointer: Pointer, slots: usize) -> &[Slot] {
        let start = pointer.to_idx().to_usize() as usize;
        &self.data[start..(start + slots)]
    }

    // TODO: figure out apis for reading and writing

    pub fn write_slot(&self, pointer: Pointer, slots: usize) -> () { todo!() }

    pub fn write(&mut self, pointer: Pointer, item: &mut [Slot]) -> Option<()> {
        // can't write to a pointer we don't own! make a copy first.
        if !pointer.is_owned() {
            return None;
        }
    }

    pub fn free(&mut self, pointer: Pointer, slots: usize) {
        assert!(pointer.is_owned());
        let unneeded_capacity = self.free.free(pointer, slots);
        self.data.truncate(self.data.len() - unneeded_capacity);
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn random_alloc_size(rng: &mut attorand::Rng) -> usize {
        rng.next_byte() as usize + 1
    }

    const STRESS_ITER: usize = 1000;

    #[test]
    pub fn stress_test_heap() {
        let mut heap = Heap::new();
        let mut pointers = BTreeMap::new();
        let mut rng = attorand::Rng::new_default();

        for i in 0..STRESS_ITER {
            if i % 100 == 0 {
                println!("{}", i);
            }
            let size = random_alloc_size(&mut rng);
            // SAFETY: data is never read
            let pointer = unsafe { heap.alloc(size) };
            pointers.insert(i, (pointer, size));

            let index = rng.next_u64_max((pointers.len() - 1) as u64) as usize;
            if rng.next_bool() {
                let (index, (to_modify, old_size)) =
                    pointers.iter().nth(index).unwrap();
                let index = *index;

                if rng.next_bool() {
                    let new_size = random_alloc_size(&mut rng);
                    // SAFETY: data is never read
                    let pointer = unsafe {
                        heap.realloc(*to_modify, *old_size, new_size)
                    };
                    pointers.insert(index, (pointer, new_size));
                } else {
                    heap.free(*to_modify, *old_size);
                    pointers.remove(&index);
                }
            }
        }

        heap.draw_free();
    }

    #[test]
    pub fn stress_test_native() {
        let mut rng = attorand::Rng::new_default();
        let mut pointers = BTreeMap::new();

        for i in 0..STRESS_ITER {
            let size = random_alloc_size(&mut rng);
            let pointer = vec![0; size];
            pointers.insert(i, (pointer, size));

            let index = rng.next_u64_max((pointers.len() - 1) as u64) as usize;
            if rng.next_bool() {
                let (index, _) = pointers.iter().nth(index).unwrap();
                let index = *index;

                if rng.next_bool() {
                    let new_size = random_alloc_size(&mut rng);
                    let pointer = vec![0; new_size];
                    pointers.insert(index, (pointer, new_size));
                } else {
                    pointers.remove(&index);
                }
            }
        }
    }
}
