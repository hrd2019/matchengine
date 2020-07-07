use std::time::SystemTime;
use std::collections::HashMap;
use std::cmp::Ordering;

enum Side{
    Bid,
    Ask,
}

struct Odr{
    pc:f64,
    qty:f64,
    side:Side,
    ts:SystemTime,
    arcy:i64,
}

type qty = f64;

struct Query {
    pcs: HashMap<f64, qty>,
    odrs: HashMap<f64, Vec<Odr>>,
    pcl:Vec<u64>,
    side: Side,
    pc: f64,
}

impl Query{
    fn new(s:Side) -> Query{
        Query{
            pcs: HashMap::new(),
            odrs:HashMap::new(),
            pcl:vec![],
            side:s,
            pc: 0.0
        }
    }

    fn insert(&mut self, odr : Odr) {
        let item = self.pcs.entry(odr.pc).or_insert(0.0);
        *item += odr.qty;

        let v = (odr.pc * odr.arcy as f64) as u64;

        let item = self.odrs.entry(odr.pc).or_insert(vec![]);
        *item.push(odr);

        self.pcl.push(v);

        for i in self.pcl.iter(){
            match self.side {
                Some(Side::Bid) => {
                    self.sort();
                },
                Some(Side::Ask) => {
                    self.sort();
                },
                _ => {}
            }
        }
    }

    fn sort(&mut self) {
        let x = self.pcl.as_mut();
        for i in 1..x.len() {
            let tmp = a[i];
            let mut j = i;
            while j > 0 && tmp < a[j - 1] {
                ;
                a[j] = a[j - 1];
                j -= 1;
            }
            a[j] = tmp;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {}
}