mod config;
mod output;

use clap::Parser;
use config::{resolve_source, Config, ResolvedSource};
use gravitas_core::gex::compute_gex;
use gravitas_fetch::mock::MockSource;
use gravitas_fetch::source::DataSource;
use gravitas_fetch::tradier::TradierSource;

#[derive(Parser)]
#[command(name = "gravitas", about = "GEX wall analysis for options scalping")]
struct Cli {
    /// Ticker symbol (e.g. SPY, AAPL, /ES)
    symbol: String,

    /// Filter by expiry date (YYYY-MM-DD)
    #[arg(short, long)]
    expiry: Option<String>,

    /// Only 0DTE options
    #[arg(short, long)]
    zero_dte: bool,

    /// Price scenario (e.g. +5, -3 for percentage)
    #[arg(short, long)]
    scenario: Option<f64>,

    /// Show Vanna exposure overlay
    #[arg(short, long)]
    vanna: bool,

    /// Auto-refresh interval in seconds
    #[arg(short, long)]
    watch: Option<u64>,

    /// Output format: ascii, json, table
    #[arg(short, long, default_value = "ascii")]
    format: String,

    /// Force data source: direct or api
    #[arg(long)]
    source: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = Config::load();

    let source: Box<dyn DataSource> =
        match resolve_source(cli.source.as_deref(), &config) {
            #[cfg(feature = "futu")]
            Ok(ResolvedSource::Futu { host, port }) => {
                Box::new(gravitas_fetch::futu::FutuSource::new(host, port))
            }
            #[cfg(not(feature = "futu"))]
            Ok(ResolvedSource::Futu { .. }) => {
                eprintln!("Futu source detected but binary was compiled without --features futu");
                eprintln!("Using mock data.");
                Box::new(MockSource::new())
            }
            Ok(ResolvedSource::Direct { tradier_token }) => {
                Box::new(TradierSource::new(tradier_token, false))
            }
            Ok(ResolvedSource::Api { api_key: _, api_base: _ }) => {
                eprintln!("API mode not yet implemented, falling back to mock data");
                Box::new(MockSource::new())
            }
            Err(e) => {
                eprintln!("Warning: {e}");
                eprintln!("Using mock data for demonstration.");
                Box::new(MockSource::new())
            }
        };

    let expiry_filter = if cli.zero_dte {
        Some(gravitas_core::types::ExpiryFilter::ZeroDte)
    } else {
        cli.expiry.map(|d| {
            let date = chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d")
                .expect("invalid date format, use YYYY-MM-DD");
            gravitas_core::types::ExpiryFilter::DateRange(date, date)
        })
    };

    loop {
        let spot = match source.fetch_spot_price(&cli.symbol).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error fetching spot price: {e}");
                return;
            }
        };

        let contracts = match source.fetch_options_chain(&cli.symbol, expiry_filter.clone()).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error fetching options chain: {e}");
                return;
            }
        };

        let spot = if let Some(pct) = cli.scenario {
            spot * (1.0 + pct / 100.0)
        } else {
            spot
        };

        let result = compute_gex(&cli.symbol, &contracts, spot, 0.05);

        match cli.format.as_str() {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            }
            "table" => {
                output::table::render_table(&result);
            }
            _ => {
                output::ascii::render_ascii(&result);
                if cli.vanna {
                    println!("\nVanna Exposure by Strike:");
                    for s in &result.strikes {
                        if s.vanna.abs() > 0.0 {
                            let dir = if s.vanna > 0.0 { "+" } else { "-" };
                            println!("  {:.1}  {dir}{:.0}", s.strike, s.vanna.abs());
                        }
                    }
                }
            }
        }

        match cli.watch {
            Some(secs) => {
                tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                // Clear screen for refresh
                print!("\x1b[2J\x1b[1;1H");
            }
            None => break,
        }
    }
}
