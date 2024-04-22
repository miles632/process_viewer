use eframe::App;
use egui::{scroll_area, Color32, ScrollArea};

use crate::process_tree::{TreeNode, ZProcess};

// just using vec now for testing
pub struct EguiApp {
    ptree: Vec<ZProcess>,
}

impl EguiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, proc_vec: Vec<ZProcess>) -> Self {
        EguiApp {
            ptree: proc_vec,
        }
    }
}

impl Default for EguiApp {
    fn default() -> Self {
        EguiApp{
            ptree: vec![]
        }
    }
}

impl eframe::App for EguiApp{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // egui::TopBottomPanel::default_height(self, )
        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().auto_shrink([false;2]).show(ui, |ui|{
                for proc in &mut self.ptree {
                    ui.horizontal(|ui|{
                        let label = format!("{:?}", proc.pid);
                        ui.label(label);
                    });
                }
            });
        }); 
    }
}