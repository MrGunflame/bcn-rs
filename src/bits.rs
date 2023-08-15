#[derive(Clone, Debug)]
pub(crate) struct BitReader<const N: usize> {
    state: [u8; N],
    cursor: usize,
}

impl<const N: usize> BitReader<N> {
    pub fn new(block: [u8; N]) -> Self {
        Self {
            state: block,
            cursor: 0,
        }
    }

    pub fn read(&mut self, bits: usize) -> u32 {
        let mut acc = 0;

        for i in self.cursor..self.cursor + bits {
            let index = i / 8;
            let shift = 7 - (i % 8);
            let bit = (self.state[index] as u32 >> shift) & 1;
            acc = (acc << 1) | bit;
        }

        self.cursor += bits;

        acc
    }
}
