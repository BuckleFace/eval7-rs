//! Lexicographic ranking of thirteen multiplicities in 0..=4 summing to seven.

pub(crate) const NON_FLUSH_ENTRIES: usize = 49_205;
pub(crate) const FLUSH_ENTRIES: usize = 8_192;

// W(n,k) counts n remaining rank digits with total k. The empty suffix has
// W(0,0)=1; W(n,k)=sum_{d=0..min(4,k)} W(n-1,k-d).
// P(n,k,c)=sum_{d<c} W(n,k-d) counts the suffixes skipped by choosing digit c.
// Summing P(12-rank,remaining,count) in deuce-to-ace order gives the exact
// lexicographic ordinal. These disjoint predecessor counts fill 0..49_205.
const fn block_prefix_counts() -> [[u16; 4096]; 4] {
    let mut ways = [[0u32; 8]; 14];
    ways[0][0] = 1;
    let mut n = 1;
    while n <= 13 {
        let mut k = 0;
        while k <= 7 {
            let mut d = 0;
            while d <= 4 && d <= k {
                ways[n][k] += ways[n - 1][k - d];
                d += 1;
            }
            k += 1;
        }
        n += 1;
    }
    assert!(ways[13][7] as usize == NON_FLUSH_ENTRIES);

    // A block combines three original P terms, updating k locally. Its address
    // packs three base-8 count digits below the three-bit incoming total k.
    // Counts >4 or sums >k are impossible for valid hands; leave those slots zero.
    let mut table = [[0; 4096]; 4];
    let mut block = 0;
    while block < 4 {
        let mut address = 0;
        while address < 4096 {
            let mut remaining = address >> 9;
            let mut total = 0u32;
            let mut valid = true;
            let mut digit = 0;
            while digit < 3 {
                let count = (address >> (3 * digit)) & 7;
                if count > 4 || count > remaining {
                    valid = false;
                    break;
                }
                let mut smaller = 0;
                while smaller < count {
                    total += ways[12 - (3 * block + digit)][remaining - smaller];
                    smaller += 1;
                }
                remaining -= count;
                digit += 1;
            }
            if valid {
                assert!(total <= u16::MAX as u32);
                table[block][address] = total as u16;
            }
            address += 1;
        }
        block += 1;
    }
    table
}

// Four blocks * eight incoming totals * 512 digit tuples * two bytes = 32 KiB.
static BLOCK_PREFIX: [[u16; 4096]; 4] = block_prefix_counts();

/// Requires thirteen three-bit count lanes in 0..=4 whose total is seven.
#[inline]
pub(crate) fn non_flush_index(counts: u64) -> usize {
    // Multiplication by 1+8+...+8^12 forms inclusive prefix sums. All partial
    // sums are <=7, so no base-8 carry occurs. Complementing each three-bit
    // digit gives seven minus that sum: the total remaining AFTER that rank.
    // Wrapping discards only unused high product bits. Blocks can consequently
    // calculate their addresses independently, without a loop-carried total.
    let remaining = !counts.wrapping_mul(((1u64 << 39) - 1) / 7);
    let mut index = 0;
    for (block, entries) in BLOCK_PREFIX.iter().enumerate() {
        let incoming = if block == 0 {
            7
        } else {
            ((remaining >> (9 * block - 3)) & 7) as usize
        };
        let digits = ((counts >> (9 * block)) as usize) & 511;
        index += entries[(incoming << 9) | digits] as usize;
    }
    // The four blocks cover deuce through king. The ace count c is then forced
    // to equal the remaining total k, and P(0,c,c)=0 since W(0,c-d)=0 for d<c.
    // Omitting that zero term and grouping the others preserves every ordinal.
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_count_vector_has_its_enumeration_ordinal() {
        fn visit(rank: usize, remaining: u8, packed: u64, ordinal: &mut usize) {
            if rank == 13 {
                if remaining == 0 {
                    assert_eq!(non_flush_index(packed), *ordinal, "counts={packed:#x}");
                    *ordinal += 1;
                }
                return;
            }
            for count in 0..=remaining.min(4) {
                visit(
                    rank + 1,
                    remaining - count,
                    packed | (u64::from(count) << (3 * rank)),
                    ordinal,
                );
            }
        }
        let mut ordinal = 0;
        visit(0, 7, 0, &mut ordinal);
        assert_eq!(ordinal, NON_FLUSH_ENTRIES);
    }
}
