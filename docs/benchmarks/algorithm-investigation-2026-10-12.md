# Evaluator algorithm investigation and proposed fair benchmarks

Date: 2026-10-12  
Repository revision inspected: `a0d34c8` (the evaluator itself is unchanged from `fd04ee3`)  
Pinned competitors: `aya_poker 0.1.0`, `aya_base 0.1.0`, `aya_codegen 0.1.0`, `miniphf 0.1.0`, and `rs_poker 5.0.0`

This is a source audit and benchmark design. No performance measurement was executed and no evaluator or benchmark implementation was changed.

## Executive conclusion

The existing 2–3 ns Aya result is credible as the cost of ranking an already-accumulated Aya hand, but it is not equivalent to Eval7's unchecked call at the point where work begins. Aya has already converted and folded seven cards into a 16-byte additive key/mask representation. Its timed rank operation performs a flush test and one compact perfect-hash lookup. Eval7's timed call starts with seven one-byte cards and performs suit-mask aggregation, four-suit canonicalisation, dense formula indexing through approximately 1.3 MiB of support data, and a lookup in a 12,018,318-byte rank payload.

rs_poker is a useful middle case. The benchmark passes it a prebuilt 64-bit card bitset, but `Hand::rank()` still iterates its seven set bits and creates the same style of additive key/mask state before consulting approximately 305 KiB of generated tables and card constants. This explains why its measured workload boundary is closer to Eval7's than Aya's, while its compact tables plausibly explain its stability across working-set sizes.

The 12 MiB rank payload is the leading explanation for Eval7's large-working-set degradation, but the evidence does not isolate it. The formula indexer performs data-dependent accesses into another 1,319,072 bytes, so both structures can lose locality. Canonicalisation and formula arithmetic are likely meaningful contributors to Eval7's approximately 9–10 ns hot-set floor, but cannot by themselves explain why the same code rises to approximately 21 ns when only the working set changes.

## Sources inspected

Eval7:

- `src/evaluator.rs:39-55`
- `src/canonical.rs:5-23`
- `src/formula.rs:12-140`
- `src/table.rs:3-23`
- `src/card.rs:27-68`

Pinned Cargo registry sources:

- `aya_base-0.1.0/src/card.rs:12-28`
- `aya_base-0.1.0/src/hand.rs:11-153`
- `aya_base-0.1.0/src/constants.rs:12-50`
- `aya_poker-0.1.0/src/standard.rs:27-33`
- `aya_poker-0.1.0/src/lib.rs:146-170`
- `aya_poker-0.1.0/build.rs:11-29`
- `aya_codegen-0.1.0/src/standard.rs:14-53, 56-200`
- `miniphf-0.1.0/src/lib.rs:7-158`
- `rs_poker-5.0.0/src/core/hand.rs:5-92`
- `rs_poker-5.0.0/src/core/card_bit_set.rs:361-384`
- `rs_poker-5.0.0/src/core/rank.rs:243-357`
- `rs_poker-5.0.0/src/core/eval.rs:1-35`
- `rs_poker-5.0.0/build.rs:221-319, 598-696`

## Aya's representation and preprocessing

### Card representation

Verified: `aya_base::Card` contains two `u64` fields, `key` and `mask`, so it occupies 16 bytes under the inspected target ABI. `Card::new(rank, suit)` indexes a static 52-element `CARDS` array rather than calculating those fields at runtime.

For a card of rank `r` and suit `s`, the generated constants encode:

- a rank-histogram contribution, `RANKS[r]`, in the low 32 bits of `key`;
- one hand-card-count increment at bit 32;
- one suit-count increment in a four-bit nibble beginning at bit 48;
- one card bit at `16 * suit + rank` in `mask`.

The 13 rank multipliers are chosen so the sum uniquely identifies every reachable rank multiset of up to seven cards. The mask uses four 16-bit lanes, of which 13 bits per lane represent ranks.

### Hand representation

Verified: `aya_base::Hand` is also exactly two `u64`s:

```text
key  = additive rank key + total-card count + four biased suit counters
mask = four suit-separated 13-bit rank masks
```

`Hand::new()` starts the four suit-count nibbles at three. Adding a fifth card of a suit raises that nibble to eight, setting its high bit. Consequently `has_flush()` is one mask operation against the four high bits.

`insert_unchecked()` does exactly two state updates in source:

```text
hand.key  += card.key
hand.mask |= card.mask
```

