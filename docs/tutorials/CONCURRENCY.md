# Concurrency in BMB

BMB provides structured concurrency primitives for safe parallel programming. All concurrency features require native compilation (`bmb build`), not the interpreter.

## Threads

### Spawn and Join

Use `spawn { ... }` to create a new thread:

```bmb
fn main() -> i64 = {
    let t = spawn { 42 };    // spawn a thread that returns 42
    let result = t.join();    // wait for thread, get return value
    println(result);          // 42
    0
};
```

Threads can capture variables from the enclosing scope:

```bmb
fn main() -> i64 = {
    let x: i64 = 10;
    let t = spawn { x + 1 };   // captures x
    let result = t.join();
    println(result);            // 11
    0
};
```

### Thread Type

`Thread<T>` represents a handle to a spawned thread that returns type `T`.

## Mutex

Protect shared data with `Mutex<T>`:

```bmb
fn main() -> i64 = {
    let counter = Mutex::new(0);

    let t1 = spawn {
        let val = counter.lock();
        // val is exclusively accessed here
        counter.unlock(val + 1)
    };

    let t2 = spawn {
        let val = counter.lock();
        counter.unlock(val + 1)
    };

    t1.join();
    t2.join();

    let final_val = counter.lock();
    println(final_val);   // 2
    0
};
```

## Channels

Message-passing concurrency with `Channel<T>`:

```bmb
fn main() -> i64 = {
    let (tx, rx) = channel();

    let producer = spawn {
        tx.send(42);
        tx.send(100);
        tx.close()
    };

    let v1 = rx.recv();   // 42
    let v2 = rx.recv();   // 100
    println(v1 + v2);     // 142

    producer.join();
    0
};
```

## Atomics

Lock-free integer operations with `Atomic<i64>`:

```bmb
fn main() -> i64 = {
    let counter = Atomic::new(0);

    let t1 = spawn { counter.fetch_add(1) };
    let t2 = spawn { counter.fetch_add(1) };

    t1.join();
    t2.join();

    let result = counter.load();
    println(result);    // 2
    0
};
```

## Arc (Atomic Reference Counting)

Share ownership across threads with `Arc<T>`:

```bmb
fn main() -> i64 = {
    let shared = Arc::new(42);
    let s2 = shared.clone();

    let t = spawn {
        let val = s2.get();
        println(val)      // 42
    };

    t.join();
    0
};
```

## Thread Pool

Execute tasks in parallel on a fixed pool of worker threads:

```bmb
fn main() -> i64 = {
    let pool = ThreadPool::new(4);   // 4 worker threads

    pool.execute(fn() { println(1) });
    pool.execute(fn() { println(2) });
    pool.execute(fn() { println(3) });

    pool.join();      // wait for all tasks
    pool.shutdown();  // clean up
    0
};
```

## Design Philosophy

BMB's concurrency model follows **Performance > Everything**:

| Feature | Purpose | Overhead |
|---------|---------|----------|
| `spawn` | OS-level threads | Minimal (no green thread runtime) |
| `Mutex` | Exclusive access | OS mutex (no overhead beyond kernel) |
| `Atomic` | Lock-free counters | Hardware atomic instructions |
| `Channel` | Message passing | Lock-free ring buffer |
| `Arc` | Shared ownership | Atomic reference count |

All primitives map directly to OS/hardware facilities. No runtime scheduler, no garbage collector, no hidden costs.

## Note

Concurrency features require **native compilation**:

```bash
bmb build program.bmb -o program
./program
```

The interpreter (`bmb run`) does not support concurrency — it returns an error directing you to use `bmb build`.
