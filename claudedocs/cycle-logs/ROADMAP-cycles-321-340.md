# Roadmap: Cycles 321-340 — Vec<T> Maturity + HashMap<K,V>

> **Sprint theme**: Transform the generic Vec<T> foundation (cycles 311-320) into
> a usable dynamic container, build HashMap<K,V> on top of it, and migrate the
> first real benchmark to use the stdlib generic containers.
>
> **Carry-forward obligations** (from cycle-311-320):
> - `vec_grow` / `realloc` integration (auto-growing vec_push)
> - HashMap<K, V> on top of Vec
> - Migrate `binary_trees` to Vec<T> and measure vs raw pointer
> - Type-inference for generic constructors with no value parameters
> - `BMB_RUNTIME_PATH` auto-discovery fragility

## Phase A: Vec<T> Maturity (Cycles 321-325)

- **321**: `bmb_realloc` runtime + `vec_grow_i64` / auto-growing `vec_push_i64`
- **322**: `vec_pop_i64`, `vec_clear`, `vec_reserve`, `vec_free`
- **323**: `vec_swap`, `vec_reverse`, `vec_contains` — iteration helpers
- **324**: f64 specialization (vec_push_f64/vec_get_f64/vec_set_f64)
- **325**: End-to-end golden tests for full Vec surface

## Phase B: HashMap<K, V> (Cycles 326-330)

- **326**: `bmb_hash_i64` primitive + stdlib `core/hash.bmb`
- **327**: `HashMap<K,V>` struct skeleton with open addressing probe loop
- **328**: `hashmap_put_i64_i64` / `hashmap_get_i64_i64`
- **329**: `hashmap_contains`, `hashmap_remove`, `hashmap_len`
- **330**: Golden tests + HashMap<i64, i64> end-to-end

## Phase C: Benchmark Migration (Cycles 331-335)

- **331**: `binary_trees` Vec<T> port — first version
- **332**: Raw-pointer baseline measurement
- **333**: Vec<T> vs raw-pointer performance comparison
- **334**: Second benchmark (e.g. k-nucleotide word count) using HashMap
- **335**: stdlib E2E test suite extended for vec + hashmap

## Phase D: Type Inference + Release (Cycles 336-340)

- **336**: Turbofish syntax design note (`vec_new::<i64>()`)
- **337**: Bidirectional inference for generic constructors — design
- **338**: `BMB_RUNTIME_PATH` auto-discovery fix (surface search path in errors)
- **339**: Legacy `core::option` / `core::result` deprecation decision
- **340**: Version bump v0.97.4 + ROADMAP update + release notes

## Success Criteria

- ✅ `vec_push_i64` auto-grows via realloc — push past initial capacity works
- ✅ `HashMap<i64, i64>` basic put/get/contains operates correctly
- ✅ At least one real benchmark or golden test migrated to Vec<T>
- ✅ All existing tests pass (zero regressions)
- ✅ Each cycle logs defects honestly and resolves actionable items in-cycle