The additions form a dependency chain when seven cards are inserted into one hand, although an optimizer may schedule the card loads and mask work around that chain.

For a flush, `flush_key()` identifies the suit whose biased count has its high bit set and shifts that suit's 13-bit rank pattern out of `mask`. For a non-flush, `rank_key()` simply returns the low 32 bits of the already-accumulated key.

Aya also implements a four-mask suit-canonical key with a five-comparison sorting network, but standard `poker_rank()` does not use it. The standard evaluator uses only the rank-histogram key or the flushed-suit pattern.

### Work completed before `poker_rank()`

Verified for the comparison suite: `aya_poker_hand(ids)` performs, outside the timed loop:

1. seven ID-to-rank and ID-to-suit conversions;
2. seven static `Card` table lookups;
3. construction of an empty `Hand`;
4. seven unchecked key additions and mask ORs.

The inputs are known-valid, so `insert_unchecked()` is appropriate for that benchmark, but it performs no duplicate or hand-length checks. Aya's `Extend<Card>` path uses the same insertions and then asserts that the accumulated count equals the mask popcount and that the hand has at most seven cards.

The existing `convert_one/aya_poker` result measures one raw-ID-to-Card conversion, but the suite does not isolate construction of a complete Aya hand. `convert_full_native_set` constructs representations for all four libraries together and therefore cannot identify Aya's seven-card construction cost.

Source-level expectation, not a measurement: complete Aya construction is small but not free. It requires seven public card conversions/table loads and fourteen accumulated state operations before the 2–3 ns lookup begins. Compiler inlining and unrolling may reduce loop overhead, so multiplying the single-card conversion result by seven would not be a reliable estimate.

## Aya's perfect hash

### Keys and table generation

Verified: `aya_codegen::PokerLookup` first constructs ordinary build-time hash maps:

- 76,155 rank-histogram keys and their best five-card `u16` values;
- 4,719 flush-pattern keys and their best five-card `u16` values.

The generator enumerates valid five-card patterns, then all valid six- and seven-card rank multisets. It evaluates the five-card subpatterns at build time and stores the maximum result for each larger multiset. Flushes are separated because a non-flush result depends only on rank multiplicities, whereas a flush depends only on the 13-bit rank pattern of the flushed suit.

`aya_poker` invokes `miniphf::build_phf_map` from its build script with a 0.95 load factor. `miniphf` implements a PTHash-style scheme:

1. assign each key to `key % bucket_count`;
2. process the largest buckets first;
3. search for a per-bucket `u32` pilot;
4. place each key at `(key XOR pilot) % value_count` without collisions;
5. emit only the dense `u16` value array and `u32` pilot array—keys are not retained.

At runtime, `MiniPhf::get()` performs one pilot lookup and one value lookup. `quickdiv::DivisorU64` stores precomputed divisors for the two modulo operations.

### Exact hold'em layout in this build

| Table | `u16` values | `u32` pilots | Payload bytes |
|---|---:|---:|---:|
| Non-flush ranks | 80,165 | 14,280 | 217,450 |
| Flush ranks | 4,969 | 984 | 13,874 |
| Total | 85,134 | 15,264 | 231,324 |

There is small additional static metadata for four slice words and four quick-division descriptors across the two `MiniPhf` values. The table payload itself is approximately 226 KiB.

The compactness is primarily algorithmic, not just a better compression of Eval7's table. Aya avoids representing every suit-canonical seven-card arrangement. It represents only rank multisets for non-flushes and rank-presence patterns for flushes, then uses a high-load perfect hash to omit unused key-space cells.

## Is Aya's 2–3 ns result comparable?

Two answers are required:

- It is comparable as a native-representation lookup benchmark: each library receives its preferred prebuilt public hand type and returns a complete comparable strength. In an application that permanently stores Aya's accumulated `Hand`, the measurement represents real marginal ranking cost.
- It is not equivalent as a complete “seven cards to rank” benchmark. Aya's rank/suit aggregation and flush state were computed before timing. Eval7 receives seven independent `Card` bytes and performs aggregation, canonicalisation, formula indexing, and table access during timing. rs_poker receives a prebuilt card bitset but still performs seven-card accumulation during timing.

The 2–3 ns figure survived the elimination audit: hands are runtime data, the input and output are black-boxed, and the general generated lookup functions contain no fixture-specific answers. Its magnitude is plausible for one branch plus two compact dependent table reads when the data is cache-resident. It should not be presented as a complete raw-card evaluation cost until the omitted construction work is measured.

