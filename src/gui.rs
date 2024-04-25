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

    fn output_procinfo(&mut self, ui: &mut egui::Ui, num_processes: &usize) {
        ui.add_space(4.0);

        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let row_height = ui.fonts(|f| f.row_height(&font_id) + ui.spacing().item_spacing.y);

        ScrollArea::vertical()
            .auto_shrink(false)
            .show_viewport(ui, |ui,viewport| {
                ui.set_height(row_height * *num_processes as f32);

                let first_item = (viewport.min.y / row_height).ceil() as usize;
                let last_item = (viewport.max.y / row_height).ceil() as usize + 1;
                let last_item = last_item.at_most(*num_processes);

                let mut used_rect = egui::Rect::NOTHING; 

                for i in first_item..last_item {
                    let identation = (i % 100) as f32;
                    let x = ui.min_rect().left() + identation;
                    let y = ui.min_rect().top() + i as f32 * row_height;

                    let text = format!("piss");

                    let text_rect = ui.painter().text(
                        pos2(x, y), 
                        egui::Align2::LEFT_TOP, 
                        text, 
                        font_id.clone(), 
                        ui.visuals().text_color(),
                    );
                    used_rect = used_rect.union(text_rect);
                }

                ui.allocate_rect(used_rect, egui::Sense::hover());
            });
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
        todo!()
    }

}