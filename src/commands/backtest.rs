use clap::Args;
use tendies::portfolio;
use textplots::{AxisBuilder, Chart, Plot};

#[derive(Args)]
pub struct BacktestArgs {
    #[arg(short, long)]
    tickers: Vec<String>,
    #[arg(long, default_value_t = 10000.0)]
    starting_balance: f64,
    #[arg(short, long, default_value = "1d")]
    interval: String,
    #[arg(short, long, default_value = "1y")]
    range: String,
}

pub async fn main(args: BacktestArgs) -> Result<(), Box<dyn std::error::Error>> {
    let provider = yahoo_finance_api::YahooConnector::new().unwrap();

    let mut all_quotes = vec![];
    let mut portfolios = vec![];
    let mut price_histories = vec![];

    let pb =
        indicatif::ProgressBar::new(args.tickers.len() as u64).with_prefix("Downloading quotes");

    for ticker in args.tickers {
        let response = provider
            .get_quote_range(&ticker, &args.interval, &args.range)
            .await?;
        let quotes = match response.quotes() {
            Ok(quotes) => quotes,
            Err(_) => {
                eprintln!("No quotes found for ticker {}", ticker);
                continue;
            }
        };
        pb.inc(1);
        let mut portfolio = portfolio::Portfolio::new(&ticker);
        portfolio.add_position_by_value(
            &ticker,
            args.starting_balance,
            quotes.first().unwrap().close,
            chrono::Utc::now(),
        );
        portfolios.push(portfolio.clone());
        price_histories.push((
            ticker.clone(),
            portfolio.clone().calculate_portfolio_history(&[(
                ticker,
                quotes.iter().map(|quote| quote.close).collect(),
            )]),
        ));
        all_quotes.extend(quotes);
    }

    pb.finish_and_clear();

    // find the largest number of data points
    let max_y = price_histories
        .iter()
        .map(|(_, history)| history.len())
        .max()
        .unwrap();

    let (x, y) = termion::terminal_size().unwrap();
    let mut chart = Chart::new((2 * x - 23).into(), (2 * y - 3).into(), 0.0, max_y as f32);

    let t = price_histories
        .iter()
        .map(|(_, history)| {
            history
                .iter()
                .enumerate()
                .map(|(j, value)| (j as f32, *value as f32))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    t.iter()
        .map(|series| textplots::Shape::Lines(series))
        .collect::<Vec<_>>()
        .iter()
        .fold(&mut chart, |chart, series| chart.lineplot(series))
        .x_axis_style(textplots::LineStyle::Solid)
        .y_axis_style(textplots::LineStyle::Solid)
        .display();

    Ok(())
}
