use std::collections::{BTreeMap, HashMap};
use std::time::SystemTime;

#[derive(Debug, Copy, Clone)]
pub enum Asset {
    BTC(i64),
    EOS(i64),
    ETC(i64),
    USDT(i64),
}

#[derive(Debug, Copy, Clone)]
pub enum OptType {
    DEAL,
    CANCEL,
}

#[derive(Debug, Copy, Clone)]
pub enum TradeType {
    Limited,
    Market,
}

#[derive(Debug, Copy, Clone)]
pub struct MatchPair {
    pub bid_id: u64,
    pub ask_id: u64,
    pub pc: f64,
    pub qty: f64,
    pub ts: SystemTime,
}

pub const ASSET_BTC: Asset = Asset::BTC(1000);
pub const ASSET_EOS: Asset = Asset::EOS(1000);
pub const ASSET_USDT: Asset = Asset::USDT(1000);

#[derive(Debug, Copy, Clone)]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Copy, Clone)]
pub struct Odr {
    pub id: u64,
    pub asset: Asset,
    pub trade: TradeType,
    pub pc: f64,
    pub qty: f64,
    pub side: Side,
    pub opt: OptType,
    pub ts: SystemTime,
}

impl Odr {
    pub fn new(
        id: u64,
        asset: Asset,
        trade: TradeType,
        opt: OptType,
        pc: f64,
        qty: f64,
        side: Side,
    ) -> Odr {
        Odr {
            id,
            asset,
            trade,
            opt,
            pc,
            qty,
            side,
            ts: SystemTime::now(),
        }
    }
}

pub type QTY = f64;
pub type Index<'a> = (&'a mut BTreeMap<i64, QTY>, &'a mut HashMap<i64, Vec<Odr>>);

#[derive(Debug)]
pub struct Queue {
    pub pcs: BTreeMap<i64, QTY>,
    pub odrs: HashMap<i64, Vec<Odr>>,
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

        let v = self.odrs.entry(k).or_insert(vec![]);
        v.push(odr)

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

        let o1 = Odr::new(
            1,
            ASSET_BTC,
            TradeType::Market,
            OptType::DEAL,
            1.2,
            0.45,
            Side::Bid,
        );
        let o2 = Odr::new(
            2,
            ASSET_BTC,
            TradeType::Market,
            OptType::DEAL,
            1.3,
            0.45,
            Side::Bid,
        );
        let o3 = Odr::new(
            3,
            ASSET_BTC,
            TradeType::Market,
            OptType::DEAL,
            1.1,
            0.45,
            Side::Bid,
        );
        let o4 = Odr::new(
            4,
            ASSET_BTC,
            TradeType::Market,
            OptType::DEAL,
            1.1,
            1.45,
            Side::Bid,
        );
        let o5 = Odr::new(
            5,
            ASSET_BTC,
            TradeType::Market,
            OptType::DEAL,
            1.05,
            1.45,
            Side::Bid,
        );

        q.insert(o1);
        q.insert(o2);
        q.insert(o3);
        q.insert(o4);
        q.insert(o5);

        println!("{:#?}\n{:#?}", q.odrs, q.pcs);
    }
}

pub fn get_accuracy(asset: &Asset) -> i64 {
    let ac: i64 = match *asset {
        Asset::BTC(x) => x,
        Asset::EOS(x) => x,
        _ => 100,
    };

    ac
}
