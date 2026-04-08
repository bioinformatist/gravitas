use gravitas_core::gex::compute_gex;
use gravitas_core::types::{GexResult, OptionContract};

const DEFAULT_API_URL: &str = env!("GRAVITAS_API_URL");

pub struct GravitasApp {
    symbol: String,
    api_url: String,
    contracts: Vec<OptionContract>,
    gex_result: Option<GexResult>,
    spot_price: f64,
    scenario_pct: f64,
    show_vanna: bool,
    auto_refresh: bool,
    refresh_secs: u64,
    last_fetch: Option<f64>,
    fetch_pending: bool,
    error_msg: Option<String>,
}

impl Default for GravitasApp {
    fn default() -> Self {
        Self {
            symbol: "SPY".to_string(),
            api_url: DEFAULT_API_URL.to_string(),
            contracts: Vec::new(),
            gex_result: None,
            spot_price: 0.0,
            scenario_pct: 0.0,
            show_vanna: false,
            auto_refresh: false,
            refresh_secs: 60,
            last_fetch: None,
            fetch_pending: false,
            error_msg: None,
        }
    }
}

impl GravitasApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn trigger_fetch(&mut self, ctx: &egui::Context) {
        if self.fetch_pending {
            return;
        }
        self.fetch_pending = true;
        self.error_msg = None;

        let api_url = self.api_url.clone();
        let symbol = self.symbol.clone();
        let ctx = ctx.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let result = crate::fetch::fetch_options(&api_url, &symbol).await;
            ctx.memory_mut(|mem| {
                mem.data.insert_temp("fetch_result".into(), result);
            });
            ctx.request_repaint();
        });
    }

    fn recompute_gex(&mut self) {
        if self.contracts.is_empty() || self.spot_price <= 0.0 {
            return;
        }
        let spot = self.spot_price * (1.0 + self.scenario_pct / 100.0);
        self.gex_result = Some(compute_gex(&self.symbol, &self.contracts, spot, 0.05));
    }
}

impl eframe::App for GravitasApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Check for fetch results
        let fetch_result: Option<Result<Vec<OptionContract>, String>> =
            ctx.memory_mut(|mem| mem.data.get_temp("fetch_result".into()));

        if let Some(result) = fetch_result {
            self.fetch_pending = false;
            self.last_fetch = Some(chrono::Utc::now().timestamp() as f64);
            ctx.memory_mut(|mem| {
                mem.data.remove::<Result<Vec<OptionContract>, String>>("fetch_result".into());
            });
            match result {
                Ok(contracts) => {
                    if !contracts.is_empty() {
                        let mut strikes: Vec<f64> = contracts.iter().map(|c| c.strike).collect();
                        strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        strikes.dedup();
                        self.spot_price = strikes[strikes.len() / 2];
                    }
                    self.contracts = contracts;
                    self.recompute_gex();
                }
                Err(e) => {
                    self.error_msg = Some(e);
                }
            }
        }

        // Auto-refresh
        if self.auto_refresh {
            if let Some(last) = self.last_fetch {
                let now = chrono::Utc::now().timestamp() as f64;
                if now - last >= self.refresh_secs as f64 {
                    self.trigger_fetch(&ctx);
                }
            }
        }

        // --- Sidebar ---
        egui::Panel::left("sidebar").min_size(160.0).show_inside(ui, |ui| {
            ui.heading("gravitas");
            ui.separator();

            ui.label("Symbol:");
            let response = ui.text_edit_singleline(&mut self.symbol);
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.trigger_fetch(&ctx);
            }

            if ui.button("Fetch").clicked() {
                self.trigger_fetch(&ctx);
            }

            ui.separator();
            ui.label("Scenario (%):");
            let old_pct = self.scenario_pct;
            ui.add(egui::Slider::new(&mut self.scenario_pct, -10.0..=10.0).suffix("%"));
            if (self.scenario_pct - old_pct).abs() > 0.01 {
                self.recompute_gex();
            }

            ui.separator();
            ui.checkbox(&mut self.show_vanna, "Vanna overlay");

            ui.separator();
            ui.checkbox(&mut self.auto_refresh, "Auto-refresh");
            if self.auto_refresh {
                ui.add(
                    egui::Slider::new(&mut self.refresh_secs, 10..=300)
                        .suffix("s")
                        .text("Interval"),
                );
            }

            ui.separator();
            if self.fetch_pending {
                ui.spinner();
                ui.label("Fetching...");
            }
            if let Some(ref err) = self.error_msg {
                ui.colored_label(egui::Color32::RED, err);
            }
        });

        // --- Main area ---
        if let Some(ref result) = self.gex_result {
            let regime = if result.is_negative_gex_regime {
                "NEGATIVE GEX (trending)"
            } else {
                "POSITIVE GEX (mean-reverting)"
            };

            ui.horizontal(|ui| {
                ui.heading(format!("GEX Wall - {}", result.symbol));
                ui.label(format!("Spot: {:.2}", result.spot_price));
                ui.label(regime);
                ui.label(format!("Nearest ZGL: {:.2}", result.nearest_zgl));
            });

            ui.separator();

            let bars: Vec<egui_plot::Bar> = result
                .strikes
                .iter()
                .map(|s| {
                    egui_plot::Bar::new(s.strike, s.net_gex)
                        .width(2.0)
                        .fill(if s.net_gex >= 0.0 {
                            egui::Color32::from_rgb(0, 180, 80)
                        } else {
                            egui::Color32::from_rgb(220, 50, 50)
                        })
                })
                .collect();

            let chart = egui_plot::BarChart::new("Net GEX", bars);

            egui_plot::Plot::new("gex_plot")
                .height(ui.available_height() - 30.0)
                .x_axis_label("Strike")
                .y_axis_label("Net GEX ($)")
                .show(ui, |plot_ui| {
                    plot_ui.bar_chart(chart);

                    for &zgl in &result.zero_gamma_levels {
                        plot_ui.vline(
                            egui_plot::VLine::new("ZGL", zgl)
                                .color(egui::Color32::YELLOW)
                                .style(egui_plot::LineStyle::dashed_dense()),
                        );
                    }

                    plot_ui.vline(
                        egui_plot::VLine::new("Spot", result.spot_price)
                            .color(egui::Color32::WHITE),
                    );

                    if self.show_vanna {
                        let vanna_points: Vec<[f64; 2]> = result
                            .strikes
                            .iter()
                            .map(|s| [s.strike, s.vanna])
                            .collect();
                        plot_ui.line(
                            egui_plot::Line::new("Vanna", vanna_points)
                                .color(egui::Color32::from_rgb(100, 150, 255)),
                        );
                    }
                });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Enter a symbol and click Fetch to load GEX data.");
            });
        }
    }
}
