pub mod matcher {
    use matchengine::{Queue, Side};

    pub trait Process {
        fn run(&mut self);
    }

    struct Matcher {
        name: String,
        queue: Queue,
    }

    impl Matcher {
        pub fn new(name: String) -> Matcher {
            let mut queue = Queue::new(Side::Bid);
            Matcher { name, queue }
        }
    }

    impl Process for Matcher {
        fn run(&mut self) {
            println!("{:#?}", self.queue)
        }
    }
}
