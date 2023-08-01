use eframe;
use eframe::egui::{Color32, FontFamily, FontId, RichText, TextEdit, TextStyle};
use eframe::egui;
use rodio::source::{SineWave, Source};
use rodio::{OutputStream, Sink};
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{task, time};

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Timer {
    label: String,
    hours: String,
    minutes: String,
    seconds: String,
    started_ms: Option<u64>,
    auto_restart: bool,
}

impl Timer {
    fn start(&mut self) {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        self.started_ms = Some(since_the_epoch.as_millis() as u64);
    }

    fn reset(&mut self) {
        self.started_ms = None;
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            label: String::from(""),
            hours: String::from("0"),
            minutes: String::from("0"),
            seconds: String::from("0"),
            started_ms: None,
            auto_restart: false,
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    timers: Vec<Timer>,
    flash_count: u32,
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Switch to light mode
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Body, FontId::new(28.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(24.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(16.0, FontFamily::Proportional)),
        ]
            .into();
        cc.egui_ctx.set_style(style);

        let ctx = cc.egui_ctx.to_owned();
        task::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;
                ctx.request_repaint();
            }
        });

        let mut app = Self::default();

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            app = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        }

        app
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self { timers: vec![], flash_count: 0 }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { timers, flash_count } = self;

        ctx.set_pixels_per_point(2.0f32);
        let now = chrono::Local::now();
        let time = now.format("%H:%M:%S").to_string();
        let date = now.format("%Y/%m/%d %a").to_string();

        let mut frm = egui::containers::Frame::default();
        frm = frm.fill( if *flash_count % 2 == 1 { Color32::RED } else { Color32::LIGHT_GRAY });
        if *flash_count > 0 {
            *flash_count -= 1;
        }

        egui::CentralPanel::default().frame(frm).show(ctx, |ui| {
            // Current time
            ui.horizontal(|ui| {
                ui.label(RichText::from(time));
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(RichText::from(date).small());
                });
            });

            // Timers
            let mut remove_indices = vec![];
            for (idx, timer) in timers.as_mut_slice().iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if let Some(s) = timer.started_ms {
                        let now = SystemTime::now();
                        let start = UNIX_EPOCH.add(Duration::from_millis(s));
                        let since_start = now.duration_since(start).expect("Time went backwards");
                        let hours = timer.hours.parse::<u32>().expect("invalid hours");
                        let minutes = timer.minutes.parse::<u32>().expect("invalid minutes");
                        let seconds = timer.seconds.parse::<u32>().expect("invalid seconds");
                        let total_secs = hours * 3600 + minutes * 60 + seconds;
                        let timer_duration = Duration::from_secs(total_secs as u64);
                        if timer_duration <= since_start {
                            // time up
                            ui.colored_label(Color32::RED, "00:00:00");

                            if timer.auto_restart {
                                play_beep();
                                *flash_count = 10;
                                timer.start();
                            }
                        } else {
                            let rest = timer_duration - since_start;
                            let rest_sec = rest.as_secs();

                            let rest_h = rest_sec / 3600;
                            let rest_m = (rest_sec % 3600) / 60;
                            let rest_s = rest_sec % 60;

                            ui.label(format!("{:02}:{:02}:{:02}", rest_h, rest_m, rest_s));
                        }

                        if ui.button("Reset").clicked() {
                            timer.reset();
                        }
                    } else {
                        TextEdit::singleline(&mut timer.hours)
                            .desired_width(40.0)
                            .show(ui)
                            .response;
                        ui.label(":");
                        TextEdit::singleline(&mut timer.minutes)
                            .desired_width(40.0)
                            .show(ui)
                            .response;
                        ui.label(":");
                        TextEdit::singleline(&mut timer.seconds)
                            .desired_width(40.0)
                            .show(ui)
                            .response;
                        if ui.button("Start").clicked() {
                            timer.start();
                        }
                    }
                    if ui.button("x").clicked() {
                        remove_indices.push(idx);
                    }
                });
                ui.horizontal(|ui| {
                    TextEdit::singleline(&mut timer.label)
                        .desired_width(160.0)
                        .show(ui)
                        .response;
                    ui.checkbox(&mut timer.auto_restart, "Loop");
                });
            }
            for idx in remove_indices {
                timers.remove(idx);
            }

            if ui.button("+ New timer").clicked() {
                timers.push(Timer::default());
            }
        });
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

fn play_beep() {
    task::spawn(async {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // Add a dummy source of the sake of the example.
        let duration = Duration::from_secs_f32(0.25);
        let g5 = SineWave::new(783.9).take_duration(duration);
        let f5 = SineWave::new(698.4).take_duration(duration);
        let potato = g5
            .clone()
            .mix(f5.clone().delay(duration))
            .mix(g5.clone().delay(duration * 2));
        let potato2 = potato.clone().mix(potato.delay(duration * 4));
        sink.append(potato2);
        // sleep_until_end is required, so run beep process in a dedicated thread.
        sink.sleep_until_end();
    });
}
