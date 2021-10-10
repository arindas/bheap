use std::{
    cmp::Ordering,
    cmp::Ordering::Less,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};

pub struct BinaryMaxHeap<T>
where
    T: Eq + PartialEq + Hash + Ord,
{
    buffer: Vec<T>,
    index: HashMap<u64, usize>,
}

fn default_hash<T: Hash>(val: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    val.hash(&mut hasher);
    hasher.finish()
}

impl<T> BinaryMaxHeap<T>
where
    T: Eq + PartialEq + Hash + Ord,
{
    pub fn from_vec(vec: Vec<T>) -> Self {
        BinaryMaxHeap {
            buffer: vec,
            index: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        BinaryMaxHeap::from_vec(vec![])
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn swap_elems_at_indices(&mut self, i: usize, j: usize) {
        let index = &mut self.index;

        index.insert(default_hash(&self.buffer[i]), j);
        index.insert(default_hash(&self.buffer[j]), i);

        self.buffer.swap(i, j);
    }

    #[inline]
    fn cmp(&self, i: usize, j: usize) -> Ordering {
        self.buffer[i].cmp(&self.buffer[j])
    }

    fn heapify_up(&mut self, idx: usize) {
        let mut i = idx;

        while i > 0 {
            let parent = (i - 1) / 2;

            if let Ordering::Greater = self.cmp(i, parent) {
                self.swap_elems_at_indices(i, parent);
                i /= 2;
            } else {
                break;
            };
        }
    }

    fn heapify_down(&mut self, idx: usize) {
        let mut i = idx;

        while i < (self.len() / 2) {
            let mut max = i;
            let (lc, rc) = (2 * i + 1, 2 * i + 2);

            if lc < self.len() {
                if let Less = self.cmp(max, lc) {
                    max = lc;
                }
            }

            if rc < self.len() {
                if let Less = self.cmp(max, rc) {
                    max = rc;
                }
            }

            if i != max {
                self.swap_elems_at_indices(i, max);
                i = max;
            } else {
                break;
            }
        }
    }

    #[inline]
    fn update_index(&mut self, i: usize) {
        self.index.insert(default_hash(&self.buffer[i]), i);
    }

    pub fn push(&mut self, elem: T) {
        let idx = self.buffer.len();

        self.buffer.push(elem);
        self.update_index(idx);

        self.heapify_up(idx);
    }

    pub fn peek(&mut self) -> Option<&T> {
        if self.buffer.is_empty() {
            return None;
        }

        Some(&self.buffer[0])
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.buffer.is_empty() {
            return None;
        }

        let elem = self.buffer.swap_remove(0);
        self.index.remove(&default_hash(&elem));
        self.heapify_down(0);

        Some(elem)
    }

    pub fn build_index(&mut self) {
        (0..self.len()).for_each(|i| self.update_index(i));
    }

    pub fn build_heap(&mut self) {
        self.build_index();

        for i in (0..(self.len() / 2)).rev() {
            self.heapify_down(i);
        }
    }

    pub fn reprioritize_element(&mut self, elem: &T) -> Option<()> {
        let i = *self.index.get(&default_hash(elem))?;

        if let Ordering::Greater = self.cmp(i, (i - 1) / 2) {
            self.heapify_up(i)
        } else {
            self.heapify_down(i)
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
