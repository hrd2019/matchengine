use matchengine::{Odr, Queue, Side};

mod matcher;

fn main() {
    println!("Hello, world!");

    let mut q = Queue::new(Side::Bid);

    let o1 = Odr::new(1.2, 0.45, Side::Bid, 5);
    let o2 = Odr::new(1.4, 0.45, Side::Bid, 5);

    q.insert(o1);
    q.insert(o2);

    println!("{:#?}", q.odrs);
}
