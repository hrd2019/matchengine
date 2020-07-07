use std::borrow::BorrowMut;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::time::SystemTime;

enum Side {
    Bid,
    Ask,
}

struct Odr {
    pc: f64,
    qty: f64,
    side: Side,
    ts: SystemTime,
    arcy: i64,
}

type qty = f64;

struct Query {
    pcs: HashMap<String, qty>,
    odrs: HashMap<String, Box<Vec<Odr>>>,
    pcl: Vec<u64>,
    side: Side,
    pc: f64,
}

impl Query {
    fn new(s: Side) -> Query {
        Query {
            pcs: HashMap::new(),
            odrs: HashMap::new(),
            pcl: vec![],
            side: s,
            pc: 0.0,
        }
    }

    fn insert(&mut self, odr: Odr) {
        let item = self.pcs.entry(odr.pc).or_insert(0.0);
        *item += odr.qty;

        let v = (odr.pc * odr.arcy as f64) as u64;

        let item = self
            .odrs
            .entry(odr.pc as String)
            .or_insert(Box::new(vec![]));
        item.push(odr);

        self.pcl.push(v);

        for i in self.pcl.iter() {
            match self.side {
                Side::Bid => {
                    self.sort();
                }
                Side::Ask => {
                    self.sort();
                }
                _ => {}
            }
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
mod tests {
    #[test]
    fn test1() {
        println!("test");
    }
}