## Why rs_poker is stable

Verified: the benchmark's `rs_poker::Hand` is a one-`u64` `CardBitSet`. `Hand::rank()` does not contain a cached rank key. It iterates the seven set bits using `trailing_zeros`, clears each visited bit, converts it to a `Card`, and folds the card into a 16-byte `SevenCardAccum`. Each fold adds a generated per-card key and ORs a generated per-card mask.

The accumulated representation uses the same 13 rank multipliers and biased suit-count idea seen in Aya. Its runtime evaluator then:

- tests the biased flush bits;
- uses a direct 8,192-entry flush table for a flush; or
- looks up a row displacement and then a non-flush value.

The generated structures are:

| Structure | Entries | Bytes |
|---|---:|---:|
| Non-flush `u16` values | 131,072 | 262,144 |
| Flush `u16` values | 8,192 | 16,384 |
| Row-offset `u32`s | 8,192 | 32,768 |
| Per-card key/mask arrays | 64 + 64 `u64`s | 1,024 |
| Total | — | 312,320 (305 KiB) |

The non-flush perfect hash splits a rank key into a row and column. Build time places the densest rows first and finds a collision-free displacement for each row. Runtime uses power-of-two masks rather than general modulo operations.

Hypothesis supported by the layout: the roughly 305 KiB table set has much better private-cache residency than Eval7's table plus formula data. The seven-card iteration/accumulation cost is fixed and dominates enough of the approximately 10 ns total that changing the hand working set has little effect. The benchmark recorded 10.275, 10.348, and 10.402 ns at 64, 4,096, and 262,144 hands, respectively. No cache counters were collected, so residency is not directly proven.

The existing `rs_poker_shared_prefix` case is structurally different: it pre-accumulates the five-card board, copies the 16-byte state, adds two hole cards, and ranks. Its approximately 2.8 ns result demonstrates the benefit of moving five of the seven folds outside the repeated solver loop.

## Eval7 cost decomposition

Verified hot path:

1. perform seven indexed loads from a 52-entry lane-mask table and OR them into one `u64`;
2. extract four 13-bit suit masks;
3. sort those four masks and repack the canonical key;
4. unpack them again in the formula indexer;
5. perform four indexed `u8` popcount-table loads;
6. perform up to eight indexed `u32` tail-table loads and the dense-index arithmetic;
7. load and little-endian-decode one `u16` rank from the 12 MiB payload.

The formula indexer's exact requested heap storage is:

- 8,192 bytes for mask popcounts;
- 1,310,880 bytes for 40,965 rows of eight `u32` tail counts;
- 1,319,072 bytes total, excluding allocator metadata.

### What the current measurements establish

- Small or repeatedly reused working sets place Eval7 unchecked around 9.1–10.1 ns.
- The 262,144-hand deterministic traversal raises it to 21.303 ns, and the analogous batch rises from 8.399 to 17.772 ns.
- The computation is identical; only input and lookup locality change.
- The 12,018,318-byte rank payload is almost forty times the size of rs_poker's complete generated evaluator data.
- Formula indexing also makes data-dependent accesses across approximately 1.3 MiB.

Therefore the large rank payload is the primary candidate for the *working-set-dependent degradation*, but it is not conclusively the sole cause. Random formula-support accesses and translation/TLB pressure can contribute. The source and aggregate benchmark cannot assign the additional approximately 11 ns between those structures.

Canonicalisation and formula calculation are stronger candidates for Eval7's *hot-set baseline cost*. Unlike the final rank access, they perform aggregation, sorting, multiple dependent support-table reads, and arithmetic even when the rank entry is hot. Their cost is nearly working-set invariant except for branch behavior and support-table locality. No existing measurement times canonicalisation, formula indexing, and final table lookup independently, so a percentage attribution would be speculation.

## Improvements that preserve Eval7's algorithm

These are investigation candidates, not recommendations to implement before measuring them.

