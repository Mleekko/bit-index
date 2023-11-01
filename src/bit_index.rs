use scrypto::prelude::*;

const SET_BITS: [u8; 256] = [0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8];

/// Most operations are O(n) but we assume a relatively small size (<10k), and limited
/// number of operations in a transaction.
/// Can grow in size but cannot shrink.
#[derive(ScryptoSbor)]
pub struct BitIndex {
    data: Vec<u8>,
    /// idx from which a search for the next empty slot should start
    cursor: usize,
    /// the number of bits flipped to 1
    size: usize,
}

impl BitIndex {
    pub fn new() -> BitIndex {
        Self { data: Vec::new(), cursor: 0, size: 0 }
    }

    /// How many elements are flipped to 1 at the moment
    pub fn size(&self) -> usize {
        return self.size;
    }

    /// How many elements we can hold without growing the size
    pub fn total_size(&self) -> usize {
        return self.data.len() * 8;
    }


    /// Sets the next empty slot to 1, and returns its index
    pub fn reserve_slot(&mut self) -> usize {
        let total_size = self.total_size();
        if self.size == total_size {
            // grow
            self.data.push(0b00000001);
            self.cursor = total_size + 1;
            self.size += 1;
            return total_size;
        }
        loop {
            let curr = self.cursor;

            // move the cursor. Both when preparing for the next iteration, and when this function returns.
            if curr == total_size - 1 {
                self.cursor = 0;
            } else {
                self.cursor += 1;
            }

            // check if the slot is empty
            let data_idx = curr / 8;
            let byte = self.data[data_idx];

            if byte == 0xff { // all full - skip!
                continue;
            }

            let bit_idx = curr % 8;
            let bit_val = (byte >> bit_idx) & 1;

            if bit_val == 0 {
                // flip to 1
                self.data[data_idx] = byte | (1 << bit_idx);
                self.size += 1;
                return curr;
            }
        }
    }

    /// Removes the specific element (unsets the bit).
    pub fn remove(&mut self, idx: usize) {
        let data_idx = idx / 8;
        let bit_idx = idx % 8;
        let byte = self.data[data_idx];
        let bit_val = (byte >> bit_idx) & 1;

        if bit_val == 1 {
            // flip to 0
            self.data[data_idx] = byte & (0b11111111 ^ (1 << bit_idx));
            self.size -= 1;
        }

    }


    /// Finds the n-th (0-based index) set bit in BitIndex.
    /// `ordinal` should be strictly less than `size()`.
    pub fn find_idx_by_ordinal(&self, ordinal: usize) -> usize {
        let mut data_idx: usize = 0;
        let mut size: usize = 0;
        let mut prev_size: usize = 0;
        loop {
            let byte = self.data[data_idx];
            size += SET_BITS[byte as usize] as usize;
            if size > ordinal {
                // this is our byte, now get the bit
                let mut i: usize = 0;
                while i < 8 {
                    let bit_val = (byte >> i) & 1;
                    if bit_val == 1 {
                        if prev_size == ordinal {
                            return data_idx * 8 + i;
                        }
                        prev_size += 1;
                    }
                    i += 1;
                }
            }
            prev_size = size;
            data_idx += 1;
        }
    }

    /// Finds the next set bit at or after `start_idx`.
    /// Returns the slot idx, or `size` if there are no set bits after the given index.
    pub fn find_next(&self, start_idx: usize) -> usize {
        let mut idx: usize = start_idx;
        while idx < self.size {
            let data_idx = idx / 8;
            let byte = self.data[data_idx];

            if byte == 0 { // no set bits - skip!
                idx = (data_idx + 1) * 8;
                continue;
            }

            // this is our byte, now get the bit
            let mut bit_idx = idx % 8;
            while bit_idx < 8 {
                let bit_val = (byte >> bit_idx) & 1;

                if bit_val == 1 {
                    return data_idx * 8 + bit_idx;
                }
                bit_idx += 1;
            }
        }
        return self.size;
    }


    /// Mark an element as present/absent
    pub fn set(&mut self, idx: usize, value: bool) {}

    /// Add a range. from - inclusive, to - exclusive.
    pub fn add_range(&mut self, from: usize, to: usize) {}


}