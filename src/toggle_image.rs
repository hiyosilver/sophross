fn generate_toggle_image(ui: &mut egui::Ui, on: &mut bool, image_src: &egui::ImageSource, tint: egui::Color32) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(1.5, 1.5);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.125 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);

        egui::Image::new(image_src.clone())
            .tint(tint)
            .texture_options(egui::TextureOptions { 
                magnification: egui::TextureFilter::Nearest,
                minification: egui::TextureFilter::Nearest,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
            })
            .paint_at(ui, rect);
    }

    response
}

pub fn toggle_image<'a>(on: &'a mut bool, image_src: &'a egui::ImageSource, tint: egui::Color32) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| generate_toggle_image(ui, on, image_src, tint)
}