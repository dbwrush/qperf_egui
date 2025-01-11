extern crate lazy_static;
use eframe::egui::{self};
use qperformance::qperf;
use rfd::FileDialog;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder { 
            inner_size: Some(egui::vec2(320.0, 600.0)),
            ..Default::default()}
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("assets/icon.png"))
                    .unwrap_or_default(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "QPerformance",
        options,
        Box::new(|_cc| Ok(Box::new(QpApp::default()))),
    )
}

struct QpApp {
    questions_path: String,
    logs_path: String,
    output_path: String,
    status_message: String,
    warns: Vec<String>,
    checked: Vec<bool>,
}

impl Default for QpApp {
    fn default() -> Self {
        Self {
            questions_path: String::new(),
            logs_path: String::new(),
            output_path: String::new(),
            status_message: String::new(),
            warns: Vec::new(),
            checked: [true, true, true, true, true, true, true, true, true].to_vec(),
        }
    }
}

impl eframe::App for QpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.visuals_mut().override_text_color = None;
            ui.heading("QPerformance");
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Questions Sets:");
                if ui.button("Single file(.rtf)").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("RTF files", &["rtf"]).pick_file() {
                        self.questions_path = path.display().to_string();
                    }
                }
                if ui.button("Folder").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.questions_path = path.display().to_string();
                    }
                }
            });
            ui.label(format!("Selected: {}", self.questions_path.clone()));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("QuizMachine Records:");
                if ui.button("Select File(.csv)").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("CSV files", &["csv"]).pick_file() {
                        self.logs_path = path.display().to_string();
                    }
                }
            });
            ui.label(format!("Selected: {}", self.logs_path.clone()));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Output Location:");
                if ui.button("Select Output File(.csv)").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("CSV file", &["csv"]).save_file() {
                        self.output_path = path.display().to_string();
                    }
                }
            });
            ui.label(format!("Selected: {}", self.output_path.clone()));

            ui.add_space(10.0);

            ui.label("Question Types:");
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.checked[0], "A");
                ui.checkbox(&mut self.checked[1], "G");
                ui.checkbox(&mut self.checked[2], "I");
                ui.checkbox(&mut self.checked[3], "Q");
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.checked[4], "R");
                ui.checkbox(&mut self.checked[5], "S");
                ui.checkbox(&mut self.checked[6], "X");
                ui.checkbox(&mut self.checked[7], "V");
            });

            ui.checkbox(&mut self.checked[8], "Memory Verse totals (Q, R, V)");

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(0, 177, 0));
                if ui.button("Run").clicked() {
                    self.run_command();
                }
                ui.visuals_mut().override_text_color = None;
                if ui.button("Clear").clicked() {
                    self.questions_path.clear();
                    self.logs_path.clear();
                    self.output_path.clear();
                    self.status_message.clear();
                    self.warns.clear();
                }
            });

            ui.add_space(10.0);

            ui.label(format!("Status: {}", self.status_message));

            if self.warns.len() > 0 {
                ui.add_space(10.0);
                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                ui.label("Warnings:");
                for warn in &self.warns {
                    ui.label(warn);
                }
                ui.visuals_mut().override_text_color = None;
            }

            ui.add_space(20.0);

            ui.label("How to use:");
            ui.label("1. Select the question set location. Either a single .RTF file, or a folder containing multiple files");
            ui.label("2. Select the QuizMachine records file (.csv).");
            ui.label("3. Select the output file location (.csv)");
            ui.label("4. Click Run. Results are saved to the chosen location");
        });
    }
}

impl QpApp {
    fn run_command(&mut self) {
        self.warns = Vec::new();
        // Validate input paths
        if !Path::new(&self.questions_path).exists() {
            self.status_message = "Question set location does not exist.".to_string();
            return;
        }

        if !Path::new(&self.logs_path).is_file() {
            self.status_message = "QuizMachine records file does not exist.".to_string();
            return;
        }

        if Path::new(&self.output_path).exists() {
            self.status_message = "Output file already exists. Choose a different file name.".to_string();
            return;
        }

        let mut types = Vec::new();
        for i in 0..9 {
            if self.checked[i] {
                types.push(['A', 'G', 'I', 'Q', 'R', 'S', 'X', 'V', 'M'][i]);
            }
        }

        // Call the qperf function
        match qperf(&self.questions_path, &self.logs_path, false, types) {
            Ok(result) => {
                // Write the result to the output file
                self.warns = result.0;
                //eprintln!("Added warns: {:?}", self.warns);
                let output = result.1;
                match fs::File::create(&self.output_path) {
                    Ok(mut file) => {
                        if file.write_all(output.as_bytes()).is_ok() {
                            self.status_message = "Saved".to_string();
                        } else {
                            self.status_message = "Error writing to output file".to_string();
                        }
                    }
                    Err(_) => {
                        self.status_message = "Error creating output file".to_string();
                    }
                }
            }
            Err(_) => {
                self.status_message = "Error running qperf function".to_string();
            }
        }
    }
}
