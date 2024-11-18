#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod totp_code;

use eframe::{egui, Storage};
use std::time::Duration;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([460.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TOTP CLIENT",
        native_options,
        Box::new(move |cc| Ok(Box::new(MyApp::new(cc)))),
    )
}


#[derive(Default, serde::Serialize, serde::Deserialize)]
struct TOTPInfo {
    user: String,
    secret_key: String,
    totp: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MyApp {
    totp_infos: Vec<TOTPInfo>,
    del_index: usize,
    progress: f32,
    is_first_update: bool,
    is_show_adding_page: bool,
    adding_page_user: String,
    adding_page_secret_key: String,
    adding_page_error_msg: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let totp_infos = vec![
            TOTPInfo {
                user: "David@example.com".to_string(),
                secret_key: "C274LGIUBYJTPSM5".to_string(),
                totp: "666666".to_string(),
            },
            TOTPInfo {
                user: "JumpServer".to_string(),
                secret_key: "C274LGIUBYJTPSM6".to_string(),
                totp: "666666".to_string(),
            },
            TOTPInfo {
                user: "you@example.com".to_string(),
                secret_key: "C274LGIUBYJTPSM7".to_string(),
                totp: "666666".to_string(),
            }
        ];
        Self {
            is_first_update: false,
            is_show_adding_page: false,
            progress: 0.0,
            totp_infos,
            del_index: 9999,
            adding_page_user: String::new(),
            adding_page_secret_key: String::new(),
            adding_page_error_msg: String::new(),
        }
    }
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Self::setup_custom_fonts(&cc.egui_ctx);
        let mut obj = Self::default();
        if let Some(storage) = cc.storage {
            obj.totp_infos = eframe::get_value(storage, "totp_infos").unwrap_or(obj.totp_infos)
        };
        obj
    }

    // fn setup_custom_fonts(ctx: &egui::Context) {
    //     let mut fonts = egui::FontDefinitions::default();
    // 
    //     // 假设你已经将系统字体文件嵌入到了你的项目中
    //     let system_font_data = include_bytes!("C:\\Windows\\Fonts\\MSYH.TTC");
    //     fonts.font_data.insert(
    //         "MSYH".to_owned(),
    //         egui::FontData::from_static(system_font_data),
    //     );
    // 
    //     // 将系统字体设置为默认字体家族的一部分
    //     fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "MSYH".to_owned());
    // 
    //     // 应用字体定义
    //     ctx.set_fonts(fonts);
    // }

    fn timer(&mut self) {
        let count = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() % 30 + 1;
        self.progress = count as f32 / 30.0;
        if count == 1 || !self.is_first_update {
            self.update_totp();
            self.is_first_update = true;
        }
    }
    fn update_totp(&mut self) {
        self.totp_infos.iter_mut().for_each(|otp| {
            otp.totp = totp_code::generate_totp(&otp.secret_key).unwrap();
        })
    }


    fn add_user(&mut self) {
        // 检测用户或密钥是否为空
        if self.adding_page_user.is_empty() {
            self.adding_page_error_msg = "The user cannot be empty.".to_string();
            return;
        }
        if self.adding_page_secret_key.is_empty() {
            self.adding_page_error_msg = "The secret cannot be empty.".to_string();
            return;
        }

        // 用户如果已存在就不添加,给于一个已存在的提示
        for item in &self.totp_infos {
            if item.user == self.adding_page_user {
                self.adding_page_error_msg = "The user already exists.".to_string();
                return;
            }
        }
        // 测试secret_key是否有效
        let number = totp_code::generate_totp(&self.adding_page_secret_key);
        if number == None {
            self.adding_page_error_msg = "The secret is invalid.".to_string();
            return;
        }
        // 添加
        self.totp_infos.push(TOTPInfo {
            user: self.adding_page_user.clone(),
            secret_key: self.adding_page_secret_key.clone(),
            totp: number.unwrap(),
        });

        self.is_show_adding_page = false;
        self.adding_page_error_msg.clear();
        self.adding_page_user.clear();
        self.adding_page_secret_key.clear();
    }

    fn del_user(&mut self) {
        if self.del_index == 9999 { return; }
        self.totp_infos.remove(self.del_index);
        self.del_index = 9999;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.timer();
        self.main_page(ctx);
        self.adding_page(ctx);
        self.del_user();
        ctx.request_repaint_after(Duration::from_secs(1));
    }
    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "totp_infos", &self.totp_infos);
    }
}

impl MyApp {
    fn main_page(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(15.0);
                self.totp_infos.iter().enumerate().for_each(
                    |(idx, otp)| {
                        ui.vertical(|ui| {
                            ui.label(format!("{}", otp.user));
                            ui.horizontal(|ui| {
                                let number_text = egui::RichText::new(&otp.totp).color(egui::Color32::from_rgb(0, 45, 179)).size(24.0);
                                let number_label = ui.label(number_text).on_hover_text("Double click to copy");
                                if number_label.double_clicked() {
                                    ctx.copy_text(otp.totp.clone());
                                };
                                number_label.context_menu(|ui| {
                                    if ui.button("COPY").clicked() {
                                        ctx.copy_text(otp.totp.clone());
                                        ui.close_menu();
                                    }
                                    if ui.button("COPY USER").clicked() {
                                        ctx.copy_text(otp.user.clone());
                                        ui.close_menu();
                                    }
                                    ui.separator();
                                    if ui.button("DELETE").clicked() {
                                        self.del_index = idx;
                                        ui.close_menu();
                                    }
                                });
                            });
                            ui.add(egui::ProgressBar::new(self.progress).desired_height(5.0));
                            ui.add_space(20.0);
                        });
                    });
                ui.vertical_centered(|ui| {
                    if ui.button(egui::RichText::new("ADD").color(egui::Color32::DARK_GREEN).size(24.0)).clicked() {
                        self.is_show_adding_page = true;
                    }
                });
            });
        });
    }

    fn adding_page(&mut self, ctx: &egui::Context) {
        if !self.is_show_adding_page { return; }
        egui::Window::new("Add NEW")
            .resizable(true)
            .collapsible(false)
            .default_pos(ctx.screen_rect().center())
            .max_height(120.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);
                    ui.add(egui::TextEdit::singleline(&mut self.adding_page_user).hint_text("Please enter user"));
                    ui.add_space(10.0);
                    ui.add(egui::TextEdit::singleline(&mut self.adding_page_secret_key).hint_text("Please enter you secret"));
                    ui.add_space(15.0);
                    ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::RightToLeft), |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            if ui.button(egui::RichText::new("ADD").color(egui::Color32::DARK_GREEN).size(15.0)).clicked() {
                                self.add_user();
                            }
                            if ui.button(egui::RichText::new("CANCEL").size(15.0)).clicked() {
                                self.is_show_adding_page = false;
                                self.adding_page_error_msg.clear();
                            }
                            if !self.adding_page_error_msg.is_empty() {
                                ui.label(egui::RichText::new(&self.adding_page_error_msg).color(egui::Color32::RED).size(15.0));
                            }
                        })
                    });
                    ui.add_space(10.0);
                })
            });
    }
}


    

