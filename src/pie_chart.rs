use egui::{Color32, Vec2};

pub struct PieChartSlice {

}

fn generate_pie_chart(ui: &mut egui::Ui, size: Vec2) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * size;
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    if response.hovered() {
        response.mark_changed();
    }
    //response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, on, ""));

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().noninteractive();
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .circle_filled(rect.center(), radius, Color32::LIGHT_RED);
        
        let mut slice: Vec<egui::Pos2> = Vec::new();

        slice.push(egui::pos2(rect.center().x, rect.center().y));
        slice.push(egui::pos2(rect.center().x, 0.0));
        slice.push(egui::pos2(rect.center().x, 0.0));

        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                rect.min,
                egui::pos2(rect.min.x + 32.0, rect.min.y),
                egui::pos2(rect.min.x + 16.0, rect.min.y + 32.0),
            ],
            egui::Color32::GRAY,
            egui::Stroke::NONE,
        ));
    }

    response
}

pub fn pie_chart<'a>(size: Vec2) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| generate_pie_chart(ui, size)
}