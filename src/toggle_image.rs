use egui::{Color32, Vec2};

fn generate_toggle_image(
    ui: &mut egui::Ui,
    on: bool,
    always_tint: bool,
    image_src: &egui::ImageSource,
    tint: Color32,
    size: Vec2,
) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(1.5, 1.5);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, on, ""));

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.125 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);

        let image_rect = egui::Rect {
            min: rect.center() - size * 0.5,
            max: rect.center() + size * 0.5,
        };

        egui::Image::new(image_src.clone())
            .tint(if on || always_tint {
                tint
            } else {
                Color32::WHITE
            })
            .texture_options(egui::TextureOptions {
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
            })
            .paint_at(ui, image_rect);
    }

    response
}

pub fn toggle_image<'a>(
    on: bool,
    always_tint: bool,
    image_src: &'a egui::ImageSource,
    tint: Color32,
    size: Vec2,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| generate_toggle_image(ui, on, always_tint, image_src, tint, size)
}
