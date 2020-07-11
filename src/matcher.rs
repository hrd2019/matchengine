pub mod matcher {
    use matchengine::{get_accuracy, Asset, MatchPair, Odr, OptType, Queue, Side};
    use std::collections::{BTreeMap, HashMap};
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::SystemTime;

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
        fn match_do(&mut self, mut odr: Odr) {
            println!("{:#?}", self.queue_bid);

            let ac = get_accuracy(&odr.asset);

            match odr.opt {
                OptType::DEAL => match odr.side {
                    Side::Bid => self.match_bid(&mut odr, ac),
                    Side::Ask => self.match_ask(&mut odr, ac),
                },
                OptType::CANCEL => {
                    let vk = (odr.pc * ac as f64) as i64;
                    let odrs = &mut self.queue_ask.odrs;

                    match odrs.get_mut(&vk) {
                        Some(o) => {
                            for (index, item) in o.iter().enumerate() {
                                if item.id == odr.id {
                                    o.remove(index);

                                    match odr.side {
                                        Side::Ask => {
                                            let mut pcs = self.queue_ask.pcs.entry(vk).or_default();
                                            *pcs -= odr.qty;
                                        }
                                        Side::Bid => {
                                            let mut pcs = self.queue_bid.pcs.entry(vk).or_default();
                                            *pcs -= odr.qty;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        fn settle_do(&self) {
            for rec in &self.rx {
                println!("settle: {:#?}", rec);
            }
        }

        fn match_bid(&mut self, odr: &mut Odr, ac: i64) {
            let pcs = &mut self.queue_ask.pcs;
            let odrs = &mut self.queue_ask.odrs;
            let ks: Vec<i64> = pcs.keys().cloned().collect();

            let vk = (odr.pc * ac as f64) as i64;

            let mut is_ok = true;

            for i in ks.iter().rev() {
                if vk >= *i && odr.qty > 0.0 {
                    let res = match_update(odr, odrs, pcs, *i);
                    match res {
                        Ok((x, y)) => {
                            let pair = MatchPair {
                                bid_id: odr.id,
                                ask_id: x,
                                pc: (*i / ac) as f64,
                                qty: y,
                                ts: SystemTime::now(),
                            };

                            &self.sx.send(pair).unwrap();
                        }
                        Err(e) => {
                            println!("err {}", e);
                            is_ok = false;
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            if is_ok && odr.qty > 0.0 {
                self.queue_bid.insert(*odr)
            }
        }

        fn match_ask(&mut self, odr: &mut Odr, ac: i64) {
            let pcs = &mut self.queue_bid.pcs;
            let odrs = &mut self.queue_bid.odrs;
            let ks: Vec<i64> = pcs.keys().cloned().collect();

            let vk = (odr.pc * ac as f64) as i64;

            let mut is_ok = true;

            for i in ks.iter() {
                if vk <= *i && odr.qty > 0.0 {
                    let res = match_update(odr, odrs, pcs, *i);
                    match res {
                        Ok((x, y)) => {
                            let pair = MatchPair {
                                bid_id: x,
                                ask_id: odr.id,
                                pc: (*i / ac) as f64,
                                qty: y,
                                ts: SystemTime::now(),
                            };

                            &self.sx.send(pair).unwrap();
                        }
                        Err(e) => {
                            println!("err {}", e);
                            is_ok = false;
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            if is_ok && odr.qty > 0.0 {
                self.queue_ask.insert(*odr)
            }
        }
    }

    type Cols = HashMap<i64, Vec<Odr>>;
    type Deep = BTreeMap<i64, f64>;

    fn match_update(
        odr: &mut Odr,
        odrs: &mut Cols,
        pcs: &mut Deep,
        first_v: i64,
    ) -> Result<(u64, f64), String> {
        let qty = pcs.get(&first_v);
        let q = match qty {
            Some(t) => *t,
            None => 0.0,
        };

        if q == 0.0 {
            let s = format!("invalid data {}", q);
            return Err(s);
        }

        let left = odr.qty - q;
        let mut vol = 0.0;

        match left {
            l if l > 0.0 => {
                vol = q;
                odr.qty = l;
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

        let mut order_id = 0;
        let col = odrs.get_mut(&first_v);
        if let Some(list) = col {
            if odr.qty == 0.0 {
                if let Some(v) = list.pop() {
                    order_id = v.id;
                }
            } else {
                if let Some(v) = list.get_mut(0) {
                    order_id = v.id;
                    if v.qty != 0.0 {
                        v.qty = odr.qty;
                    }
                }
            }
        };

        Ok((order_id, vol))
    }
}
