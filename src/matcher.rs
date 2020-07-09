use matchengine::Asset;

pub mod matcher {
    use matchengine::{get_accuracy, Asset, Odr, Queue, Side};
    use std::borrow::Borrow;
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
                _ => (),
            }

            // &self.sx.send().unwrap();
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
            let minV = ks[ks.len() - 1];
            if v < minV {
                return;
            }

            if v >= minV {
                let qty = pcs.get(&minV);
                let q = match qty {
                    Some(t) => *t,
                    None => 0.0,
                };

                let o = odr.qty - q;
                let mut a = 0;
                if o > 0.0 {
                    a = 1;
                } else if o == 0.0 {
                    a = 0;
                } else {
                    a = -1;
                }

                match a {
                    1 => odr.qty = o,
                    0 => {
                        pcs.remove(&minV);
                    }
                    -1 => {
                        pcs.insert(minV, q - odr.qty);
                    }
                    _ => {}
                }
            }

            ();
        }

        fn match_ask(&mut self, odr: &mut Odr, ac: i64) {
            let v = (odr.pc * ac as f64) as i64;
            ()
        }
    }
}
