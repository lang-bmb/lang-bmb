// Rust translation of tests/run/methods.bmb (backlog 2M-01), for its
// expected output (`rustc --edition 2021`).
#[derive(Clone, Copy)]
struct Account {
    id: u64,
    balance: i64,
}

struct Log {
    moves: Vec<i64>,
}

impl Account {
    fn open(id: u64) -> Account {
        Account { id, balance: 0 }
    }

    fn balance(&self) -> i64 {
        self.balance
    }

    fn deposit(&mut self, x: i64) -> bool {
        if x <= 0 || self.balance + x > 1000000 {
            return false;
        }
        self.balance += x;
        true
    }

    fn withdraw(&mut self, x: i64) -> bool {
        if x <= 0 || x > self.balance {
            return false;
        }
        self.balance -= x;
        true
    }
}

impl Log {
    fn new() -> Log {
        Log { moves: Vec::new() }
    }

    fn record(&mut self, ok: bool, x: i64) {
        if ok && self.moves.len() < 1000 {
            self.moves.push(x);
        }
    }

    fn count(&self) -> usize {
        self.moves.len()
    }

    fn close(self) -> Vec<i64> {
        self.moves
    }
}

trait Rank {
    fn rank(&self) -> u64;

    fn better(&self, other: &Self) -> bool {
        self.rank() > other.rank()
    }
}

impl Rank for Account {
    fn rank(&self) -> u64 {
        u64::try_from(self.balance / 100).unwrap()
    }
}

impl Rank for u64 {
    fn rank(&self) -> u64 {
        *self % 10
    }
}

fn top<T: Copy + Rank>(xs: &[T]) -> u64 {
    let mut best = 0;
    for x in xs {
        let r = x.rank();
        if r > best {
            best = r;
        }
    }
    best
}

fn flag(b: bool) -> i64 {
    if b { 1 } else { 0 }
}

fn main() {
    let mut a = Account::open(1);
    let mut b = Account::open(2);
    let mut log = Log::new();
    let ok = a.deposit(500);
    log.record(ok, 500);
    let ok2 = a.withdraw(800);
    log.record(ok2, -800);
    let ok3 = a.withdraw(120);
    log.record(ok3, -120);
    println!("{}", flag(ok) + flag(ok2) * 10 + flag(ok3) * 100);
    println!("{}", flag(b.deposit(999999)));
    println!("{}", flag(b.deposit(2)));
    println!("{}", a.balance());
    println!("{}", b.balance());
    println!("{}", a.rank());
    println!("{}", flag(b.better(&a)));
    println!("{}", flag(a.better(&b)));
    let digits: [u64; 4] = [12, 7, 45, 31];
    println!("{}", top(&digits));
    println!("{}", digits[1].rank());
    println!("{}", log.count());
    println!("{}", b.id);
    for m in log.close() {
        println!("{m}");
    }
}
