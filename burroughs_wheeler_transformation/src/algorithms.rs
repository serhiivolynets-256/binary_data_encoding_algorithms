use rdxsort;
use rdxsort::{RdxSort, RdxSortTemplate};
use std::fmt::{Display, Formatter};
use tracing::debug;

pub struct TableLine<const N: usize> {
    line: [u8; N],
}

pub struct Table<const N: usize> {
    table: [[u8; N]; N],
}

#[derive(Debug, Clone)]
pub struct EnumeratedItem {
    value: u8,
    index: usize,
}

impl RdxSortTemplate for EnumeratedItem {
    fn cfg_nbuckets() -> usize {
        256
    }

    fn cfg_nrounds() -> usize {
        1
    }

    fn get_bucket(&self, round: usize) -> usize {
        self.value as usize
    }

    fn reverse(round: usize, bucket: usize) -> bool {
        false
    }
}

impl<const N: usize> Display for Table<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for line in self.table.iter() {
            writeln!(
                f,
                "{}",
                line.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )?;
        }

        Ok(())
    }
}

pub struct BurroughsWheelerEncode {}

impl Default for BurroughsWheelerEncode {
    fn default() -> Self {
        Self {}
    }
}

impl BurroughsWheelerEncode {
    pub fn encode_block<const N: usize>(block: &[u8; N]) -> Option<([u8; N], usize)>
    where
        [u8; N]: RdxSortTemplate,
        [[u8; N]]: RdxSort,
    {
        let mut table: [[u8; N]; N] = Self::gen_shift_table(block);
        Self::sort_table(&mut table);

        let last_row = Self::get_last_row(&table);

        if let Ok(i) = table.binary_search(block) {
            Some((last_row, i))
        } else {
            None
        }
    }

    fn gen_shift_table<const N: usize>(block: &[u8; N]) -> [[u8; N]; N] {
        let mut table = [[0u8; N]; N];

        for i in 0..N {
            for j in 0..N {
                table[i][j] = block[(i + j) % N];
            }
        }

        table
    }

    fn sort_table<const N: usize>(table: &mut [[u8; N]; N])
    where
        [u8; N]: RdxSortTemplate,
        [[u8; N]]: RdxSort,
    {
        table.rdxsort();
    }

    fn get_last_row<const N: usize>(table: &[[u8; N]; N]) -> [u8; N] {
        let mut result = [0u8; N];
        for i in (0..N) {
            result[i] = table[i][N - 1];
        }

        result
    }

    fn get_permutation_table<const N: usize>(block: &[u8; N]) -> [usize; N] {
        let mut enumerated_block: [EnumeratedItem; N] = std::array::from_fn(|i| EnumeratedItem {
            index: i,
            value: block[i],
        });

        enumerated_block.rdxsort();

        let mut permutation_table = [0; N];
        for (i, j) in enumerated_block.iter().enumerate() {
            permutation_table[i] = j.index;
        }

        permutation_table
    }

    pub fn encode<const N: usize>(bytes: &[u8; N]) -> ([u8; N], usize)
    where
        [u8; N]: RdxSortTemplate,
        [[u8; N]]: RdxSort,
    {
        let mut table = BurroughsWheelerEncode::gen_shift_table(bytes);
        BurroughsWheelerEncode::sort_table(&mut table);

        let encoded_word = BurroughsWheelerEncode::get_last_row(&table);
        let pos = table.binary_search(&bytes).unwrap();

        (encoded_word, pos)
    }

    pub fn decode<const N: usize>(encoded_word: &[u8; N], pos: usize) -> [u8; N] {
        let permutation_table = BurroughsWheelerEncode::get_permutation_table(&encoded_word);
        let word = BurroughsWheelerEncode::decode_word(&encoded_word, permutation_table, pos);

        word
    }

    fn decode_word<const N: usize>(
        word: &[u8; N],
        permutation_table: [usize; N],
        mut pos: usize,
    ) -> [u8; N] {
        let mut res = [0; N];

        for i in 0..N {
            pos = permutation_table[pos];
            res[i] = word[pos];
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithms::BurroughsWheelerEncode;
    use utils::init_tracing;

    #[test]
    fn test_table_creation() {
        const CHUNK_SIZE: usize = 10;

        init_tracing();

        let bytes: Vec<u8> = (0..100).map(|_| rand::random::<u8>()).collect();
        let (chunks, _) = bytes.as_chunks::<CHUNK_SIZE>();

        for word in chunks {
            let (code, pos) = BurroughsWheelerEncode::encode(&word);
            let decoded_bytes = BurroughsWheelerEncode::decode(&code, pos);

            assert_eq!(word, &decoded_bytes);
        }
    }
}
