use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;

pub enum Asset {
    A,
    B,
    C,
}

#[derive(Debug, Copy, Clone)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Copy, Clone)]
pub struct Odr {
    pc: f64,
    qty: f64,
    side: Side,
    ts: SystemTime,
    arcy: i64,
}

impl Odr {
    pub fn new(pc: f64, qty: f64, side: Side, arcy: i64) -> Odr {
        Odr {
            pc,
            qty,
            side,
            arcy,
            ts: SystemTime::now(),
        }
    }
}

type qty = f64;

#[derive(Debug)]
pub struct Queue {
    pcs: BTreeMap<String, qty>,
    pub odrs: HashMap<String, Box<Vec<Odr>>>,
    pcl: Box<Vec<f64>>,
    side: Side,
    pc: f64,
}

impl Queue {
    pub fn new(s: Side) -> Queue {
        Queue {
            pcs: BTreeMap::new(),
            odrs: HashMap::new(),
            pcl: Box::new(vec![]),
            side: s,
            pc: 0.0,
        }
    }

    pub fn insert(&mut self, odr: Odr) {
        let item = self
            .pcs
            .entry(String::from(odr.pc.to_string()))
            .or_insert(0.0);
        *item += odr.qty;

        let v = odr.pc * odr.arcy as f64;

        let item = self
            .odrs
            .entry(odr.pc.to_string())
            .or_insert(Box::new(vec![]));
        item.push(odr);

        self.pcl.push(v);

        // for i in self.pcl.iter() {
        match self.side {
            Side::Bid => {
                self.sort();
            }
            Side::Ask => {
                self.sort();
            }
            _ => {} // }
        }
    }

    fn sort(&mut self) {
        let x = self.pcl.as_mut();
        for i in 1..x.len() {
            let tmp = x[i];
            let mut j = i;
            while j > 0 && tmp < x[j - 1] {
                x[j] = x[j - 1];
                j -= 1;
            }
            x[j] = tmp;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        println!("test");

        let mut q = Queue::new(Side::Bid);

        let o1 = Odr::new(1.2, 0.45, Side::Bid, 5);
        let o2 = Odr::new(1.3, 0.45, Side::Bid, 5);
        let o3 = Odr::new(1.1, 0.45, Side::Bid, 5);
        let o4 = Odr::new(1.1, 1.45, Side::Bid, 5);
        let o5 = Odr::new(1.05, 1.45, Side::Bid, 5);

        q.insert(o1);
        q.insert(o2);
        q.insert(o3);
        q.insert(o4);
        q.insert(o5);

        println!("{:#?}\n{:#?}\n{:#?}", q.odrs, q.pcs, q.pcl);
    }
}