1. **Measure the three stages independently.** Add benchmark-only entry points for canonical-key creation, formula indexing from a precomputed key, and rank lookup from a precomputed index. Repeat across hot and large working sets. This is the highest-value next step because it separates compute cost from both support-table and rank-table locality.
2. **Collect cache and TLB counters with fixed core placement.** At minimum record L1/L2/LLC load misses, DTLB misses, cycles, and instructions for 4,096 and 262,144 hands. On the 7950X3D, run and label the V-cache and non-V-cache CCDs separately. This can confirm or reject the leading locality hypothesis.
3. **Evaluate an explicit four-value sorting network.** Eval7 currently calls `sort_unstable()` for four `u16`s. Five compare/exchange operations would preserve the exact suit-canonical ordering and may make code generation more predictable. Aya's unused `canonical_key()` demonstrates this form. Assembly and benchmarks must decide whether LLVM already produces equivalent code.
4. **Prototype a reusable canonicalisation accumulator for solver loops.** An internal state containing four unsorted suit masks could absorb known board cards once, be copied, accept two hole cards, then use the unchanged sort, formula, and rank table. This preserves the distinctive suit-canonical algorithm while removing five lane-mask loads/ORs per runout. It should begin as benchmark-only/internal work; any public API requires separate design review.
5. **Audit formula-table layout.** The tail table is rectangular and contains states that the seven-card path cannot reach. A generated compact layout grouped by `(slots, remaining)` may reduce its cache footprint while preserving the formula and ordering. Extra address arithmetic could erase the benefit, so prototype measurements are essential.
6. **Compare the popcount table with `u16::count_ones()`.** Removing the 8 KiB lookup may reduce one class of loads, but four hardware popcounts may or may not be faster. This is a local implementation experiment that leaves the indexing formula unchanged.
7. **Investigate page layout only after counters.** Alignment, large-page experiments, or batch prefetching could help a genuinely TLB- or latency-bound table, but they are platform-specific and should not precede proof of the bottleneck.

Replacing the canonical table with rank-histogram and flush tables would likely produce an Aya/rs_poker-style evaluator, but that would replace Eval7's defining method rather than improve it. It is not recommended under the current preservation goal. Reordering the 6,009,159 entries or changing the dense formula is likewise a format/algorithm change and should not be an early optimization.

## Proposed additional benchmarks

No benchmark described below has been implemented or run.

### Shared controls

- Use the same pinned crate versions, compiler, thin LTO, one codegen unit, Criterion configuration, and deterministic affine hand traversal as the completed comparison.
- Store the common input once as `Vec<[u8; 7]>`, using suit-major IDs `0..51`. Do not keep parallel native-hand vectors for the end-to-end groups.
- Use valid, distinct, sorted combinations generated before timing. Validity is a dataset invariant rather than evaluator-specific work in the trusted-input track.
- In every iteration, black-box the copied raw ID array, perform all stated conversion/preprocessing inside the timed closure, and black-box the returned rank type. Do not convert ranks to a common category inside timing.
- Run the existing semantic preflight over every fixture family before measurement. Also hash or fold all results once outside timing so every implementation is confirmed to agree on categories, ordering, and the documented tie cases.
- Retain 64, 4,096, and 262,144 working sets and 4,096/262,144 sequential batches. This preserves direct comparison with the native-representation results.
- Record per-implementation input bytes, native temporary size, validation behavior, and exactly which constructor/API is called.
- Pin execution to one documented logical core. On the 7950X3D, identify the CCD/core class and perform separate labelled runs on each CCD class. Record effective frequency if tooling permits.
- Use at least five separate process runs per core class for run-to-run uncertainty. Criterion's within-run interval is not a substitute for independent-run variation.
- Keep evaluator order stable within one run for reproducibility, but rotate the implementation order between process runs to expose order/thermal bias.

### Track 1: complete trusted-valid raw-ID pipeline

Benchmark group names:

```text
raw_end_to_end/{64|4096|262144}/{eval7|aya_poker|rs_poker|poker}
raw_batch/{4096|262144}/{eval7|aya_poker|rs_poker|poker}
```

Per-evaluation work:

| Evaluator | Work inside timing |
|---|---|
| Eval7 | Convert all seven IDs with `Card::new`; build `[Card; 7]`; call the reused `Evaluator::evaluate_7_unchecked`. |
| Aya | Convert each ID to public `Rank` and `Suit`; call `Card::new`; create `Hand`; call `insert_unchecked` seven times; call `poker_rank`. |
| rs_poker | Convert each ID with `Card::from`; create `SevenCardAccum`; add seven cards; call `rank`. This is its fastest documented accumulated evaluation path and performs no duplicate validation. |
| poker | Convert each ID to `Rank`/`Suit` and `Card`; build the seven-card array; call the reused evaluator's seven-card/best-five operation and unwrap the valid result. |

