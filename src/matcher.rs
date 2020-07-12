pub mod matcher {
    use matchengine::{
        get_accuracy, Asset, Index, MatchPair, Odr, OptType, Queue, Side, TradeType,
    };
    use std::collections::{BTreeMap, HashMap};
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::SystemTime;

    pub trait Process {
        fn process(&mut self, odr: Odr);
        fn settle_do(&self);
        fn match_deal(&mut self, odr: &mut Odr, ac: i64);
        fn match_cancel(&mut self, odr: &mut Odr, vk: i64);
    }

    struct Matcher {
        curcy_asset: Asset,
        value_asset: Asset,
        queue_bid: Queue,
        queue_ask: Queue,
        sx: Sender<MatchPair>,
        rx: Receiver<MatchPair>,
    }

    impl Matcher {
        pub fn new(curcy_asset: Asset, value_asset: Asset) -> Matcher {
            let queue_bid = Queue::new(Side::Bid);
            let queue_ask = Queue::new(Side::Ask);
            let (sx, rx) = mpsc::channel();
            Matcher {
                curcy_asset,
                value_asset,
                queue_bid,
                queue_ask,
                sx: sx,
                rx: rx,
            }
        }
    }

    impl Process for Matcher {
        fn process(&mut self, mut odr: Odr) {
            println!("{:#?}", self.queue_bid);

            let ac = get_accuracy(&odr.asset);

            match odr.opt {
                OptType::DEAL => {
                    self.match_deal(&mut odr, ac);
                }
                OptType::CANCEL => {
                    let vk = (odr.pc * ac as f64) as i64;
                    self.match_cancel(&mut odr, vk);
                }
            }
        }

        fn settle_do(&self) {
            for rec in &self.rx {
                println!("settle: {:#?}", rec);
            }
        }

        fn match_deal(&mut self, odr: &mut Odr, ac: i64) {
            let mut index = get_index(&mut self.queue_ask, &mut self.queue_bid, &odr);
            let ks: Vec<i64> = index.0.keys().cloned().collect();
            let vk = (odr.pc * ac as f64) as i64;
            let mut is_ok = true;

            match odr.trade {
                TradeType::Limited => {
                    let i = ks.get(0).cloned().expect("");
                    if i == vk && odr.qty > 0.0 {
                        is_ok = match_send(odr, &mut index, i, ac, &mut self.sx);
                    }
                }
                TradeType::Market => {
                    for i in ks.iter() {
                        if vk <= *i && odr.qty > 0.0 {
                            is_ok = match_send(odr, &mut index, *i, ac, &mut self.sx);
                        } else {
                            break;
                        }

                        if !is_ok {
                            break;
                        }
                    }
                }
            }

            if is_ok && odr.qty > 0.0 {
                update_odr(&mut self.queue_ask, &mut self.queue_bid, odr)
            }
        }

        fn match_cancel(&mut self, odr: &mut Odr, vk: i64) {
            let index = get_index(&mut self.queue_ask, &mut self.queue_bid, &odr);
            let odrs = index.1.get_mut(&vk);
            match odrs {
                Some(list) => {
                    for (i, item) in list.iter().enumerate() {
                        if item.id == odr.id {
                            list.remove(i);

                            let mut qty = index.0.entry(vk).or_default();
                            *qty -= odr.qty;

                            break;
                        }
                    }
                }
                _ => {}
            };
        }
    }

    fn match_update(odr: &mut Odr, index: &mut Index, vk: i64) -> Result<(u64, f64), String> {
        let list = index.1.get_mut(&vk).expect("no match list");
        let mut target = list.get(0).cloned().expect("no data");

        let left = odr.qty - target.qty;
        let mut vol = 0.0;

        match left {
            l if l > 0.0 => {
                vol = target.qty;
                odr.qty = l;
            }
            l if l == 0.0 => {
                vol = target.qty;
                odr.qty = 0.0;
            }
            l if l < 0.0 => {
                vol = odr.qty;
                odr.qty = 0.0;
            }
            _ => {}
        }

        target.qty -= vol;
        if target.qty == 0.0 {
            list.remove(0);
        } else {
            let mut o = list.get_mut(0).expect("no data");
            o.qty = target.qty;
        }

        let mut qty = *index.0.get(&vk).expect("no match data");
        qty -= vol;
        if qty != 0.0 {
            index.0.insert(vk, qty);
        } else {
            index.0.remove(&vk);
        }

        Ok((target.id, vol))
    }

    fn match_send(
        odr: &mut Odr,
        index: &mut Index,
        vk: i64,
        ac: i64,
        sx: &mut Sender<MatchPair>,
    ) -> bool {
        let mut is_ok = true;

        let res = match_update(odr, index, vk);
        match res {
            Ok((x, y)) => {
                let pair = match odr.side {
                    Side::Bid => MatchPair {
                        bid_id: odr.id,
                        ask_id: x,
                        pc: (vk / ac) as f64,
                        qty: y,
                        ts: SystemTime::now(),
                    },
                    Side::Ask => MatchPair {
                        bid_id: x,
                        ask_id: odr.id,
                        pc: (vk / ac) as f64,
                        qty: y,
                        ts: SystemTime::now(),
                    },
                };

                sx.send(pair).unwrap();
            }
            Err(e) => {
                println!("err {}", e);
                is_ok = false;
            }
        }

        is_ok
    }

    fn get_index<'a>(queue_ask: &'a mut Queue, queue_bid: &'a mut Queue, odr: &Odr) -> Index<'a> {
        let index: Index = match odr.side {
            Side::Bid => {
                let pcs = &mut queue_ask.pcs;
                let odrs = &mut queue_ask.odrs;

                (pcs, odrs)
            }
            Side::Ask => {
                let pcs = &mut queue_bid.pcs;
                let odrs = &mut queue_bid.odrs;

                (pcs, odrs)
            }
        };

        index
    }

    fn update_odr(queue_ask: &mut Queue, queue_bid: &mut Queue, odr: &Odr) {
        match odr.side {
            Side::Ask => {
                queue_ask.insert(*odr);
            }
            Side::Bid => {
                queue_bid.insert(*odr);
            }
        }
    }
}
