use egui::{Color32, Stroke, Pos2, Vec2};

pub struct PieChartSlice {
    pub fraction: f32,
    pub color: Color32,
    pub tooltip: String,
}

fn generate_pie_chart(ui: &mut egui::Ui, size: Vec2, mut slices: Vec<PieChartSlice>) -> egui::Response {
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

        let mut polygon: Vec<Pos2> = Vec::new();

        let target_segments: u32 = 32;
        let mut actual_segments: u32 = 0;

        for slice in &slices {
            actual_segments += (target_segments as f32 * slice.fraction) as u32;
        }


        let mut current_angle: f32 = 0.0;
        for slice in &slices {
            let segment_count: u32 = (target_segments as f32 * slice.fraction) as u32;
            let segment_angle: f32 = (std::f32::consts::PI * 2.0 * slice.fraction) / segment_count as f32;

            for seg in 0..=segment_count {

                let x: f32 = rect.center().x + radius * current_angle.cos();
                let y: f32 = rect.center().y + radius * current_angle.sin();

                polygon.push(egui::pos2(x, y));

                if seg < segment_count {
                    current_angle += segment_angle;
                }
            }

            polygon.push(rect.center());

            //Try Winding number algorithm!?

            fn is_point_in_polygon(point: Pos2, polygon: &[Pos2]) -> bool {
                fn ray_line_intersection(r0: Pos2, r1: Pos2, a: Pos2, b: Pos2) -> Option<Pos2> {
                    let s1: Vec2 = r1 - r0;
                    let s2: Vec2 = b - a;

                    let s = (-s1.y * (r0.x - a.x) + s1.x * (r0.y - a.y)) / (-s2.x * s1.y + s1.x * s2.y);
                    let t = (s2.x * (r0.y - a.y) - s2.y * (r0.x - a.x)) / (-s2.x * s1.y + s1.x * s2.y);

                    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
                        return Some(Pos2 { x: r0.x + (t * s1.x), y: r0.y + (t * s1.y) });
                    }

                    return None;
                }

                let mut intersection_count: u32 = 0;
                for idx in 0..polygon.len() {
                    if let Some(_) = ray_line_intersection(
                        point,
                        Pos2 { x: point.x + 100.0, y: point.y },
                        polygon[idx],
                        polygon[if idx == polygon.len() - 1 {0} else {idx + 1}],
                    ) {
                        intersection_count += 1;
                    }
                }
                intersection_count % 2 == 1
            }
            
            let mut hovered: bool = false;
            ui.input(|i|
                if let Some(pointer_pos) = i.pointer.latest_pos() {
                    hovered = pointer_pos.distance(rect.center()) <= radius && is_point_in_polygon(pointer_pos, &polygon);
                }
            );

            let mut stroke: Stroke = Stroke::NONE;
            if hovered {
                stroke = Stroke::new(2.0, Color32::WHITE);
                egui::show_tooltip(ui.ctx(), egui::Id::new("tooltip"), |ui| {
                    ui.label(format!("{:.2}% {}", slice.fraction * 100.0, &slice.tooltip));
                });
            }

            ui.painter().add(egui::Shape::convex_polygon(
                polygon.clone(),
                slice.color,
                stroke,
            ));
            polygon.clear();
        }
    }

    response
}

pub fn pie_chart<'a>(size: Vec2, slices: Vec<PieChartSlice>) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| generate_pie_chart(ui, size, slices)
}