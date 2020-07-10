use matchengine::{Odr, OptType, Queue, Side, ASSET_A};

mod matcher;

fn main() {
    println!("Hello, world!");

    let mut q = Queue::new(Side::Bid);

    let o1 = Odr::new(1, ASSET_A, OptType::DEAL, 1.2, 0.45, Side::Bid);
    let o2 = Odr::new(2, ASSET_A, OptType::DEAL, 1.4, 0.45, Side::Bid);

    q.insert(o1);
    q.insert(o2);

    println!("{:#?}", q.odrs);
}
