use matchengine::Asset;

pub mod matcher {
    use matchengine::{get_accuracy, Asset, Odr, Queue, Side};
    use std::borrow::Borrow;
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};

    pub trait Process {
        fn match_do(&mut self, odr: Odr);
        fn settle_do(&self);
        fn match_bid(&mut self, odr: Odr, ac: i64);
        fn match_ask(&mut self, odr: Odr, ac: i64);
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
        fn match_do(&mut self, odr: Odr) {
            println!("{:#?}", self.queue);

            let ac = get_accuracy(&odr.asset);

            match odr.side {
                Side::Bid => self.match_ask(odr, ac),
                Side::Ask => self.match_bid(odr, ac),
                _ => (),
            }

            // &self.sx.send().unwrap();
        }

        fn settle_do(&self) {
            for rec in &self.rx {
                println!("settle: {:#?}", rec);
            }
        }

        fn match_bid(&mut self, odr: Odr, ac: i64) {
            let pcs = &self.queue.pcs;
            let ks = pcs.keys().cloned().collect();

            let v = (odr.pc * ac as f64) as i64;
            if v > ks[0] {}

            ();
        }

        fn match_ask(&mut self, odr: Odr, ac: i64) {
            let v = (odr.pc * ac as f64) as i64;
            ()
        }
    }
}
