use std::collections::{BTreeMap, VecDeque};

use crate::models::{Order, Trade, Side, OrderType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MatchingMode {
    Fifo,
    ProRata,
}

pub struct OrderBook {
    pub buy_book: BTreeMap<u64, VecDeque<Order>>,  // descending order
    pub sell_book: BTreeMap<u64, VecDeque<Order>>, // ascending order
    pub mode: MatchingMode,
}

impl OrderBook {
    pub fn new(mode: MatchingMode) -> Self {
        Self {
            buy_book: BTreeMap::new(),
            sell_book: BTreeMap::new(),
            mode,
        }
    }

    pub fn submit_order(&mut self, order: Order) -> Vec<Trade> {
        match order.side {
            Side::Buy => self.match_buy(order),
            Side::Sell => self.match_sell(order),
        }
    }

    fn match_buy(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = vec![];
        let prices: Vec<u64> = self.sell_book.keys().cloned().collect();

        for price in prices {
            if let Some(limit_price) = order.price {
                if price > limit_price {
                    break;
                }
            }

            let mut queue = match self.sell_book.remove(&price) {
                Some(q) => q,
                None => continue,
            };

            if self.mode == MatchingMode::ProRata {
                let total_available: u64 = queue.iter().map(|o| o.quantity).sum();
                if total_available == 0 {
                    continue;
                }
            
                let mut fills = vec![];
                let mut total_assigned = 0;
            
                // First pass: floor allocation
                for sell_order in &queue {
                    let share = ((sell_order.quantity as f64 / total_available as f64) * order.quantity as f64).floor() as u64;
                    fills.push(share);
                    total_assigned += share;
                }
            
                // Distribute leftover starting with largest sellers
                let mut remaining = order.quantity - total_assigned;
                let mut sorted_indices: Vec<_> = queue.iter().enumerate().collect();
                sorted_indices.sort_by(|a, b| b.1.quantity.cmp(&a.1.quantity));
            
                for (i, _) in sorted_indices {
                    if remaining == 0 { break; }
                    fills[i] += 1;
                    remaining -= 1;
                }
            
                // Execute trades
                let mut new_queue = VecDeque::new();
                for (i, mut sell_order) in queue.into_iter().enumerate() {
                    let trade_qty = fills[i].min(order.quantity).min(sell_order.quantity);
            
                    if trade_qty > 0 {
                        trades.push(Trade {
                            price,
                            quantity: trade_qty,
                            buyer: order.user_id.clone(),
                            seller: sell_order.user_id.clone(),
                            timestamp: chrono::Utc::now(),
                        });
            
                        order.quantity -= trade_qty;
                        sell_order.quantity -= trade_qty;
                    }
            
                    if sell_order.quantity > 0 {
                        new_queue.push_back(sell_order);
                    }
            
                    if order.quantity == 0 {
                        break;
                    }
                }
            
                if !new_queue.is_empty() {
                    self.sell_book.insert(price, new_queue);
                }
            } else {
                while let Some(mut sell_order) = queue.pop_front() {
                    let trade_qty = order.quantity.min(sell_order.quantity);
                    trades.push(Trade {
                        price,
                        quantity: trade_qty,
                        buyer: order.user_id.clone(),
                        seller: sell_order.user_id.clone(),
                        timestamp: chrono::Utc::now(),
                    });

                    order.quantity -= trade_qty;
                    sell_order.quantity -= trade_qty;

                    if sell_order.quantity > 0 {
                        queue.push_front(sell_order);
                        break;
                    }
                    if order.quantity == 0 {
                        break;
                    }
                }

                if !queue.is_empty() {
                    self.sell_book.insert(price, queue);
                }
            }

            if order.quantity == 0 {
                break;
            }
        }

        if order.quantity > 0 && matches!(order.order_type, OrderType::Limit) {
            let price = order.price.unwrap();
            self.buy_book.entry(price)
                .or_insert_with(VecDeque::new)
                .push_back(order);
        }

        trades
    }

    fn match_sell(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = vec![];
        let prices: Vec<u64> = self.buy_book.keys().rev().cloned().collect();

        for price in prices {
            if let Some(limit_price) = order.price {
                if price < limit_price {
                    break;
                }
            }

            let mut queue = match self.buy_book.remove(&price) {
                Some(q) => q,
                None => continue,
            };

            if self.mode == MatchingMode::ProRata {
                let total_available: u64 = queue.iter().map(|o| o.quantity).sum();
                if total_available == 0 {
                    continue;
                }
            
                let mut fills = vec![];
                let mut total_assigned = 0;
            
                // First pass: floor allocation
                for buy_order in &queue {
                    let share = ((buy_order.quantity as f64 / total_available as f64) * order.quantity as f64).floor() as u64;
                    fills.push(share);
                    total_assigned += share;
                }
            
                // Distribute leftover starting with largest buyers
                let mut remaining = order.quantity - total_assigned;
                let mut sorted_indices: Vec<_> = queue.iter().enumerate().collect();
                sorted_indices.sort_by(|a, b| b.1.quantity.cmp(&a.1.quantity));
            
                for (i, _) in sorted_indices {
                    if remaining == 0 { break; }
                    fills[i] += 1;
                    remaining -= 1;
                }
            
                // Execute trades
                let mut new_queue = VecDeque::new();
                for (i, mut buy_order) in queue.into_iter().enumerate() {
                    let trade_qty = fills[i].min(order.quantity).min(buy_order.quantity);
            
                    if trade_qty > 0 {
                        trades.push(Trade {
                            price,
                            quantity: trade_qty,
                            buyer: buy_order.user_id.clone(),
                            seller: order.user_id.clone(),
                            timestamp: chrono::Utc::now(),
                        });
            
                        order.quantity -= trade_qty;
                        buy_order.quantity -= trade_qty;
                    }
            
                    if buy_order.quantity > 0 {
                        new_queue.push_back(buy_order);
                    }
            
                    if order.quantity == 0 {
                        break;
                    }
                }
            
                if !new_queue.is_empty() {
                    self.buy_book.insert(price, new_queue);
                }
            } else {
                while let Some(mut buy_order) = queue.pop_front() {
                    let trade_qty = order.quantity.min(buy_order.quantity);
                    trades.push(Trade {
                        price,
                        quantity: trade_qty,
                        buyer: buy_order.user_id.clone(),
                        seller: order.user_id.clone(),
                        timestamp: chrono::Utc::now(),
                    });

                    order.quantity -= trade_qty;
                    buy_order.quantity -= trade_qty;

                    if buy_order.quantity > 0 {
                        queue.push_front(buy_order);
                        break;
                    }

                    if order.quantity == 0 {
                        break;
                    }
                }

                if !queue.is_empty() {
                    self.buy_book.insert(price, queue);
                }
            }

            if order.quantity == 0 {
                break;
            }
        }

        if order.quantity > 0 && matches!(order.order_type, OrderType::Limit) {
            let price = order.price.unwrap();
            self.sell_book.entry(price)
                .or_insert_with(VecDeque::new)
                .push_back(order);
        }

        trades
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Order, Side, OrderType};

    fn make_order(user: &str, price: u64, qty: u64, side: Side) -> Order {
        Order::new(
            user.to_string(),
            "AAPL".to_string(),
            side,
            OrderType::Limit,
            Some(price),
            qty,
        )
    }

    fn make_market_order(user: &str, qty: u64, side: Side) -> Order {
        Order::new(
            user.to_string(),
            "AAPL".to_string(),
            side,
            OrderType::Market,
            None,
            qty,
        )
    }

    #[test]
    fn test_fifo_match_single_fill() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("seller1", 100, 10, Side::Sell));
        let trades = book.submit_order(make_order("buyer1", 100, 10, Side::Buy));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[0].quantity, 10);
        assert_eq!(trades[0].buyer, "buyer1");
        assert_eq!(trades[0].seller, "seller1");
    }

    #[test]
    fn test_fifo_partial_fill_and_resting_order() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("seller1", 100, 5, Side::Sell));
        let trades = book.submit_order(make_order("buyer1", 100, 10, Side::Buy));

        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 5);
        assert!(book.buy_book.get(&100).is_some());
    }

    #[test]
    fn test_pro_rata_split_across_sellers() {
        let mut book = OrderBook::new(MatchingMode::ProRata);
        book.submit_order(make_order("s1", 100, 10, Side::Sell));
        book.submit_order(make_order("s2", 100, 20, Side::Sell));
        book.submit_order(make_order("s3", 100, 30, Side::Sell));

        let trades = book.submit_order(make_order("b1", 100, 30, Side::Buy));
        assert_eq!(trades.len(), 3);
        let total_qty: u64 = trades.iter().map(|t| t.quantity).sum();
        assert_eq!(total_qty, 30);
    }

    #[test]
    fn test_market_order_executes_best_price() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("s1", 99, 5, Side::Sell));
        book.submit_order(make_order("s2", 98, 5, Side::Sell));

        let trades = book.submit_order(make_market_order("b1", 10, Side::Buy));
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 98);
        assert_eq!(trades[1].price, 99);
    }

    #[test]
    fn test_fifo_resting_limit_order() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        let order = make_order("buyer1", 99, 10, Side::Buy);
        book.submit_order(order.clone());
        assert_eq!(book.buy_book.get(&99).unwrap().front().unwrap().user_id, "buyer1");
    }

    #[test]
    fn test_market_order_partial_fill() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("s1", 100, 3, Side::Sell));
        let trades = book.submit_order(make_market_order("b1", 5, Side::Buy));
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 3);
        assert!(book.sell_book.is_empty()); // market order doesn’t rest
    }

    #[test]
    fn test_limit_order_cross_multiple_price_levels() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("s1", 100, 3, Side::Sell));
        book.submit_order(make_order("s2", 101, 2, Side::Sell));
        let trades = book.submit_order(make_order("b1", 101, 5, Side::Buy));
        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].price, 100);
        assert_eq!(trades[1].price, 101);
    }

    #[test]
    fn test_book_cleanup_after_full_match() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("s1", 100, 5, Side::Sell));
        book.submit_order(make_order("b1", 100, 5, Side::Buy));
        assert!(book.sell_book.get(&100).is_none());
        assert!(book.buy_book.get(&100).is_none());
    }

    #[test]
    fn test_no_trade_when_price_doesnt_cross() {
        let mut book = OrderBook::new(MatchingMode::Fifo);
        book.submit_order(make_order("s1", 105, 5, Side::Sell));
        let trades = book.submit_order(make_order("b1", 100, 5, Side::Buy));
        assert_eq!(trades.len(), 0);
        assert!(book.buy_book.get(&100).is_some());
        assert!(book.sell_book.get(&105).is_some());
    }

    #[test]
    fn test_pro_rata_multiple_levels_only_best_matched() {
        let mut book = OrderBook::new(MatchingMode::ProRata);
        book.submit_order(make_order("s1", 100, 10, Side::Sell));
        book.submit_order(make_order("s2", 101, 10, Side::Sell));
        let trades = book.submit_order(make_order("b1", 101, 5, Side::Buy));
        // Only best price level (100) should match
        assert!(trades.iter().all(|t| t.price == 100));
    }

}
