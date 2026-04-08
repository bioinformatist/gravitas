use gravitas_core::types::GexResult;

pub fn render_ascii(result: &GexResult) {
    let regime = if result.is_negative_gex_regime {
        "NEGATIVE GEX (trending)"
    } else {
        "POSITIVE GEX (mean-reverting)"
    };

    println!(
        "{sym}  GEX Wall  |  Spot: {spot:.2}  |  {regime}  |  Nearest ZGL: {zgl:.2}",
        sym = result.symbol,
        spot = result.spot_price,
        regime = regime,
        zgl = result.nearest_zgl,
    );
    if result.zero_gamma_levels.len() > 1 {
        let others: Vec<String> = result
            .zero_gamma_levels
            .iter()
            .filter(|z| (**z - result.nearest_zgl).abs() > 0.01)
            .map(|z| format!("{z:.2}"))
            .collect();
        if !others.is_empty() {
            println!("  Other ZGLs: {}", others.join(", "));
        }
    }
    println!();

    if result.strikes.is_empty() {
        println!("  (no strikes with valid data)");
        return;
    }

    let max_abs = result
        .strikes
        .iter()
        .map(|s| s.net_gex.abs())
        .fold(0.0_f64, f64::max);

    let bar_width = 30;

    println!("{:<10} {:>12}   {}", "Strike", "Net GEX", "Distribution");
    println!("{}", "-".repeat(60));

    for strike in &result.strikes {
        let bar_len = if max_abs > 0.0 {
            ((strike.net_gex.abs() / max_abs) * bar_width as f64) as usize
        } else {
            0
        };

        let bar: String = if strike.net_gex >= 0.0 {
            "\u{2588}".repeat(bar_len)
        } else {
            "\u{2588}".repeat(bar_len)
        };

        let label = if strike.net_gex >= 0.0 {
            if bar_len == bar_width { " CALL WALL" } else { "" }
        } else if bar_len == bar_width {
            " PUT WALL"
        } else {
            ""
        };

        let gex_str = format_gex(strike.net_gex);

        // Mark ZGL between strikes
        let is_near_zgl = result.zero_gamma_levels.iter().any(|zgl| {
            *zgl >= strike.strike - 2.5 && *zgl <= strike.strike + 2.5
        });

        if is_near_zgl {
            println!(
                "  [ZGL]    {:>12}   {}",
                format!("{:.2}", result.nearest_zgl),
                "-".repeat(bar_width)
            );
        }

        println!(
            "{:<10} {:>12}   {}{label}",
            format!("{:.1}", strike.strike),
            gex_str,
            bar,
        );
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
