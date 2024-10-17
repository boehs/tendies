use chrono::{DateTime, Utc};

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
}

#[derive(Clone)]
pub struct Position {
    pub ticker: String,
    pub quantity: f64,
    pub share_price: f64,
    pub purchase_date: DateTime<Utc>,
    pub leverage: f64,
}

impl Portfolio {
    pub fn new(name: &str) -> Self {
        Portfolio {
            name: name.to_string(),
            positions: Vec::new(),
        }
    }

    pub fn add_position_by_quantity(
        &mut self,
        ticker: &str,
        quantity: f64,
        share_price: f64,
        purchase_date: DateTime<Utc>,
    ) {
        self.positions.push(Position {
            ticker: ticker.to_string(),
            quantity,
            share_price,
            purchase_date,
            leverage: 1.0,
        });
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
        value
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
}
