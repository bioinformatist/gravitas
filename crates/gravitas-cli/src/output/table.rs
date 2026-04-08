use comfy_table::{presets, Attribute, Cell, Color, ContentArrangement, Table};
use gravitas_core::types::GexResult;

pub fn render_table(result: &GexResult) {
    let regime = if result.is_negative_gex_regime {
        "NEGATIVE GEX (trending)"
    } else {
        "POSITIVE GEX (mean-reverting)"
    };

    println!(
        "{} | Spot: {:.2} | {} | Nearest ZGL: {:.2}",
        result.symbol, result.spot_price, regime, result.nearest_zgl
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Strike").add_attribute(Attribute::Bold),
            Cell::new("Call GEX").add_attribute(Attribute::Bold),
            Cell::new("Put GEX").add_attribute(Attribute::Bold),
            Cell::new("Net GEX").add_attribute(Attribute::Bold),
            Cell::new("Vanna").add_attribute(Attribute::Bold),
        ]);

    for strike in &result.strikes {
        let net_color = if strike.net_gex >= 0.0 {
            Color::Green
        } else {
            Color::Red
        };

        table.add_row(vec![
            Cell::new(format!("{:.1}", strike.strike)),
            Cell::new(format_gex(strike.call_gex)),
            Cell::new(format_gex(strike.put_gex)),
            Cell::new(format_gex(strike.net_gex)).fg(net_color),
            Cell::new(format!("{:.0}", strike.vanna)),
        ]);
    }

    println!("{table}");

    if result.zero_gamma_levels.len() > 1 {
        print!("ZGLs: ");
        for (i, zgl) in result.zero_gamma_levels.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            if (*zgl - result.nearest_zgl).abs() < 0.01 {
                print!("{zgl:.2} (nearest)");
            } else {
                print!("{zgl:.2}");
            }
        }
        println!();
    }
}

fn format_gex(val: f64) -> String {
    let abs = val.abs();
    let sign = if val >= 0.0 { "+" } else { "-" };
    if abs >= 1e9 {
        format!("{sign}{:.2}B", abs / 1e9)
    } else if abs >= 1e6 {
        format!("{sign}{:.2}M", abs / 1e6)
    } else if abs >= 1e3 {
        format!("{sign}{:.1}K", abs / 1e3)
    } else {
        format!("{sign}{:.0}", abs)
    }
}