Evaluator objects and immutable generated tables are initialized before timing. Per-hand native state is not. This track answers the fairest performance question for solver-owned, already-validated raw card IDs: “starting from the same seven bytes, what is each library's fastest public complete pipeline?”

The use of `SevenCardAccum` rather than `rs_poker::Hand` is intentional. Building a bitset and then immediately iterating it to rebuild the accumulator adds a container round trip that a performance-sensitive caller can avoid. A supplementary conventional-container case should make that difference visible rather than hiding it.

### Track 2: conventional/checked container pipeline

Benchmark group:

```text
raw_checked/{4096}/{eval7|aya_poker|rs_poker|poker}
```

- Eval7 converts seven cards and calls checked `evaluate_7`.
- Aya builds via `Extend<Card>`/`FromIterator`, which checks duplicate/count consistency after insertion, then calls `poker_rank`.
- rs_poker builds a `Hand` with seven `insert` calls, verifies each returned `true`, then calls `Hand::rank()`.
- poker uses its normal public seven-card input and result validation.

This is not expected to equalize validation algorithms; it deliberately measures conventional safe public usage. Report each validation policy beside the result. Do not combine this table with Track 1 rankings.

### Construction-only diagnostic

For 4,096 raw hands, separately time production of each representation and black-box the completed value:

- Eval7 `[Card; 7]`;
- Aya accumulated `Hand`;
- rs_poker `SevenCardAccum` and, separately, `Hand`;
- poker `[Card; 7]`.

This diagnostic identifies how much work the existing native-rank benchmark placed before the timer. Do not subtract independently measured means and present the result as a statistically measured residual; show construction and end-to-end observations side by side.

### Fixed-board full rebuild

For each existing dry, paired, monotone, and connected board, enumerate all 1,081 legal hole-card pairs. Store only `[u8; 2]` hole IDs plus one shared `[u8; 5]` board. Inside every timed iteration concatenate the seven raw IDs and execute the complete Track 1 pipeline for the selected evaluator.

Group names:

```text
fixed_board_raw_full/{texture}/{evaluator}
fixed_board_raw_full_batch/{texture}/{evaluator}
```

This is the equal-work reference: nobody reuses native board state.

### Fixed-board incremental capability

Then measure a separately labelled solver-oriented capability track. Setup outside timing may convert and accumulate the fixed five-card board. The two changing hole IDs remain raw and their conversion is timed.

Group names:

```text
fixed_board_raw_incremental/{texture}/{evaluator}
fixed_board_raw_incremental_batch/{texture}/{evaluator}
```

| Evaluator | Precomputed once | Work inside each timed hole pair |
|---|---|---|
| Eval7 | Five board `Card`s | Convert two hole IDs, assemble the seven-card array with the board, run unchanged unchecked evaluation. Current API cannot reuse canonical state. |
| Aya | Accumulated five-card `Hand` | Convert two hole IDs, copy the board hand, insert two cards, call `poker_rank`. |
| rs_poker | Five-card `SevenCardAccum` | Convert two hole IDs, copy the accumulator, add two cards, call `rank`. |
| poker | Five board `Card`s and reusable evaluator | Convert two hole IDs, assemble seven cards, run its normal evaluator. |

This track intentionally allows API-supported board-state reuse. It is fair as a capability comparison, not as equal internal work, and must remain in a separate table from full rebuild. The report should state that Aya and rs_poker cache rank/suit aggregation while current Eval7 and poker only reuse card conversion.

For both fixed-board tracks, test individual cyclic traversal and one complete 1,081-hand batch. Consume every result. Compare texture effects only within the same track.

## Recommended next phase

1. Implement the raw-ID end-to-end and fixed-board full-rebuild groups first; they resolve the largest fairness ambiguity in the current report.
2. Add the incremental capability group without changing Eval7 so the current API gap is quantified honestly.
3. Add Eval7 stage-isolation benchmarks and collect hardware counters before choosing any optimization.
4. If counters confirm formula/support or rank-table locality, prototype formula-layout reduction. If compute dominates the hot case, prototype the explicit sorting network.
5. For solver workloads, a board-prefix canonical accumulator is the most distinctive and realistic optimization candidate: it preserves suit canonicalisation, the formula index, rank encoding, and embedded table while avoiding repeated work already known to be shared.

Do not prioritize replacing Eval7 with rank-histogram perfect hashing unless the project explicitly decides that preserving the existing Eval7 algorithm is no longer a goal.
