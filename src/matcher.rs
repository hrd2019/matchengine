pub mod matcher {
    use matchengine::{Asset, Queue, Side};

    pub trait Process {
        fn matchdo(&mut self);
        fn settledo(&mut self);
    }

    struct Matcher {
        curcy_asset: Asset,
        value_asset: Asset,
        queue: Queue,
    }

    impl Matcher {
        pub fn new(c: Asset, v: Asset) -> Matcher {
            let mut queue = Queue::new(Side::Bid);
            Matcher {
                curcy_asset: c,
                value_asset: v,
                queue,
            }
        }
    }

    impl Process for Matcher {
        fn matchdo(&mut self) {
            println!("{:#?}", self.queue);
        }

        fn settledo(&mut self) {
            println!("{:#?}", self.queue);
        }
    }
}
