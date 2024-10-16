use clap::Parser;
use rgb::RGB8;
use textplots::{AxisBuilder, Chart, ColorPlot, Plot};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    ticker: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let provider = yahoo_finance_api::YahooConnector::new().unwrap();
    let response = provider.get_quote_range(&args.ticker, "1wk", "10y").await?;
    let quotes = match response.quotes() {
        Ok(quotes) => quotes
            .iter()
            .enumerate()
            .map(|(i, q)| (i as f32, q.adjclose as f32))
            .collect::<Vec<(f32, f32)>>(),
        Err(_) => {
            eprintln!("No quotes found for ticker {}", args.ticker);
            return Ok(());
        }
    };

    let max_x = quotes.iter().map(|(x, _)| *x).fold(f32::NAN, f32::max);

    // simulate a 3x leveraged etf by multiplying movement by 3
    // Simulate a 3x leveraged ETF by multiplying daily movements by 3
    let leveraged_quotes: Vec<(f32, f32)> = quotes
        .windows(2)
        .scan(quotes[0].1, |state, window| {
            let (_, prev_close) = window[0];
            let (_, curr_close) = window[1];

            // Calculate daily return
            let daily_return = (curr_close - prev_close) / prev_close;
            // Apply 3x leverage
            let leveraged_return = 3.0 * daily_return;
            // Cumulatively apply the leveraged return
            *state *= 1.0 + leveraged_return;

            Some((window[1].0, *state)) // Return the updated day and leveraged close
        })
        .collect();

    Chart::new(200, 50, 0.0, max_x)
        .lineplot(&textplots::Shape::Lines(&quotes))
        .linecolorplot(
            &textplots::Shape::Lines(&leveraged_quotes),
            RGB8 { r: 255, g: 0, b: 0 },
        )
        .x_axis_style(textplots::LineStyle::Solid)
        .y_axis_style(textplots::LineStyle::Solid)
        .display();

    Ok(())
}
