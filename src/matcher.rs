pub mod matcher {
    use matchengine::{get_accuracy, Asset, MatchPair, Odr, OptType, Queue, Side, TradeType};
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

                    match odr.side {
                        Side::Ask => {
                            match_cancel(
                                &odr,
                                &mut self.queue_ask.odrs,
                                &mut self.queue_ask.pcs,
                                vk,
                            );
                        }
                        Side::Bid => {
                            match_cancel(
                                &odr,
                                &mut self.queue_bid.odrs,
                                &mut self.queue_bid.pcs,
                                vk,
                            );
                        }
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

            match odr.trade {
                TradeType::Limited => {
                    let i = ks.get(0).expect("no found");
                    if *i == vk && odr.qty > 0.0 {
                        is_ok = match_send(odr, odrs, pcs, *i, ac, &mut self.sx);
                    }
                }
                TradeType::Market => {
                    for i in ks.iter() {
                        if vk >= *i && odr.qty > 0.0 {
                            is_ok = match_send(odr, odrs, pcs, *i, ac, &mut self.sx);
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
                self.queue_bid.insert(*odr)
            }
        }

        fn match_ask(&mut self, odr: &mut Odr, ac: i64) {
            let pcs = &mut self.queue_bid.pcs;
            let odrs = &mut self.queue_bid.odrs;
            let ks: Vec<i64> = pcs.keys().cloned().collect();

            let vk = (odr.pc * ac as f64) as i64;

            let mut is_ok = true;

            match odr.trade {
                TradeType::Limited => {
                    let i = ks.get(0).cloned().expect("");
                    if i == vk && odr.qty > 0.0 {
                        is_ok = match_send(odr, odrs, pcs, i, ac, &mut self.sx);
                    }
                }
                TradeType::Market => {
                    for i in ks.iter() {
                        if vk <= *i && odr.qty > 0.0 {
                            is_ok = match_send(odr, odrs, pcs, *i, ac, &mut self.sx);
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
        vk: i64,
    ) -> Result<(u64, f64), String> {
        let mut qty = *pcs.get(&vk).expect("no match data");

        let list = odrs.get_mut(&vk).expect("no match list");
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

        qty -= vol;
        if qty != 0.0 {
            pcs.insert(vk, qty);
        } else {
            pcs.remove(&vk);
        }

        Ok((target.id, vol))
    }

    fn match_send(
        odr: &mut Odr,
        odrs: &mut Cols,
        pcs: &mut Deep,
        vk: i64,
        ac: i64,
        sx: &mut Sender<MatchPair>,
    ) -> bool {
        let mut is_ok = true;

        let res = match_update(odr, odrs, pcs, vk);
        match res {
            Ok((x, y)) => {
                let pair = MatchPair {
                    bid_id: x,
                    ask_id: odr.id,
                    pc: (vk / ac) as f64,
                    qty: y,
                    ts: SystemTime::now(),
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

    fn match_cancel(odr: &Odr, odrs: &mut Cols, pcs: &mut Deep, vk: i64) {
        let mut pcs = pcs.entry(vk).or_default();
        *pcs -= odr.qty;

        let odrs = odrs.get_mut(&vk);
        match odrs {
            Some(list) => {
                for (index, item) in list.iter().enumerate() {
                    if item.id == odr.id {
                        list.remove(index);
                        break;
                    }
                }
            }
            _ => {}
        }
    }
}
