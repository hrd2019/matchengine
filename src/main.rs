use matchengine::{Odr, Queue, Side, ASSET_A};

mod matcher;

fn main() {
    println!("Hello, world!");

    let mut q = Queue::new(Side::Bid);

    let o1 = Odr::new(ASSET_A, 1.2, 0.45, Side::Bid);
    let o2 = Odr::new(ASSET_A, 1.4, 0.45, Side::Bid);

    q.insert(o1);
    q.insert(o2);

    println!("{:#?}", q.odrs);
}
