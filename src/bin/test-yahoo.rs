use clap::Parser;
use rgb::RGB8;
use textplots::{AxisBuilder, Chart, ColorPlot, Plot};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    tickers: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let provider = yahoo_finance_api::YahooConnector::new().unwrap();

    let mut all_quotes = vec![];

    for ticker in args.tickers {
        let response = provider.get_quote_range(&ticker, "1wk", "10y").await?;
        let quotes = match response.quotes() {
            Ok(quotes) => quotes
                .iter()
                .enumerate()
                .map(|(i, q)| (i as f32, q.adjclose as f32))
                .collect::<Vec<(f32, f32)>>(),
            Err(_) => {
                eprintln!("No quotes found for ticker {}", ticker);
                continue;
            }
        };

        all_quotes.push((ticker, quotes));
    }

    let max_x = all_quotes
        .clone()
        .iter()
        .flat_map(|(_, quotes)| quotes.iter().map(|(x, _)| *x))
        .fold(f32::NAN, f32::max);

    let shapes = all_quotes
        .iter()
        .map(|(_, s)| textplots::Shape::Lines(s))
        .collect::<Vec<_>>();
    let mut chart = Chart::new(200, 50, 0.0, max_x);
    let mut chart_ref = &mut chart;
    for shape in &shapes {
        chart_ref = chart_ref.lineplot(shape);
    }

    chart_ref
        .x_axis_style(textplots::LineStyle::Solid)
        .y_axis_style(textplots::LineStyle::Solid)
        .display();

    Ok(())
}
