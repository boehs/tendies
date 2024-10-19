use chrono::{DateTime, Utc};
use rand::Rng;
use rgb::RGB8;
use serde::Deserialize;
use serde_qs;

use crate::position::Position;

#[derive(Deserialize, Default)]
struct Options {
    #[serde(default = "l_default")]
    pub l: f64,
}

fn l_default() -> f64 {
    1.0
}

fn parse_ticker(ticker: &str) -> (String, Options) {
    // get the ticker and options, split by ?. If there are no options, use the default
    let mut parts = ticker.splitn(2, '?');
    let ticker = parts.next().unwrap();
    let options = parts
        .next()
        .map(|options| serde_qs::from_str(options).unwrap())
        .unwrap_or(serde_qs::from_str("").unwrap());
    (ticker.to_string(), options)
}

pub struct Quote {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adjclose: f64,
}

#[derive(Clone)]
pub struct Portfolio {
    pub name: String,
    pub positions: Vec<Position>,
    pub color: RGB8,
    pub trading_fee: f64,
    pub balance: f64,
}

impl Portfolio {
    pub fn new(name: &str, balance: f64) -> Self {
        let mut rng = rand::thread_rng();

        Portfolio {
            name: name.to_string(),
            positions: Vec::new(),
            color: RGB8::new(rng.gen(), rng.gen(), rng.gen()),
            trading_fee: 0.0,
            balance,
        }
    }

    pub fn add_position_by_quantity(
        &mut self,
        ticker: &str,
        quantity: f64,
        share_price: f64,
        purchase_date: DateTime<Utc>,
    ) {
        let (ticker, options) = parse_ticker(ticker);
        let total_cost = quantity * share_price * options.l + self.trading_fee;

        if self.balance < total_cost {
            eprintln!("Not enough balance to add position");
            return;
        }

        self.positions.push(Position {
            ticker,
            quantity,
            share_price,
            purchase_date,
            leverage: options.l,
        });
        self.balance -= total_cost;
    }

    pub fn add_position_by_value(
        &mut self,
        ticker: &str,
        value: f64,
        share_price: f64,
        purchase_date: DateTime<Utc>,
    ) {
        let quantity = value / share_price;
        self.add_position_by_quantity(ticker, quantity, share_price, purchase_date);
    }

    pub fn sell_position_by_quantity(&mut self, ticker: &str, quantity: f64, share_price: f64) {
        let (ticker, options) = parse_ticker(ticker);
        let mut remaining_quantity = quantity;
        for position in self.positions.iter_mut().filter(|p| p.ticker == ticker) {
            if remaining_quantity <= 0.0 {
                break;
            }
            if position.quantity >= remaining_quantity {
                position.quantity -= remaining_quantity;
                self.balance += remaining_quantity * share_price * options.l;
                self.balance -= self.trading_fee;
                remaining_quantity = 0.0;
            } else {
                self.balance += position.quantity * share_price * options.l;
                self.balance -= self.trading_fee;
                remaining_quantity -= position.quantity;
                position.quantity = 0.0;
            }
        }

        if remaining_quantity > 0.0 {
            eprintln!("Not enough shares to sell");
        }

        // Remove positions with zero quantity
        self.positions.retain(|p| p.quantity > 0.0);
    }

    pub fn sell_position_by_value(&mut self, ticker: &str, value: f64, share_price: f64) {
        let quantity = value / share_price;
        self.sell_position_by_quantity(ticker, quantity, share_price);
    }

    pub fn calculate_value(&self, current_prices: &[(String, f64)]) -> f64 {
        let mut value: f64 = 0.0;
        for position in &self.positions {
            if let Some(price) = current_prices
                .iter()
                .find(|(ticker, _)| *ticker == position.ticker)
                .map(|(_, price)| *price)
            {
                value += position.quantity * price * position.leverage;
            }
        }
        value + self.balance
    }

    /// Takes a list of (ticker,[price history]) and returns the price history for the whole portfolio
    pub fn calculate_portfolio_history(&self, price_histories: &[(String, Vec<f64>)]) -> Vec<f64> {
        let mut portfolio_history = vec![0.0; price_histories[0].1.len()];
        for position in &self.positions {
            if let Some(price_history) = price_histories
                .iter()
                .find(|(ticker, _)| *ticker == position.ticker)
                .map(|(_, history)| history)
            {
                for (i, price) in price_history.iter().enumerate() {
                    portfolio_history[i] += position.quantity * price * position.leverage;
                }
            }
        }
        portfolio_history
    }

    pub fn get_positions_for_ticker(&self, ticker: &str) -> Vec<&Position> {
        self.positions
            .iter()
            .filter(|p| p.ticker == ticker)
            .collect()
    }
}
