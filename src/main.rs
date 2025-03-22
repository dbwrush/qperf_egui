#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //hide console in Windows releases
extern crate qperf_lib;

use qperf_lib::qperf;
use eframe::egui::{self};
use rfd::FileDialog;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder { 
            inner_size: Some(egui::vec2(320.0, 850.0)),
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
    delimiter: String,
    tourn: String,
    warns: Vec<String>,
    checked: Vec<bool>,
    display_individual_rounds: bool,
    disp_paths: (String, String),
    output: String,
    ready_save: String,
}

impl Default for QpApp {
    fn default() -> Self {
        Self {
            questions_path: String::new(),
            logs_path: String::new(),
            output_path: String::new(),
            status_message: "Select question set(s)!".to_string(),
            delimiter: ",".to_string(),
            tourn: "".to_string(),
            warns: Vec::new(),
            checked: [true, true, true, true, true, true, true, true, true].to_vec(),
            display_individual_rounds: false,
            disp_paths: (String::new(), String::new()),
            output: String::new(),
            ready_save: String::new(),
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
                if ui.button("Pick files(.rtf)").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("RTF files", &["rtf"]).pick_files() {
                        //combine all paths to a single string separated by commas
                        self.questions_path = path.iter().map(|p| p.display().to_string()).collect::<Vec<String>>().join(",");

                        //set disp_path. Should show only the shortened (individual name) for each file, separated by commas
                        self.disp_paths.0 = path.iter().map(|p| p.file_name().unwrap().to_str().unwrap().to_string()).collect::<Vec<String>>().join(",");

                        self.status_message = "Select QuizMachine log(s)!".to_string();
                    }
                }
            });
            ui.label(format!("Selected: {}", self.disp_paths.0.clone()));

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("QuizMachine Records:");
                if ui.button("Pick Files(.csv)").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("CSV files", &["csv"]).pick_files() {
                        //combine all paths to a single string separated by commas
                        self.logs_path = path.iter().map(|p| p.display().to_string()).collect::<Vec<String>>().join(",");

                        //set disp_path. Should show only the shortened (individual name) for each file, separated by commas
                        self.disp_paths.1 = path.iter().map(|p| p.file_name().unwrap().to_str().unwrap().to_string()).collect::<Vec<String>>().join(",");

                        self.status_message = "Ready to run!\n(Recommended) Enter a tournament name for better filtering!".to_string();
                    }
                }
            });
            ui.label(format!("Selected: {}", self.disp_paths.1.clone()));

            ui.add_space(10.0);

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

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Delimiter: ");
                ui.text_edit_singleline(&mut self.delimiter);
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Tournament: ");
                ui.text_edit_singleline(&mut self.tourn);
            });
            //Warn briefly that the tournament name is to filter out junk data from practice/other events.
            ui.label("(Recommended) Fill in to exclude data from other tournaments!");

            ui.add_space(10.0);

            ui.checkbox(&mut self.display_individual_rounds, "Display individual round results");


            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(0, 177, 0));
                if self.questions_path.is_empty() || self.logs_path.is_empty() {
                    //show Run button grayed out if output is empty
                    ui.add_enabled(false, egui::Button::new("Run"));
                } else if ui.button("Run").clicked() {
                        self.run_command();
                }
                ui.visuals_mut().override_text_color = None;
                if ui.button("Clear").clicked() {
                    self.questions_path.clear();
                    self.logs_path.clear();
                    self.output_path.clear();
                    self.status_message = "Select question set(s)!".to_string();
                    self.warns.clear();
                    self.delimiter = ",".to_string();
                    self.tourn = "".to_string();
                    self.display_individual_rounds = false;
                    self.disp_paths = (String::new(), String::new());
                    self.checked = [true, true, true, true, true, true, true, true, true].to_vec();
                    self.output.clear();
                    self.ready_save.clear();
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

            ui.visuals_mut().override_text_color = Some(egui::Color32::from_rgb(0, 177, 0));
            ui.label(format!("{}", self.ready_save));
            ui.visuals_mut().override_text_color = None;
            
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Save Output:");
                if self.output.is_empty() {
                    //show Save button grayed out if output is empty
                    ui.add_enabled(false, egui::Button::new("Save File (.csv)"));
                } else if ui.button("Save File (.csv)").clicked() {
                    //check that output isn't blank
                    if self.output.is_empty() {
                        self.status_message = "Run the program before saving!".to_string();
                    } else {
                        if let Some(path) = FileDialog::new().add_filter("CSV file", &["csv"]).save_file() {
                            self.output_path = path.display().to_string();
                            self.write_output();
                        }
                    }
                }
            });
            ui.add_space(20.0);

            ui.label("How to use:");
            ui.label("1. Select the question set file(s), must be original RTF files from Set Maker!");
            ui.label("2. Select the QuizMachine records (.csv) file(s).");
            ui.label("3. Choose question types, delimiter, and tournament name");
            ui.label("4. Click Run, check Status for errors!");
            ui.label("5. Save the output file.");
        });
    }
}

impl QpApp {
    fn run_command(&mut self) {
        self.warns = Vec::new();
        // Validate input paths

        let mut tourn_name = self.tourn.clone();

        //If tournament does not have '' at beginning AND end, add them. QuizMachine records have them.
        if tourn_name != "" {
            if !tourn_name.starts_with('\'') {
                tourn_name = format!("'{}", tourn_name);
                if !tourn_name.ends_with('\'') {
                    tourn_name = format!("{}'", tourn_name);
                }
            }
        }

        //iterate through questions_path and logs_path to check if they are valid
        for path in self.questions_path.split(",") {
            if !Path::new(&path).exists() {
                self.status_message = "Question set location does not exist.".to_string();
                return;
            }
        }

        for path in self.logs_path.split(",") {
            if !Path::new(&path).is_file() {
                self.status_message = "QuizMachine records file does not exist.".to_string();
                return;
            }
        }

        let mut types = Vec::new();
        for i in 0..9 {
            if self.checked[i] {
                types.push(['A', 'G', 'I', 'Q', 'R', 'S', 'X', 'V', 'M'][i]);
            }
        }

        // Call the qperf function
        match qperf(&self.questions_path, &self.logs_path, false, types, self.delimiter.clone(), tourn_name, self.display_individual_rounds) {
            Ok(result) => {
                // Write the result to the output file
                self.warns = result.0;
                //eprintln!("Added warns: {:?}", self.warns);
                self.output = result.1;
                self.ready_save = "Ready to save".to_string();
            }
            Err(_) => {
                self.status_message = "Error running qperf function".to_string();
            }
        }
    }

    fn write_output(&mut self) {
        if Path::new(&self.output_path).exists() {
            self.status_message = "Output file already exists. Choose a different file name.".to_string();
            return;
        }
        //check if path is valid
        if self.output_path.is_empty() {
            self.status_message = "Output file path is empty!".to_string();
            return;
        }
        match fs::File::create(&self.output_path) {
            Ok(mut file) => {
                if file.write_all(self.output.as_bytes()).is_ok() {
                    self.ready_save = "Saved successfully".to_string();
                } else {
                    self.status_message = "Error writing to output file".to_string();
                }
            }
            Err(_) => {
                self.status_message = "Error creating output file".to_string();
            }
        }
    }
}
