pub mod matcher {
    use matchengine::{Asset, Queue, Side};
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};

    pub trait Process {
        fn match_do(&mut self);
        fn settle_do(&mut self);
    }

    struct Matcher<T> {
        curcy_asset: Asset,
        value_asset: Asset,
        queue: Queue,
        sx: Sender<T>,
        rx: Receiver<T>,
    }

    impl<T> Matcher<T> {
        pub fn new(c: Asset, v: Asset) -> Matcher<T> {
            let mut queue = Queue::new(Side::Bid);
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

    impl<T> Process for Matcher<T> {
        fn match_do(&mut self) {
            println!("{:#?}", self.queue);
        }

        fn settle_do(&mut self) {
            for rec in self.rx {
                println!("settle: {:#?}", rec);
            }
        }
    }
}
