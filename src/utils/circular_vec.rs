use std::slice::Iter;

pub struct RingVec<T: Sized> {
    data: Vec<T>,
    size: usize,
    next_push_index: usize,
}

impl <T> RingVec<T> {
    pub fn new(size: usize) -> Self {
        return RingVec {
            data: Vec::<T>::with_capacity(size),
            size,
            next_push_index: 0,
        };
    }

    pub fn push(&mut self, value: T) {
        if self.size == 0 {
            return;
        }

        if self.next_push_index >= self.data.len() {
            self.data.push(value);
        } else {
            #[allow(unused_must_use)] // the returned value isn't needed
            #[allow(clippy::indexing_slicing)] // tested
            {
                std::mem::replace(&mut self.data[self.next_push_index], value);
            }
        }

        self.next_push_index = self.next_push_index.wrapping_add(1) % self.size;
    }

    pub fn iter(&self) -> Iter<T> {
        return self.data.iter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let mut vec = RingVec::<i8>::new(3);
        assert!(vec.data.is_empty());

        vec.push(11);
        assert_eq!(vec.data, vec![11]);

        vec.push(13);
        assert_eq!(vec.data, vec![11, 13]);

        vec.push(15);
        assert_eq!(vec.data, vec![11, 13, 15]);

        vec.push(17);
        assert_eq!(vec.data, vec![17, 13, 15]);

        vec.push(19);
        assert_eq!(vec.data, vec![17, 19, 15]);

        vec.push(21);
        assert_eq!(vec.data, vec![17, 19, 21]);

        vec.push(23);
        assert_eq!(vec.data, vec![23, 19, 21]);
    }

    #[test]
    fn test_push_empty() {
        let mut vec = RingVec::<i8>::new(0);
        assert!(vec.data.is_empty());

        vec.push(11);
        assert!(vec.data.is_empty());

        vec.push(13);
        assert!(vec.data.is_empty());
    }
}
