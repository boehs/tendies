use clap::Args;
use tendies::{portfolio, position::PositionVec};
use textplots::{AxisBuilder, Chart, ColorPlot};

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
        let mut portfolio = portfolio::Portfolio::new(&ticker, args.starting_balance);
        portfolio.add_position_by_value(
            &ticker,
            args.starting_balance,
            quotes.first().unwrap().close,
            chrono::Utc::now(),
        );
        price_histories.push((
            portfolio.clone(),
            portfolio.calculate_portfolio_history(&[(
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
    let mut chart = Chart::new((2 * x - 23).into(), (2 * y).into(), 0.0, max_y as f32);

    let t = price_histories
        .iter()
        .map(|(port, history)| {
            (
                port,
                history
                    .iter()
                    .enumerate()
                    .map(|(j, value)| (j as f32, *value as f32))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    t.iter()
        .map(|(port, series)| (port.color, textplots::Shape::Lines(series)))
        .collect::<Vec<_>>()
        .iter()
        .fold(&mut chart, |chart, (c, p)| chart.linecolorplot(p, *c))
        .x_axis_style(textplots::LineStyle::Solid)
        .y_axis_style(textplots::LineStyle::Solid)
        .display();

    // print a key for the colors, by shading █ to represent the color
    for (port, _) in t.iter() {
        print!(
            "\x1B[38;2;{};{};{}m█\x1b[0m {} ",
            port.color.r, port.color.g, port.color.b, port.name
        );
    }
    println!();

    Ok(())
}
