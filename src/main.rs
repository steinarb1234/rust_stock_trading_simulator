use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};


const INITIAL_CASH: f32 = 10000.0;

#[derive(Debug)]
struct MarketData {
    symbol: String,
    price: f32,
}

#[derive(Debug)]
enum OrderType {
    Market,
    Limit(f32),
}

#[derive(Debug)]
struct Order {
    symbol: String,
    quantity: i32,
    order_type: OrderType,
}

struct Portfolio {
    cash: f32,
    holdings: HashMap<String, i32>, // Symbol to quantity mapping
}

impl Portfolio {
    fn execute_order(&mut self, order: &Order, market_price: f32) {
        match order.order_type {
            OrderType::Market => {
                // Execute market order at the current market price
                self.process_order(order, market_price);
            },
            OrderType::Limit(limit_price) => {
                if (order.quantity > 0 && market_price <= limit_price) || (order.quantity < 0 && market_price >= limit_price) {
                    // Execute limit order if the market price is favorable
                    self.process_order(order, limit_price);
                }
                // Else, do not execute the order as the limit condition is not met
            }
        }
    }

    fn process_order(&mut self, order: &Order, execution_price: f32) {
        let total_order_value = execution_price * order.quantity.abs() as f32;

        if order.quantity > 0 {
            // Buying stocks
            if self.cash >= total_order_value {
                *self.holdings.entry(order.symbol.clone()).or_insert(0) += order.quantity;
                self.cash -= total_order_value;
            } else {
                println!("Not enough cash to execute buy order.");
            }
        } else if order.quantity < 0 {
            // Selling stocks
            let current_holding = self.holdings.entry(order.symbol.clone()).or_insert(0);

            if *current_holding >= -order.quantity {
                *current_holding += order.quantity; // Deducting as quantity is negative
                self.cash += total_order_value;
            } else {
                println!("Not enough shares to execute sell order.");
            }
        }
    }

    fn calculate_profit_loss(&self, current_market_data: &[MarketData]) -> f32 {
        let mut total_value = 0.0;

        // Calculate the total value of the portfolio based on current market prices
        for data in current_market_data {
            if let Some(&quantity) = self.holdings.get(&data.symbol) {
                total_value += data.price * quantity as f32;
            }
        }

        // Total portfolio value - initial cash gives profit or loss
        total_value + self.cash - INITIAL_CASH
    }
}

fn load_market_data(file_path: &str) -> io::Result<Vec<MarketData>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            let symbol = parts[0].to_string();
            let price = parts[1].parse::<f32>().unwrap_or(0.0);
            data.push(MarketData { symbol, price });
        }
    }

    Ok(data)
}


fn find_market_price(market_data: &[MarketData], symbol: &str) -> Option<f32> {
    market_data.iter().find(|&data| data.symbol == symbol).map(|data| data.price)
}


fn main() {
    // Initialize portfolio with some initial cash
    let mut portfolio = Portfolio {
        cash: INITIAL_CASH,
        holdings: HashMap::new(),
    };

    // TODO::Load data from API in async
    let market_data = load_market_data("market_data.csv").expect("Failed to load market data");

    // TODO::implement user interface to make orders
    let orders = vec![
        Order {
            symbol: "AAPL".to_string(),
            quantity: 1,
            order_type: OrderType::Market,
        },
        Order {
            symbol: "MSFT".to_string(),
            quantity: 2,
            order_type: OrderType::Limit(280.0),
        },
        Order {
            symbol: "AAPL".to_string(),
            quantity: -1,
            order_type: OrderType::Limit(280.0),
        },
    ];

    for order in orders {
        println!("Processing order: {:?}", order);
        if let Some(market_price) = find_market_price(&market_data, &order.symbol) {
            portfolio.execute_order(&order, market_price);
        } else {
            println!("Market data not found for {}", order.symbol);
        }
        println!("Current Holdings: {:?}", portfolio.holdings);
        println!("Current Cash Balance: ${:.2}", portfolio.cash);
        let profit_loss = portfolio.calculate_profit_loss(&market_data);
        println!("Current profit or loss: ${:.2}\n", profit_loss);
    }
}
