use matchengine::{Odr, Query, Side};

fn main() {
    println!("Hello, world!");

    let mut q = Query::new(Side::Bid);

    let o1 = Odr::new(1.2, 0.45, Side::Bid, 5);
    let o2 = Odr::new(1.3, 0.45, Side::Bid, 5);

    q.insert(o1);
    q.insert(o2);

    println!("{:#?}", q.odrs);
}
