pub mod matcher {
    use matchengine::{Asset, Odr, Queue, Side};
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};

    pub trait Process {
        fn match_do(&mut self);
        fn settle_do(&self);
    }

    struct Matcher {
        curcy_asset: Asset,
        value_asset: Asset,
        queue: Queue,
        sx: Sender<Odr>,
        rx: Receiver<Odr>,
    }

    impl Matcher {
        pub fn new(c: Asset, v: Asset) -> Matcher {
            let queue = Queue::new(Side::Bid);
            let (sx, rx) = mpsc::channel();
            Matcher {
                curcy_asset: c,
                value_asset: v,
                queue,
                sx: sx,
                rx: rx,
            }
        }
    }

    impl Process for Matcher {
        fn match_do(&mut self) {
            println!("{:#?}", self.queue);
        }

        fn settle_do(&self) {
            for rec in self.rx {
                println!("settle: {:#?}", rec);
            }
        }
    }
}
