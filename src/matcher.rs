pub mod matcher {
    use matchengine::{get_accuracy, Asset, Odr, Queue, Side};
    use std::collections::BTreeMap;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};

    pub trait Process {
        fn match_do(&mut self, odr: Odr);
        fn settle_do(&self);
        fn match_bid(&mut self, odr: &mut Odr, ac: i64);
        fn match_ask(&mut self, odr: &mut Odr, ac: i64);
    }

    struct Matcher {
        curcy_asset: Asset,
        value_asset: Asset,
        queue_bid: Queue,
        queue_ask: Queue,
        sx: Sender<Odr>,
        rx: Receiver<Odr>,
    }

    impl Matcher {
        pub fn new(c: Asset, v: Asset) -> Matcher {
            let queue_bid = Queue::new(Side::Bid);
            let queue_ask = Queue::new(Side::Ask);
            let (sx, rx) = mpsc::channel();
            Matcher {
                curcy_asset: c,
                value_asset: v,
                queue_bid,
                queue_ask,
                sx: sx,
                rx: rx,
            }
        }
    }

    impl Process for Matcher {
        fn match_do(&mut self, mut odr: Odr) {
            println!("{:#?}", self.queue_bid);

            let ac = get_accuracy(&odr.asset);

            match odr.side {
                Side::Bid => self.match_bid(&mut odr, ac),
                Side::Ask => self.match_ask(&mut odr, ac),
            }
        }

        fn settle_do(&self) {
            for rec in &self.rx {
                println!("settle: {:#?}", rec);
            }
        }

        fn match_bid(&mut self, odr: &mut Odr, ac: i64) {
            let pcs = &mut self.queue_ask.pcs;
            let ks: Vec<i64> = pcs.keys().cloned().collect();

            let v = (odr.pc * ac as f64) as i64;

            for i in ks.iter().rev() {
                if v >= *i {
                    let mut tmp = *odr;
                    tmp.qty = match_update(odr, pcs, *i);
                    tmp.pc = (*i / ac) as f64;
                    &self.sx.send(tmp).unwrap();
                }
            }

            if odr.qty > 0.0 {
                let item = self.queue_bid.pcs.entry(v).or_insert(0.0);
                *item += odr.qty;
            }
        }

        fn match_ask(&mut self, odr: &mut Odr, ac: i64) {
            let pcs = &mut self.queue_bid.pcs;
            let ks: Vec<i64> = pcs.keys().cloned().collect();

            let vk = (odr.pc * ac as f64) as i64;

            for i in ks.iter() {
                if vk <= *i {
                    let mut tmp = *odr;
                    tmp.qty = match_update(odr, pcs, *i);
                    tmp.pc = (*i / ac) as f64;
                    &self.sx.send(tmp).unwrap();
                }
            }

            if odr.qty > 0.0 {
                let item = self.queue_bid.pcs.entry(vk).or_insert(0.0);
                *item += odr.qty;
            }
        }
    }

    fn match_update(odr: &mut Odr, pcs: &mut BTreeMap<i64, f64>, first_v: i64) -> f64 {
        let qty = pcs.get(&first_v);
        let q = match qty {
            Some(t) => *t,
            None => 0.0,
        };

        // let stat;
        let left = odr.qty - q;
        let mut vol = 0.0;

        // if left > 0.0 {
        //     stat = 1;
        //     vol = q;
        // } else if left == 0.0 {
        //     stat = 0;
        //     vol = q;
        // } else {
        //     stat = -1;
        //     vol = odr.qty;
        // }

        match left {
            l if l > 0.0 => {
                vol = q;
                odr.qty = left;
                pcs.remove(&first_v);
            }
            l if l == 0.0 => {
                vol = q;
                odr.qty = 0.0;
                pcs.remove(&first_v);
            }
            l if l < 0.0 => {
                vol = odr.qty;
                odr.qty = 0.0;
                pcs.insert(first_v, q - odr.qty);
            }
            _ => {}
        }

        vol
    }
}
