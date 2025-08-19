use std::fmt::Debug;

pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    write_pos: usize,
    count: usize,
}

impl<T: Clone + Default + Debug> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![T::default(); capacity],
            capacity,
            write_pos: 0,
            count: 0,
        }
    }

    pub fn write(&mut self, value: T) {
        self.buffer[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.capacity;

        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn read_oldest(&self) -> Option<T> {
        if self.count == 0 {
            return None;
        }

        if self.count < self.capacity {
            Some(self.buffer[0].clone())
        } else {
            Some(self.buffer[self.write_pos].clone())
        }
    }

    pub fn get_all_chronological(&self) -> Vec<T> {
        if self.count == 0 {
            return Vec::new();
        }

        if self.count < self.capacity {
            self.buffer[0..self.count].to_vec()
        } else {
            let mut result = Vec::new();
            for i in 0..self.capacity {
                let pos = (self.write_pos + i) % self.capacity;
                result.push(self.buffer[pos].clone());
            }
            result
        }
    }

    pub fn get_last_n(&self, n: usize) -> Vec<T> {
        if self.count == 0 || n == 0 {
            return Vec::new();
        }

        let items_to_get = n.min(self.count);
        let all = self.get_all_chronological();

        if all.len() >= items_to_get {
            all[all.len() - items_to_get..].to_vec()
        } else {
            all
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == self.capacity
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.count = 0;
        self.buffer = vec![T::default(); self.capacity];
    }
}

impl<T: Debug> CircularBuffer<T> {
    pub fn print_debug(&self) {
        println!("\n=== Circular Buffer Debug ===");
        println!("Buffer: {:?}", self.buffer);
        println!(
            "Capacity: {}, Count: {}, Write pos: {}",
            self.capacity, self.count, self.write_pos
        );
        println!("=============================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let cb: CircularBuffer<i32> = CircularBuffer::new(4);
        assert_eq!(cb.capacity(), 4);
        assert_eq!(cb.len(), 0);
        assert!(cb.is_empty());
        assert!(!cb.is_full());
    }

    #[test]
    fn test_write_and_read() {
        let mut cb = CircularBuffer::new(3);

        cb.write(10);
        assert_eq!(cb.read_oldest(), Some(10));
        assert_eq!(cb.len(), 1);

        cb.write(20);
        cb.write(30);
        assert_eq!(cb.read_oldest(), Some(10));
        assert!(cb.is_full());

        cb.write(40);
        assert_eq!(cb.read_oldest(), Some(20));
        assert_eq!(cb.get_all_chronological(), vec![20, 30, 40]);
    }

    #[test]
    fn test_get_last_n() {
        let mut cb = CircularBuffer::new(4);

        cb.write(10);
        cb.write(20);
        cb.write(30);
        cb.write(40);

        assert_eq!(cb.get_last_n(2), vec![30, 40]);
        assert_eq!(cb.get_last_n(10), vec![10, 20, 30, 40]);
        assert_eq!(cb.get_last_n(0), vec![]);

        cb.write(50);
        assert_eq!(cb.get_last_n(3), vec![30, 40, 50]);
    }

    #[test]
    fn test_clear() {
        let mut cb = CircularBuffer::new(3);
        cb.write(10);
        cb.write(20);

        cb.clear();
        assert!(cb.is_empty());
        assert_eq!(cb.len(), 0);
        assert_eq!(cb.read_oldest(), None);
    }

    #[test]
    fn test_with_bytes() {
        let mut cb: CircularBuffer<u8> = CircularBuffer::new(4);

        cb.write(0x50);
        cb.write(0x4b);
        cb.write(0x07);
        cb.write(0x08);

        let last_4 = cb.get_last_n(4);
        assert_eq!(last_4, vec![0x50, 0x4b, 0x07, 0x08]);
    }
}
