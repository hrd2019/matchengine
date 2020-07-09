use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;

#[derive(Debug, Copy, Clone)]
pub enum Asset {
    A(i64),
    B(i64),
    C(i64),
}

pub const AssetA: Asset = Asset::A(1000);
pub const AssetB: Asset = Asset::B(1000);

#[derive(Debug, Copy, Clone)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Copy, Clone)]
pub struct Odr {
    pub asset: Asset,
    pub pc: f64,
    pub qty: f64,
    pub side: Side,
    pub ts: SystemTime,
}

// impl Copy for Odr {}
//
// impl Clone for Odr {
//     fn clone(&self) -> Odr {
//         *self
//     }
// }

impl Odr {
    pub fn new(asset: Asset, pc: f64, qty: f64, side: Side) -> Odr {
        Odr {
            asset,
            pc,
            qty,
            side,
            ts: SystemTime::now(),
        }
    }
}

type qty = f64;

#[derive(Debug)]
pub struct Queue {
    pub pcs: BTreeMap<i64, qty>,
    pub odrs: HashMap<i64, Box<Vec<Odr>>>,
    pub side: Side,
    pub pc: i64,
}

impl Queue {
    pub fn new(s: Side) -> Queue {
        Queue {
            pcs: BTreeMap::new(),
            odrs: HashMap::new(),
            side: s,
            pc: 0,
        }
    }

    pub fn insert(&mut self, odr: Odr) {
        let ac = get_accuracy(&odr.asset);
        let k = (odr.pc * ac as f64) as i64;
        let item = self.pcs.entry(k).or_insert(0.0);
        *item += odr.qty;

        let item = self.odrs.entry(k).or_insert(Box::new(vec![]));
        item.push(odr);

        // for i in self.pcl.iter() {
        // match self.side {
        //     Side::Bid => {
        //         self.sort();
        //     }
        //     Side::Ask => {
        //         self.sort();
        //     }
        //     _ => {} // }
        // }
    }

    // fn sort(&mut self) {
    //     let x = self.pcl.as_mut();
    //     for i in 1..x.len() {
    //         let tmp = x[i];
    //         let mut j = i;
    //         while j > 0 && tmp < x[j - 1] {
    //             x[j] = x[j - 1];
    //             j -= 1;
    //         }
    //         x[j] = tmp;
    //     }
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        println!("test");

        let mut q = Queue::new(Side::Bid);

        let o1 = Odr::new(AssetA, 1.2, 0.45, Side::Bid);
        let o2 = Odr::new(AssetA, 1.3, 0.45, Side::Bid);
        let o3 = Odr::new(AssetA, 1.1, 0.45, Side::Bid);
        let o4 = Odr::new(AssetA, 1.1, 1.45, Side::Bid);
        let o5 = Odr::new(AssetA, 1.05, 1.45, Side::Bid);

        q.insert(o1);
        q.insert(o2);
        q.insert(o3);
        q.insert(o4);
        q.insert(o5);

        println!("{:#?}\n{:#?}\n{:#?}", q.odrs, q.pcs, q.pcl);
    }
}

pub fn get_accuracy(asset: &Asset) -> i64 {
    let ac: i64 = match *asset {
        Asset::A(x) => x,
        Asset::B(x) => x,
        _ => 100,
    };

    ac
}
