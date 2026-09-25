fn run() {
    let mut c = Counter::default();
    println!("{}", c.inc());
    println!("{}", c.get());

    foo(&mut c);

    // Compiler error!
    // {
    //     bar(
    //         &mut c,
    //         &mut c // 💥 COMPILER ERROR IS HERE (cannot borrow `c` a 2nd time)
    //     );
    // }

    // Compiler error!
    // {
    //     let r1 = &mut c; // 1st mut borrow starts
    //     let r2 = &mut c; // 💥 COMPILER ERROR IS HERE (cannot borrow `c` a 2nd time)
    //     r2.inc();        // ❌ NEVER RUNS (code doesn't compile)
    //     r1.inc();        // Reason: r1 was still alive above (this is not where error occurs)
    // }
}

pub fn foo(c_mut: &mut Counter) {
    c_mut.inc();
}

pub fn bar(c_mut_1: &mut Counter, c_mut_2: &mut Counter) {
    c_mut_1.inc();
    c_mut_2.inc();
}

#[derive(Default, Debug)]
pub struct Counter {
    count: u8,
}

impl Counter {
    pub fn get(&self) -> u8 {
        self.count
    }
    pub fn inc(&mut self) -> u8 {
        self.count += 1;
        self.count
    }
}

