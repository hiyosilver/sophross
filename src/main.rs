#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod database;
mod datepicker;
mod ingredients;
mod pie_chart;
mod toggle_image;

use ingredients::*;
use pie_chart::{pie_chart, PieChartSlice};
use toggle_image::toggle_image;

use std::collections::HashSet;
use std::rc::Rc;

use eframe::epaint::textures::TextureFilter;
use eframe::{egui, NativeOptions};
use egui::{
    color_picker::{color_edit_button_srgba, Alpha},
    vec2, Align, CentralPanel, Color32, ComboBox, CursorIcon, Direction, Frame, ImageSource,
    Layout, Rounding, SizeHint, Slider, Stroke, Style as BaseStyle, TextureOptions,
    TextureWrapMode, TopBottomPanel, Ui, Vec2, ViewportBuilder, Visuals, WidgetText,
};

use egui_dock::{
    AllowedSplits, DockArea, DockState, NodeIndex, OverlayType, Style, SurfaceIndex,
    TabInteractionStyle, TabViewer,
};

use datepicker::DatePickerButton;
use egui_extras::{Column, TableBuilder};

use rusqlite::{params, Connection};

use crate::database::Database;
use chrono::NaiveDate;

macro_rules! labeled_widget {
    ($ui:expr, $x:expr, $l:expr) => {
        $ui.horizontal(|ui| {
            ui.add($x);
            ui.label($l);
        });
    };
    ($ui:expr, $x:expr, $l:expr, $d:expr) => {
        $ui.horizontal(|ui| {
            ui.add($x).on_hover_text($d);
            ui.label($l).on_hover_text($d);
        });
    };
}

// Creates a slider which has a unit attached to it
// When given an extra parameter it will be used as a multiplier (e.g 100.0 when working with percentages)
macro_rules! unit_slider {
    ($val:expr, $range:expr) => {
        egui::Slider::new($val, $range)
    };
    ($val:expr, $range:expr, $unit:expr) => {
        egui::Slider::new($val, $range).custom_formatter(|value, decimal_range| {
            egui::emath::format_with_decimals_in_range(value, decimal_range) + $unit
        })
    };
    ($val:expr, $range:expr, $unit:expr, $mul:expr) => {
        egui::Slider::new($val, $range)
            .custom_formatter(|value, decimal_range| {
                egui::emath::format_with_decimals_in_range(value * $mul, decimal_range) + $unit
            })
            .custom_parser(|string| string.parse::<f64>().ok().map(|valid| valid / $mul))
    };
}

const CATEGORY_ICON_NAMES: [&str; 6] = ["apple", "bean", "bread", "candy", "drink", "drop"];

fn get_icon_image_source(id: &str) -> ImageSource {
    match id {
        "apple" => egui::include_image!("../icons/categories/apple.png"),
        "bean" => egui::include_image!("../icons/categories/bean.png"),
        "bread" => egui::include_image!("../icons/categories/bread.png"),
        "candy" => egui::include_image!("../icons/categories/candy.png"),
        "drink" => egui::include_image!("../icons/categories/drink.png"),
        "drop" => egui::include_image!("../icons/categories/drop.png"),

        "add" => egui::include_image!("../icons/plus-square.png"),
        "edit" => egui::include_image!("../icons/edit.png"),
        "delete" => egui::include_image!("../icons/delete.png"),

        _ => egui::include_image!("../icons/categories/placeholder.png"),
    }
}

fn main() -> eframe::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            //.with_inner_size(vec2(1200.0, 800.0))
            //.with_min_inner_size(vec2(640.0, 480.0))
            .with_maximized(true),
        ..Default::default()
    };

    let _visuals = Visuals {
        dark_mode: true,
        override_text_color: None,
        widgets: Default::default(),
        selection: Default::default(),
        hyperlink_color: Default::default(),
        faint_bg_color: Default::default(),
        extreme_bg_color: Default::default(),
        code_bg_color: Default::default(),
        warn_fg_color: Default::default(),
        error_fg_color: Default::default(),
        window_rounding: Default::default(),
        window_shadow: Default::default(),
        window_fill: Default::default(),
        window_stroke: Default::default(),
        window_highlight_topmost: false,
        menu_rounding: Default::default(),
        panel_fill: Default::default(),
        popup_shadow: Default::default(),
        resize_corner_size: 0.0,
        text_cursor: Default::default(),
        text_cursor_preview: false,
        clip_rect_margin: 0.0,
        button_frame: false,
        collapsing_header_frame: false,
        indent_has_left_vline: false,
        striped: false,
        slider_trailing_fill: false,
        handle_shape: egui::style::HandleShape::Circle,
        interact_cursor: None,
        image_loading_spinners: false,
        numeric_color_space: egui::style::NumericColorSpace::GammaByte,
    };
    
    let _spacing = egui::style::Spacing {
        item_spacing: Default::default(),
        window_margin: Default::default(),
        button_padding: Default::default(),
        menu_margin: Default::default(),
        indent: 0.0,
        interact_size: Default::default(),
        slider_width: 0.0,
        combo_width: 0.0,
        text_edit_width: 0.0,
        icon_width: 0.0,
        icon_width_inner: 0.0,
        icon_spacing: 0.0,
        tooltip_width: 0.0,
        menu_width: 0.0,
        indent_ends_with_horizontal_line: false,
        combo_height: 0.0,
        scroll: Default::default(),
    };

    eframe::run_native(
        "SophrOSS",
        options,
        Box::new(|creation_context| {
            let style = BaseStyle {
                visuals: Visuals::dark(),
                ..BaseStyle::default()
            };
            creation_context.egui_ctx.set_style(style);
            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    context: MyContext,
    tree: DockState<String>,
}

impl Default for MyApp {
    fn default() -> Self {
        let phi: f32 = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let mut dock_state = DockState::new(vec![
            "Ingredients".to_owned(),
            "Categories".to_owned(),
            "Style Editor".to_owned(),
        ]);
        dock_state.translations.tab_context_menu.eject_button = "Undock".to_owned();
        let [a, b] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            1.0 - (1.0 / phi),
            vec!["Daily Log".to_owned()],
        );
        let [_, _] =
            dock_state
                .main_surface_mut()
                .split_below(a, 1.0 / phi, vec!["Details".to_owned()]);
        let [_, _] =
            dock_state
                .main_surface_mut()
                .split_below(b, 0.5, vec!["Statistics".to_owned()]);

        let mut open_tabs = HashSet::new();

        for node in dock_state[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }

        let context = MyContext {
            style: None,
            open_tabs,

            date: None,

            show_window_close: true,
            show_window_collapse: true,
            show_close_buttons: true,
            show_add_buttons: false,
            draggable_tabs: true,
            show_tab_name_on_hover: false,
            allowed_splits: AllowedSplits::default(),

            database: Database::new(),

            ingredients_list: Vec::new(),
            show_new_ingredient_dialog: false,
            update_ingredients: true,
            selected_ingredient: None,
            selected_ingredient_nutrition_info: None,

            new_ingredient_name: String::from(""),
            new_ingredient_name_was_empty: false,
            new_ingredient_brand: String::from(""),
            new_ingredient_amount: 1.0,
            new_ingredient_unit: Unit::Grams,
            new_ingredient_calories: 0.0,
            new_ingredient_selected_categories: Vec::new(),
            new_ingredient_nutritional_info: None,

            categories_list: Vec::new(),
            show_new_category_dialog: false,
            update_categories: true,
            selected_category: None,

            new_category_name: String::from(""),
            new_category_name_was_empty: false,
            new_category_icon_name: String::from(""),
            new_category_icon_color: Color32::WHITE,
            new_category_selected_icon: None,
            new_category_selected_icon_was_invalid: false,

            log_entry_list: Vec::new(),
            log_entry_dates: HashSet::new(),
            show_new_log_entry_dialog: false,
            update_log_entries: true,
            selected_log_entry: None,
            selected_log_entry_nutrition_info: None,

            new_log_entry_fraction: 1.0,
            new_log_entry_ingredient_search: String::from(""),
            new_log_entry_ingredient_previous_search: String::from(""),
            new_log_entry_filtered_ingredients: Vec::new(),
            new_log_entry_selected_ingredient: None,
        };

        Self {
            context,
            tree: dock_state,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.context
            .date
            .get_or_insert_with(|| chrono::offset::Utc::now().date_naive());

        egui_extras::install_image_loaders(ctx);
        TopBottomPanel::top("egui_dock::MenuBar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    // allow certain tabs to be toggled
                    for tab in &["Style Editor"] {
                        if ui
                            .selectable_label(self.context.open_tabs.contains(*tab), *tab)
                            .clicked()
                        {
                            if let Some(index) = self.tree.find_tab(&tab.to_string()) {
                                self.tree.remove_tab(index);
                                self.context.open_tabs.remove(*tab);
                            } else {
                                self.tree[SurfaceIndex::main()]
                                    .push_to_focused_leaf(tab.to_string());
                            }

                            ui.close_menu();
                        }
                    }
                });
            })
        });
        CentralPanel::default()
            // When displaying a DockArea in another UI, it looks better
            // to set inner margins to 0.
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                let style = {
                    let mut new_style = self
                        .context
                        .style
                        .get_or_insert(Style::from_egui(ui.style()))
                        .clone();
                    new_style.tab_bar.height = 32.0;
                    new_style
                };

                DockArea::new(&mut self.tree)
                    .style(style)
                    .show_close_buttons(self.context.show_close_buttons)
                    .show_add_buttons(self.context.show_add_buttons)
                    .draggable_tabs(self.context.draggable_tabs)
                    .show_tab_name_on_hover(self.context.show_tab_name_on_hover)
                    .allowed_splits(self.context.allowed_splits)
                    .show_window_close_buttons(self.context.show_window_close)
                    .show_window_collapse_buttons(self.context.show_window_collapse)
                    .show_inside(ui, &mut self.context);
            });
    }
}

struct MyContext {
    pub style: Option<Style>,
    open_tabs: HashSet<String>,

    date: Option<NaiveDate>,

    show_close_buttons: bool,
    show_add_buttons: bool,
    draggable_tabs: bool,
    show_tab_name_on_hover: bool,
    allowed_splits: AllowedSplits,
    show_window_close: bool,
    show_window_collapse: bool,

    database: Database,

    ingredients_list: Vec<Rc<Ingredient>>,
    show_new_ingredient_dialog: bool,
    update_ingredients: bool,
    selected_ingredient: Option<usize>,
    selected_ingredient_nutrition_info: Option<usize>,

    new_ingredient_name: String,
    new_ingredient_name_was_empty: bool,
    new_ingredient_brand: String,
    new_ingredient_amount: f32,
    new_ingredient_unit: Unit,
    new_ingredient_calories: f32,
    new_ingredient_selected_categories: Vec<usize>,
    new_ingredient_nutritional_info: Option<NutritionalInfo>,

    categories_list: Vec<Category>,
    show_new_category_dialog: bool,
    update_categories: bool,
    selected_category: Option<usize>,

    new_category_name: String,
    new_category_name_was_empty: bool,
    new_category_icon_name: String,
    new_category_icon_color: Color32,
    new_category_selected_icon: Option<usize>,
    new_category_selected_icon_was_invalid: bool,

    log_entry_list: Vec<LogEntry>,
    log_entry_dates: HashSet<NaiveDate>,
    show_new_log_entry_dialog: bool,
    update_log_entries: bool,
    selected_log_entry: Option<usize>,
    selected_log_entry_nutrition_info: Option<usize>,

    new_log_entry_fraction: f32,
    new_log_entry_ingredient_search: String,
    new_log_entry_ingredient_previous_search: String,
    new_log_entry_filtered_ingredients: Vec<Rc<Ingredient>>,
    new_log_entry_selected_ingredient: Option<Rc<Ingredient>>,
}

impl MyContext {
    fn new_ingredient(&mut self, ui: &mut Ui) {
        macro_rules! create {
            () => {
                if self.new_ingredient_name.len() == 0 {
                    self.new_ingredient_name_was_empty = true;
                } else {
                    self.new_ingredient_nutritional_info
                        .as_mut()
                        .unwrap()
                        .default_amount = self.new_ingredient_amount;
                    self.new_ingredient_nutritional_info
                        .as_mut()
                        .unwrap()
                        .default_unit = self.new_ingredient_unit;
                    self.new_ingredient_nutritional_info
                        .as_mut()
                        .unwrap()
                        .kilocalories = self.new_ingredient_calories;

                    let new_ingredient = Ingredient {
                        id: 0,
                        name: self.new_ingredient_name.clone(),
                        brand: self.new_ingredient_brand.clone(),
                        categories: self
                            .new_ingredient_selected_categories
                            .iter()
                            .map(|n| self.categories_list[*n].clone())
                            .collect(),
                        nutritional_info: vec![self
                            .new_ingredient_nutritional_info
                            .clone()
                            .unwrap()],
                    };

                    let _ = self.database.insert_ingredient(&new_ingredient);

                    self.update_ingredients = true;
                    cancel!();
                }
            };
        }
        macro_rules! clear {
            () => {
                self.new_ingredient_name.clear();
                self.new_ingredient_name_was_empty = false;
                self.new_ingredient_brand.clear();
                self.new_ingredient_amount = 1.0;
                self.new_ingredient_selected_categories.clear();
                self.new_ingredient_calories = 0.0;
            };
        }
        macro_rules! cancel {
            () => {
                clear!();
                self.show_new_ingredient_dialog = false;
            };
        }

        ui.heading("Create new ingredient");
        ui.horizontal(|ui| {
            ui.label("Name: ");
            let result = ui.text_edit_singleline(&mut self.new_ingredient_name);
            if result.changed() && self.new_ingredient_name.len() > 0 {
                self.new_ingredient_name_was_empty = false;
            }
            if self.new_ingredient_name_was_empty {
                ui.colored_label(
                    Color32::from_rgb(192, 32, 16),
                    egui::RichText::new("Name is required!").strong(),
                );
            }
        });
        ui.horizontal(|ui| {
            ui.label("Brand: ");
            let _ = ui.text_edit_singleline(&mut self.new_ingredient_brand);
        });
        ui.horizontal(|ui| {
            ui.label("Amount: ");
            ui.add(egui::DragValue::new(&mut self.new_ingredient_amount).clamp_range(0..=9999));
            ComboBox::from_label("Default unit")
                .selected_text(self.new_ingredient_unit.to_string())
                .show_ui(ui, |ui| {
                    for unit in [
                        Unit::Grams,
                        Unit::Teaspoons,
                        Unit::Tablespoons,
                        Unit::Pieces,
                    ] {
                        ui.selectable_value(&mut self.new_ingredient_unit, unit, unit.to_string());
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Calories: ");
            ui.add(egui::DragValue::new(&mut self.new_ingredient_calories).clamp_range(0..=9999));
        });
        ui.horizontal(|ui| {
            egui::Grid::new("category_icon_grid")
                .spacing(vec2(-4.0, 0.0))
                .show(ui, |ui| {
                    for idx in 0..self.categories_list.len() {
                        let category = &self.categories_list[idx];
                        let category_selected =
                            self.new_ingredient_selected_categories.contains(&idx);

                        if ui
                            .add(toggle_image::toggle_image(
                                category_selected,
                                true,
                                &get_icon_image_source(&category.icon_name),
                                category.icon_color,
                                vec2(16.0, 16.0),
                            ))
                            .on_hover_text_at_pointer(category.name.clone())
                            .changed()
                        {
                            if category_selected {
                                let index = self
                                    .new_ingredient_selected_categories
                                    .iter()
                                    .position(|x| *x == idx)
                                    .unwrap();
                                self.new_ingredient_selected_categories.remove(index);
                            } else {
                                self.new_ingredient_selected_categories.push(idx);
                            }
                        }
                    }
                });
            if self.categories_list.len() == 0 {
                ui.colored_label(
                    Color32::from_rgb(192, 192, 16),
                    egui::RichText::new("No categories available."),
                );
            } else if self.new_ingredient_selected_categories.len() == 0 {
                ui.colored_label(
                    Color32::from_rgb(192, 192, 16),
                    egui::RichText::new("Consider adding a category."),
                );
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                create!();
            };
            if ui.button("Clear").clicked() {
                clear!();
            };
            if ui.button("Cancel").clicked() {
                cancel!();
            };
        });
    }

    fn ingredients_view(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.add_enabled_ui(!self.show_new_ingredient_dialog, |ui| {
                if ui
                    .add(
                        egui::Button::image_and_text(
                            egui::Image::new(get_icon_image_source("add"))
                                .tint(Color32::GRAY)
                                .fit_to_exact_size(vec2(16.0, 16.0))
                                .texture_options(TextureOptions {
                                    magnification: TextureFilter::Linear,
                                    minification: TextureFilter::Linear,
                                    wrap_mode: TextureWrapMode::ClampToEdge,
                                }),
                            "New ingredient",
                        )
                        .min_size(vec2(0.0, 24.0)),
                    )
                    .clicked()
                {
                    self.show_new_ingredient_dialog = true;
                    self.new_ingredient_nutritional_info = Some(NutritionalInfo {
                        default_amount: 1.0,
                        default_unit: Unit::Grams,
                        kilocalories: 0.0,
                        macronutrients: Macronutrients {
                            proteins: Proteins {
                                essential_amino_acids: EssentialAminoAcids {
                                    histidine: 0.0,
                                    isoleucine: 0.0,
                                    leucine: 0.0,
                                    lysine: 0.0,
                                    methionine: 0.0,
                                    phenylalanine: 0.0,
                                    threonine: 0.0,
                                    tryptophan: 0.0,
                                    valine: 0.0,
                                },
                                non_essential_amino_acids: NonEssentialAminoAcids {
                                    alanine: 0.0,
                                    arginine: 0.0,
                                    asparagine: 0.0,
                                    aspartic_acid: 0.0,
                                    cysteine: 0.0,
                                    glutamic_acid: 0.0,
                                    glutamine: 0.0,
                                    glycine: 0.0,
                                    proline: 0.0,
                                    serine: 0.0,
                                    tyrosine: 0.0,
                                },
                            },
                            fats: Fats {
                                saturated: 0.0,
                                monounsaturated: 0.0,
                                polyunsaturated: 0.0,
                            },
                            carbohydrates: Carbohydrates {
                                starch: 0.0,
                                fiber: 0.0,
                                sugars: 0.0,
                                sugar_alcohols: 0.0,
                            },
                        },
                        micronutrients: Micronutrients {
                            vitamins: Vitamins {
                                vitamin_a: 0.0,
                                vitamin_b1: 0.0,
                                vitamin_b2: 0.0,
                                vitamin_b3: 0.0,
                                vitamin_b5: 0.0,
                                vitamin_b6: 0.0,
                                vitamin_b9: 0.0,
                                vitamin_b12: 0.0,
                                vitamin_c: 0.0,
                                vitamin_d: 0.0,
                                vitamin_e: 0.0,
                                vitamin_k: 0.0,
                                betaine: 0.0,
                                choline: 0.0,
                            },
                            minerals: Minerals {
                                calcium: 0.0,
                                copper: 0.0,
                                iron: 0.0,
                                magnesium: 0.0,
                                manganese: 0.0,
                                phosphorus: 0.0,
                                potassium: 0.0,
                                selenium: 0.0,
                                sodium: 0.0,
                                zinc: 0.0,
                            },
                        },
                    });
                }
            });
            ui.label(format!(
                "{} {}",
                self.ingredients_list.len(),
                if self.ingredients_list.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            ));
        });
        if self.show_new_ingredient_dialog {
            self.new_ingredient(ui);
        }
        ui.separator();
        TableBuilder::new(ui)
            .sense(egui::Sense::click())
            .striped(true)
            .cell_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                    .with_main_align(egui::Align::LEFT),
            )
            .column(Column::auto().resizable(true))
            .columns(Column::remainder().resizable(true), 3)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Categories");
                });
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Brand");
                });
                header.col(|ui| {
                    ui.heading("Calories");
                });
            })
            .body(|body| {
                body.rows(30.0, self.ingredients_list.len(), |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selected_ingredient.is_some_and(|idx| idx == row_index));

                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            for category in &self.ingredients_list[row_index].categories {
                                let icon_name = &category.icon_name;

                                let response = ui.add(
                                    egui::Image::new(get_icon_image_source(icon_name).clone())
                                        .tint(category.icon_color)
                                        .fit_to_exact_size(vec2(16.0, 16.0))
                                        .texture_options(TextureOptions {
                                            magnification: TextureFilter::Nearest,
                                            minification: TextureFilter::Nearest,
                                            wrap_mode: TextureWrapMode::ClampToEdge,
                                        }),
                                );

                                ui.add_space(-4.0);

                                //Workaround to allow for both clicking rows and showing tooltip
                                if ui.rect_contains_pointer(response.rect) {
                                    egui::show_tooltip(
                                        ui.ctx(),
                                        egui::Id::new("category_tooltip"),
                                        |ui| {
                                            ui.label(category.name.clone());
                                        },
                                    );
                                }
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.label(&self.ingredients_list[row_index].name);
                    });
                    row.col(|ui| {
                        ui.label(egui::RichText::new(&self.ingredients_list[row_index].brand).italics());
                    });
                    row.col(|ui| {
                        ui.label(
                            &self.ingredients_list[row_index].nutritional_info
                                [self.selected_ingredient_nutrition_info.unwrap_or(0)]
                            .kilocalories
                            .to_string(),
                        );
                    });

                    if row.response().clicked() {
                        self.selected_ingredient = Some(row_index);
                        self.selected_ingredient_nutrition_info = Some(0);
                    }
                });
            });
    }

    fn new_category(&mut self, ui: &mut Ui) {
        macro_rules! create_category {
            () => {
                let valid_name = self.new_category_name.len() > 0;
                let valid_icon = self.new_category_selected_icon.is_some();
                if !valid_name {
                    self.new_category_name_was_empty = true;
                }
                if !valid_icon {
                    self.new_category_selected_icon_was_invalid = true;
                }
                if valid_name && valid_icon {
                    let new_category = Category {
                        id: 0,
                        name: self.new_category_name.clone(),
                        icon_name: self.new_category_icon_name.clone(),
                        icon_color: self.new_category_icon_color.clone(),
                    };

                    let _ = self.database.insert_category(&new_category);

                    self.update_categories = true;
                    cancel_category!();
                }
            };
        }
        macro_rules! clear_category {
            () => {
                self.new_category_name.clear();
                self.new_category_name_was_empty = false;
                self.new_category_icon_color = Color32::WHITE;
                self.new_category_icon_name = String::from("");
                self.new_category_selected_icon = None;
                self.new_category_selected_icon_was_invalid = false;
            };
        }
        macro_rules! cancel_category {
            () => {
                clear_category!();
                self.show_new_category_dialog = false;
            };
        }

        ui.heading("Create new category");
        ui.horizontal(|ui| {
            ui.label("Name: ");
            let name_result = ui.text_edit_singleline(&mut self.new_category_name);
            if name_result.changed() && self.new_category_name.len() > 0 {
                self.new_category_name_was_empty = false;
            }
            if self.new_category_name_was_empty {
                ui.colored_label(Color32::from_rgb(192, 32, 16), "Name is required!");
            }
        });
        ui.horizontal(|ui| {
            ui.color_edit_button_srgba(&mut self.new_category_icon_color);
            egui::Grid::new("category_icon_grid")
                .spacing(vec2(-4.0, 0.0))
                .show(ui, |ui| {
                    for idx in 0..CATEGORY_ICON_NAMES.len() {
                        let active: bool = {
                            if let Some(index) = self.new_category_selected_icon {
                                idx == index
                            } else {
                                false
                            }
                        };
                        if ui
                            .add(toggle_image::toggle_image(
                                active,
                                false,
                                &get_icon_image_source(CATEGORY_ICON_NAMES[idx]),
                                self.new_category_icon_color,
                                vec2(16.0, 16.0),
                            ))
                            .changed()
                        {
                            self.new_category_selected_icon = Some(idx);
                            self.new_category_icon_name = CATEGORY_ICON_NAMES[idx].to_owned();
                        }
                    }
                });
            if let Some(_) = self.new_category_selected_icon {
                self.new_category_selected_icon_was_invalid = false;
            }
            if self.new_category_selected_icon_was_invalid {
                ui.colored_label(Color32::from_rgb(192, 32, 16), "Category icon is required!");
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                create_category!();
            };
            if ui.button("Clear").clicked() {
                clear_category!();
            };
            if ui.button("Cancel").clicked() {
                cancel_category!();
            };
        });
    }

    fn categories_view(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.add_enabled_ui(!self.show_new_category_dialog, |ui| {
                if ui
                    .add(
                        egui::Button::image_and_text(
                            egui::Image::new(get_icon_image_source("add"))
                                .tint(Color32::GRAY)
                                .fit_to_exact_size(vec2(16.0, 16.0))
                                .texture_options(TextureOptions {
                                    magnification: TextureFilter::Nearest,
                                    minification: TextureFilter::Nearest,
                                    wrap_mode: TextureWrapMode::ClampToEdge,
                                }),
                            "New category",
                        )
                        .min_size(vec2(0.0, 24.0)),
                    )
                    .clicked()
                {
                    self.show_new_category_dialog = true;
                }
            });
            ui.label(format!(
                "{} {}",
                self.categories_list.len(),
                if self.categories_list.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            ));
        });
        if self.show_new_category_dialog {
            self.new_category(ui);
        }
        ui.separator();
        TableBuilder::new(ui)
            .sense(egui::Sense::click())
            .striped(true)
            .cell_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                    .with_main_align(egui::Align::LEFT),
            )
            .column(Column::auto().resizable(true))
            .columns(Column::remainder(), 2)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Icon");
                });
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Used #");
                });
            })
            .body(|body| {
                body.rows(30.0, self.categories_list.len(), |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selected_category.is_some_and(|idx| idx == row_index));

                    row.col(|ui| {
                        ui.add(
                            egui::Image::new(
                                get_icon_image_source(&self.categories_list[row_index].icon_name)
                                    .clone(),
                            )
                            .tint(self.categories_list[row_index].icon_color)
                            .fit_to_exact_size(vec2(16.0, 16.0))
                            .texture_options(TextureOptions {
                                magnification: TextureFilter::Nearest,
                                minification: TextureFilter::Nearest,
                                wrap_mode: TextureWrapMode::ClampToEdge,
                            }),
                        );
                    });
                    row.col(|ui| {
                        ui.label(&self.categories_list[row_index].name);
                    });

                    if row.response().clicked() {
                        self.selected_category = Some(row_index);
                    }
                });
            });
    }

    fn details_view(&mut self, ui: &mut Ui) {
        fn nutritional_info_view(ui: &mut Ui, ingredient: &Ingredient, index: usize) {
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.label(format!(
                        "per {}{}:",
                        ingredient.nutritional_info[index].default_amount,
                        ingredient.nutritional_info[index].default_unit
                    ));
                    ui.label(format!(
                        "Calories: {}",
                        ingredient.nutritional_info[index].kilocalories
                    ));
                    ui.collapsing("Macronutrients", |ui| {
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::left_to_right(Align::Center),
                            |ui| {
                                ui.label(format!(
                                    "Protein: {}\nFat: {}\nCarbohydrates (net): {}",
                                    ingredient.nutritional_info[index]
                                        .macronutrients
                                        .proteins
                                        .total_proteins(),
                                    ingredient.nutritional_info[index]
                                        .macronutrients
                                        .fats
                                        .total_fats(),
                                    ingredient.nutritional_info[index]
                                        .macronutrients
                                        .carbohydrates
                                        .net_carbs()
                                ));
                                ui.add(pie_chart::pie_chart(
                                    vec2(4.0, 4.0),
                                    vec![
                                        PieChartSlice {
                                            fraction: 1.0 / 3.0,
                                            color: Color32::LIGHT_GREEN,
                                            tooltip: "Protein".to_owned(),
                                        },
                                        PieChartSlice {
                                            fraction: 1.0 / 3.0,
                                            color: Color32::LIGHT_BLUE,
                                            tooltip: "Fat".to_owned(),
                                        },
                                        PieChartSlice {
                                            fraction: 1.0 / 3.0,
                                            color: Color32::LIGHT_RED,
                                            tooltip: "Carbohydrates".to_owned(),
                                        },
                                    ],
                                ));
                            },
                        );
                    });
                    ui.collapsing("Micronutrients", |ui| {
                        ui.collapsing("Vitamins", |ui| {
                            ui.label(format!(
                                "Vitamin A: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_a
                            ));
                            ui.label(format!(
                                "Vitamin B1: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b1
                            ));
                            ui.label(format!(
                                "Vitamin B2: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b2
                            ));
                            ui.label(format!(
                                "Vitamin B3: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b3
                            ));
                            ui.label(format!(
                                "Vitamin B5: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b5
                            ));
                            ui.label(format!(
                                "Vitamin B6: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b6
                            ));
                            ui.label(format!(
                                "Vitamin B9: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b9
                            ));
                            ui.label(format!(
                                "Vitamin B12: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_b12
                            ));
                            ui.label(format!(
                                "Vitamin C: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_c
                            ));
                            ui.label(format!(
                                "Vitamin D: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_d
                            ));
                            ui.label(format!(
                                "Vitamin E: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_e
                            ));
                            ui.label(format!(
                                "Vitamin K: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .vitamin_k
                            ));
                            ui.label(format!(
                                "Betaine: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .betaine
                            ));
                            ui.label(format!(
                                "Choline: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .vitamins
                                    .choline
                            ));
                        });
                        ui.collapsing("Minerals", |ui| {
                            ui.label(format!(
                                "Calcium: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .calcium
                            ));
                            ui.label(format!(
                                "Copper: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .copper
                            ));
                            ui.label(format!(
                                "Iron: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .iron
                            ));
                            ui.label(format!(
                                "Magnesium: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .magnesium
                            ));
                            ui.label(format!(
                                "Manganese: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .manganese
                            ));
                            ui.label(format!(
                                "Phosphorus: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .phosphorus
                            ));
                            ui.label(format!(
                                "Potassium: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .potassium
                            ));
                            ui.label(format!(
                                "Selenium: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .selenium
                            ));
                            ui.label(format!(
                                "Sodium: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .sodium
                            ));
                            ui.label(format!(
                                "Zinc: {}",
                                ingredient.nutritional_info[index]
                                    .micronutrients
                                    .minerals
                                    .zinc
                            ));
                        });
                    });
                });
        }

        if let Some(idx) = self.selected_ingredient {
            let ingredient = &self.ingredients_list[idx];
            ui.horizontal(|ui| {
                for category in &ingredient.categories {
                    let icon_name = &category.icon_name;

                    ui.add(
                        egui::Image::new(get_icon_image_source(icon_name).clone())
                            .tint(category.icon_color)
                            .fit_to_exact_size(vec2(16.0, 16.0))
                            .texture_options(TextureOptions {
                                magnification: TextureFilter::Nearest,
                                minification: TextureFilter::Nearest,
                                wrap_mode: TextureWrapMode::ClampToEdge,
                            }),
                    )
                    .on_hover_text(category.name.clone());

                    ui.add_space(-4.0);
                }
                ui.label(egui::RichText::new(&ingredient.name).heading().underline());
                ui.label(egui::RichText::new(&ingredient.brand).italics());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui
                        .add(
                            egui::Button::image_and_text(
                                egui::Image::new(get_icon_image_source("delete"))
                                    .tint(Color32::RED)
                                    .fit_to_exact_size(vec2(16.0, 16.0))
                                    .texture_options(TextureOptions {
                                        magnification: TextureFilter::Nearest,
                                        minification: TextureFilter::Nearest,
                                        wrap_mode: TextureWrapMode::ClampToEdge,
                                    }),
                                "Delete",
                            )
                            .min_size(vec2(0.0, 24.0)),
                        )
                        .clicked()
                    {
                        let delete_result = self.database.delete_ingredient(&ingredient);
                        if let Ok(_) = delete_result {
                            self.update_ingredients = true;
                            if self.ingredients_list.len() == 1 {
                                self.selected_ingredient = None;
                                self.selected_ingredient_nutrition_info = None;
                            } else if let Some(selected) = self.selected_ingredient {
                                if selected == self.ingredients_list.len() - 1 {
                                    self.selected_ingredient = Some(selected - 1);
                                    self.selected_ingredient_nutrition_info = Some(0);
                                }
                            }
                        }
                    }
                    if ui
                        .add(
                            egui::Button::image_and_text(
                                egui::Image::new(get_icon_image_source("edit"))
                                    .tint(Color32::GRAY)
                                    .fit_to_exact_size(vec2(16.0, 16.0))
                                    .texture_options(TextureOptions {
                                        magnification: TextureFilter::Nearest,
                                        minification: TextureFilter::Nearest,
                                        wrap_mode: TextureWrapMode::ClampToEdge,
                                    }),
                                "Edit",
                            )
                            .min_size(vec2(0.0, 24.0)),
                        )
                        .clicked()
                    {
                        println!("Edit ingredient button pressed!");
                    }
                });
            });
            nutritional_info_view(
                ui,
                ingredient,
                self.selected_ingredient_nutrition_info.unwrap_or(0),
            );
        } else if let Some(idx) = self.selected_category {
            let category = &self.categories_list[idx];

            ui.horizontal(|ui| {
                ui.add_space(-4.0);
                ui.add(
                    egui::Image::new(get_icon_image_source(&category.icon_name).clone())
                        .tint(category.icon_color)
                        .fit_to_exact_size(vec2(16.0, 16.0))
                        .texture_options(TextureOptions {
                            magnification: TextureFilter::Nearest,
                            minification: TextureFilter::Nearest,
                            wrap_mode: TextureWrapMode::ClampToEdge,
                        }),
                )
                .on_hover_text(category.name.clone());

                ui.heading(&category.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui
                        .add(
                            egui::Button::image_and_text(
                                egui::Image::new(get_icon_image_source("delete"))
                                    .tint(Color32::RED)
                                    .fit_to_exact_size(vec2(16.0, 16.0))
                                    .texture_options(TextureOptions {
                                        magnification: TextureFilter::Nearest,
                                        minification: TextureFilter::Nearest,
                                        wrap_mode: TextureWrapMode::ClampToEdge,
                                    }),
                                "Delete",
                            )
                            .min_size(vec2(0.0, 24.0)),
                        )
                        .clicked()
                    {
                        let delete_result = self.database.delete_category(&category);
                        if let Ok(result) = delete_result {
                            self.update_categories = true;
                            if result > 0 {
                                self.update_ingredients = true;
                            }
                            if self.categories_list.len() == 1 {
                                self.selected_category = None;
                            } else if let Some(selected) = self.selected_category {
                                if selected == self.categories_list.len() - 1 {
                                    self.selected_category = Some(selected - 1);
                                }
                            }
                        }
                    }
                });
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("-nothing selected-");
            });
        }
    }

    fn new_log_entry(&mut self, ui: &mut Ui) {
        macro_rules! create_log_entry {
            () => {
                if let Some(ingredient) = &self.new_log_entry_selected_ingredient {
                    let log_entry = LogEntry {
                        id: 0,
                        ingredient: ingredient.clone(),
                        fraction: self.new_log_entry_fraction,
                    };

                    let _ = self.database.insert_log_entry(&self.date.unwrap(), &log_entry);

                    self.update_log_entries = true;
                    cancel_log_entry!();
                }
            };
        }
        macro_rules! clear_log_entry {
            () => {
                self.new_log_entry_fraction = 1.0;
                self.new_log_entry_ingredient_search = String::from("");
                self.new_log_entry_ingredient_previous_search = String::from("");
                self.new_log_entry_filtered_ingredients.clear();
                self.new_log_entry_selected_ingredient = None;
            };
        }
        macro_rules! cancel_log_entry {
            () => {
                clear_log_entry!();
                self.show_new_log_entry_dialog = false;
            };
        }

        ui.heading("Create new log entry");
        ui.horizontal(|ui| {
            ui.label("Fraction: ");
            ui.add(
                egui::DragValue::new(&mut self.new_log_entry_fraction)
                    .clamp_range(0..=9999)
                    .speed(0.1),
            );
        });
        if  self.new_log_entry_ingredient_search.len() > 1 &&
            !self.new_log_entry_ingredient_search.eq_ignore_ascii_case(
                self.new_log_entry_ingredient_previous_search.as_str(),
            )
        {
            self.new_log_entry_ingredient_previous_search =
                self.new_log_entry_ingredient_search.clone();

            self.new_log_entry_filtered_ingredients = self
                .ingredients_list
                .iter()
                .filter(|&n| n.name.to_ascii_lowercase().contains(&self.new_log_entry_ingredient_search.to_ascii_lowercase()))
                .map(|x| x.clone())
                .collect();
        }
        ui.horizontal(|ui| {
            ui.label("Ingredient: ");
            ComboBox::from_id_source("new_log_ingredient")
                .width(128.0)
                .selected_text(if let Some(ingredient) = &self.new_log_entry_selected_ingredient {
                    ingredient.name.clone()
                } else {
                    "-".to_owned()
                })
                .show_ui(ui, |ui| {
                    let text_edit =
                        egui::TextEdit::singleline(&mut self.new_log_entry_ingredient_search)
                            .lock_focus(true);
                    ui.add(text_edit).request_focus();

                    for ingredient in &self.new_log_entry_filtered_ingredients {
                        ui.horizontal(|ui| {
                            for category in &ingredient.categories {
                                ui.add(egui::Image::new(get_icon_image_source(&category.icon_name))
                                    .tint(category.icon_color)
                                    .fit_to_exact_size(vec2(16.0, 16.0))
                                    .texture_options(TextureOptions {
                                        magnification: TextureFilter::Nearest,
                                        minification: TextureFilter::Nearest,
                                        wrap_mode: TextureWrapMode::ClampToEdge,
                                    }));
                            }
                            ui.selectable_value(
                                &mut self.new_log_entry_selected_ingredient,
                                Some(ingredient.clone()),
                                ingredient.name.clone(),
                            );
                        });
                    }
                });
        });
        ui.horizontal(|ui| {
            if ui.button("Create").clicked() {
                create_log_entry!();
            };
            if ui.button("Clear").clicked() {
                clear_log_entry!();
            };
            if ui.button("Cancel").clicked() {
                cancel_log_entry!();
            };
        });
    }

    fn daily_log_view(&mut self, ui: &mut Ui) {
        let date = self
            .date
            .get_or_insert_with(|| chrono::offset::Utc::now().date_naive());

        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            if ui.add(
                DatePickerButton::new(date)
                    .highlight_weekends(false)
                    .min_size(vec2(0.0, 24.0))
                    .with_data(&self.log_entry_dates)
            ).changed() {
                self.update_log_entries = true;
            };
            ui.add_enabled_ui(!self.show_new_log_entry_dialog, |ui| {
                if ui
                    .add(
                        egui::Button::image_and_text(
                            egui::Image::new(get_icon_image_source("add"))
                                .tint(Color32::GRAY)
                                .fit_to_exact_size(vec2(16.0, 16.0))
                                .texture_options(TextureOptions {
                                    magnification: TextureFilter::Nearest,
                                    minification: TextureFilter::Nearest,
                                    wrap_mode: TextureWrapMode::Repeat,
                                }),
                            "New log entry",
                        )
                        .min_size(vec2(0.0, 24.0)),
                    )
                    .clicked()
                {
                    self.show_new_log_entry_dialog = true;
                }
            });
            ui.label(format!(
                "{} {}",
                self.log_entry_list.len(),
                if self.log_entry_list.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            ));
        });
        if self.show_new_log_entry_dialog {
            self.new_log_entry(ui);
        }
        ui.separator();
        TableBuilder::new(ui)
            .sense(egui::Sense::click())
            .striped(true)
            .cell_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                    .with_main_align(egui::Align::LEFT),
            )
            .column(Column::auto().resizable(true))
            .columns(Column::remainder().resizable(true), 2)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Amount");
                });
                header.col(|ui| {
                    ui.heading("Calories");
                });
            })
            .body(|body| {
                body.rows(30.0, self.log_entry_list.len(), |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selected_log_entry.is_some_and(|idx| idx == row_index));

                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            for category in &self.log_entry_list[row_index].ingredient.categories {
                                let icon_name = &category.icon_name;

                                let response = ui.add(
                                    egui::Image::new(get_icon_image_source(icon_name).clone())
                                        .tint(category.icon_color)
                                        .fit_to_exact_size(vec2(16.0, 16.0))
                                        .texture_options(TextureOptions {
                                            magnification: TextureFilter::Nearest,
                                            minification: TextureFilter::Nearest,
                                            wrap_mode: TextureWrapMode::ClampToEdge,
                                        }),
                                );

                                ui.add_space(-4.0);

                                //Workaround to allow for both clicking rows and showing tooltip
                                if ui.rect_contains_pointer(response.rect) {
                                    egui::show_tooltip(
                                        ui.ctx(),
                                        egui::Id::new("category_tooltip"),
                                        |ui| {
                                            ui.label(category.name.clone());
                                        },
                                    );
                                }
                            }
                            ui.label(&self.log_entry_list[row_index].ingredient.name);
                        });
                    });
                    row.col(|ui| {
                        ui.label(
                            format!("{} x {} {}",
                                    &self.log_entry_list[row_index].fraction.to_string(),
                                    &self.log_entry_list[row_index].ingredient.nutritional_info[0]
                                        .default_amount
                                        .to_string(),
                                    &self.log_entry_list[row_index].ingredient.nutritional_info[0].default_unit.to_string())
                        );
                    });

                    row.col(|ui| {
                        ui.label(
                            &self.log_entry_list[row_index]
                                .calculate_calories(self.selected_log_entry_nutrition_info.unwrap_or(0))
                                .to_string(),
                        );
                    });

                    if row.response().clicked() {
                        self.selected_log_entry = Some(row_index);
                        self.selected_ingredient_nutrition_info = Some(0);
                    }
                });
            });
    }

    fn style_editor(&mut self, ui: &mut Ui) {
        fn rounding_ui(ui: &mut Ui, rounding: &mut Rounding) {
            labeled_widget!(ui, Slider::new(&mut rounding.nw, 0.0..=15.0), "North-West");
            labeled_widget!(ui, Slider::new(&mut rounding.ne, 0.0..=15.0), "North-East");
            labeled_widget!(ui, Slider::new(&mut rounding.sw, 0.0..=15.0), "South-West");
            labeled_widget!(ui, Slider::new(&mut rounding.se, 0.0..=15.0), "South-East");
        }

        ui.heading("Style Editor");

        ui.collapsing("DockArea Options", |ui| {
            ui.checkbox(&mut self.show_close_buttons, "Show close buttons");
            ui.checkbox(&mut self.show_add_buttons, "Show add buttons");
            ui.checkbox(&mut self.draggable_tabs, "Draggable tabs");
            ui.checkbox(&mut self.show_tab_name_on_hover, "Show tab name on hover");
            ui.checkbox(&mut self.show_window_close, "Show close button on windows");
            ui.checkbox(
                &mut self.show_window_collapse,
                "Show collaspse button on windows",
            );
            ComboBox::new("cbox:allowed_splits", "Split direction(s)")
                .selected_text(format!("{:?}", self.allowed_splits))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.allowed_splits, AllowedSplits::All, "All");
                    ui.selectable_value(
                        &mut self.allowed_splits,
                        AllowedSplits::LeftRightOnly,
                        "LeftRightOnly",
                    );
                    ui.selectable_value(
                        &mut self.allowed_splits,
                        AllowedSplits::TopBottomOnly,
                        "TopBottomOnly",
                    );
                    ui.selectable_value(&mut self.allowed_splits, AllowedSplits::None, "None");
                });
        });

        let style = self.style.as_mut().unwrap();

        ui.collapsing("Border", |ui| {
            egui::Grid::new("border").show(ui, |ui| {
                ui.label("Width:");
                ui.add(Slider::new(
                    &mut style.main_surface_border_stroke.width,
                    1.0..=50.0,
                ));
                ui.end_row();

                ui.label("Color:");
                color_edit_button_srgba(
                    ui,
                    &mut style.main_surface_border_stroke.color,
                    Alpha::OnlyBlend,
                );
                ui.end_row();
            });
        });

        ui.collapsing("Separator", |ui| {
            egui::Grid::new("separator").show(ui, |ui| {
                ui.label("Width:");
                ui.add(Slider::new(&mut style.separator.width, 1.0..=50.0));
                ui.end_row();

                ui.label("Extra Interact Width:");
                ui.add(Slider::new(
                    &mut style.separator.extra_interact_width,
                    0.0..=50.0,
                ));
                ui.end_row();

                ui.label("Offset limit:");
                ui.add(Slider::new(&mut style.separator.extra, 1.0..=300.0));
                ui.end_row();

                ui.label("Idle color:");
                color_edit_button_srgba(ui, &mut style.separator.color_idle, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Hovered color:");
                color_edit_button_srgba(ui, &mut style.separator.color_hovered, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Dragged color:");
                color_edit_button_srgba(ui, &mut style.separator.color_dragged, Alpha::OnlyBlend);
                ui.end_row();
            });
        });

        ui.collapsing("Tabs", |ui| {
            ui.separator();

            ui.checkbox(&mut style.tab_bar.fill_tab_bar, "Expand tabs");
            ui.checkbox(
                &mut style.tab_bar.show_scroll_bar_on_overflow,
                "Show scroll bar on tab overflow",
            );
            ui.checkbox(
                &mut style.tab.hline_below_active_tab_name,
                "Show a line below the active tab name",
            );
            ui.horizontal(|ui| {
                ui.add(Slider::new(&mut style.tab_bar.height, 20.0..=50.0));
                ui.label("Tab bar height");
            });

            ComboBox::new("add_button_align", "Add button align")
                .selected_text(format!("{:?}", style.buttons.add_tab_align))
                .show_ui(ui, |ui| {
                    for align in [egui_dock::TabAddAlign::Left, egui_dock::TabAddAlign::Right] {
                        ui.selectable_value(
                            &mut style.buttons.add_tab_align,
                            align,
                            format!("{:?}", align),
                        );
                    }
                });

            ui.separator();

            fn tab_style_editor_ui(ui: &mut Ui, tab_style: &mut TabInteractionStyle) {
                ui.separator();

                ui.label("Rounding");
                rounding_ui(ui, &mut tab_style.rounding);

                ui.separator();

                egui::Grid::new("tabs_colors").show(ui, |ui| {
                    ui.label("Title text color:");
                    color_edit_button_srgba(ui, &mut tab_style.text_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Outline color:")
                        .on_hover_text("The outline around the active tab name.");
                    color_edit_button_srgba(ui, &mut tab_style.outline_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Background color:");
                    color_edit_button_srgba(ui, &mut tab_style.bg_fill, Alpha::OnlyBlend);
                    ui.end_row();
                });
            }

            ui.collapsing("Active", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.active);
            });

            ui.collapsing("Inactive", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.inactive);
            });

            ui.collapsing("Focused", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.focused);
            });

            ui.collapsing("Hovered", |ui| {
                tab_style_editor_ui(ui, &mut style.tab.hovered);
            });

            ui.separator();

            egui::Grid::new("tabs_colors").show(ui, |ui| {
                ui.label("Close button color unfocused:");
                color_edit_button_srgba(ui, &mut style.buttons.close_tab_color, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Close button color focused:");
                color_edit_button_srgba(
                    ui,
                    &mut style.buttons.close_tab_active_color,
                    Alpha::OnlyBlend,
                );
                ui.end_row();

                ui.label("Close button background color:");
                color_edit_button_srgba(ui, &mut style.buttons.close_tab_bg_fill, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Bar background color:");
                color_edit_button_srgba(ui, &mut style.tab_bar.bg_fill, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Horizontal line color:").on_hover_text(
                    "The line separating the tab name area from the tab content area",
                );
                color_edit_button_srgba(ui, &mut style.tab_bar.hline_color, Alpha::OnlyBlend);
                ui.end_row();
            });
        });

        ui.collapsing("Tab body", |ui| {
            ui.separator();

            ui.label("Rounding");
            rounding_ui(ui, &mut style.tab.tab_body.rounding);

            ui.label("Stroke width:");
            ui.add(Slider::new(
                &mut style.tab.tab_body.stroke.width,
                0.0..=10.0,
            ));
            ui.end_row();

            egui::Grid::new("tab_body_colors").show(ui, |ui| {
                ui.label("Stroke color:");
                color_edit_button_srgba(ui, &mut style.tab.tab_body.stroke.color, Alpha::OnlyBlend);
                ui.end_row();

                ui.label("Background color:");
                color_edit_button_srgba(ui, &mut style.tab.tab_body.bg_fill, Alpha::OnlyBlend);
                ui.end_row();
            });
        });
        ui.collapsing("Overlay", |ui| {
            let selected_text = match style.overlay.overlay_type {
                OverlayType::HighlightedAreas => "Highlighted Areas",
                OverlayType::Widgets => "Widgets",
            };
            ui.label("Overlay Style:");
            ComboBox::new("overlay styles", "")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut style.overlay.overlay_type,
                        OverlayType::HighlightedAreas,
                        "Highlighted Areas",
                    );
                    ui.selectable_value(
                        &mut style.overlay.overlay_type,
                        OverlayType::Widgets,
                        "Widgets",
                    );
                });
            ui.collapsing("Feel", |ui|{
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.center_drop_coverage, 0.0..=1.0, "%", 100.0),
                    "Center drop coverage",
                    "how big the area where dropping a tab into the center of another should be."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.fade_hold_time, 0.0..=4.0, "s"),
                    "Fade hold time",
                    "How long faded windows should hold their fade before unfading, in seconds."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.max_preference_time, 0.0..=4.0, "s"),
                    "Max preference time",
                    "How long the overlay may prefer to stick to a surface despite hovering over another, in seconds."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.window_drop_coverage, 0.0..=1.0, "%", 100.0),
                    "Window drop coverage",
                    "How big the area for undocking a window should be. [is overshadowed by center drop coverage]"
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.feel.interact_expansion, 1.0..=100.0, "ps"),
                    "Interact expansion",
                    "How much extra interaction area should be allocated for buttons on the overlay"
                );
            });

            ui.collapsing("Visuals", |ui|{
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.max_button_size, 10.0..=500.0, "ps"),
                    "Max button size",
                    "The max length of a side on a overlay button in egui points"
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.button_spacing, 0.0..=50.0, "ps"),
                    "Button spacing",
                    "Spacing between buttons on the overlay, in egui units."
                );
                labeled_widget!(
                    ui,
                    unit_slider!(&mut style.overlay.surface_fade_opacity, 0.0..=1.0, "%", 100.0),
                    "Window fade opacity",
                    "how visible windows are when dragging a tab behind them."
                );
                labeled_widget!(
                    ui,
                    egui::Slider::new(&mut style.overlay.selection_stroke_width, 0.0..=50.0),
                    "Selection stroke width",
                    "width of a selection which uses a outline stroke instead of filled rect."
                );
                egui::Grid::new("overlay style preferences").show(ui, |ui| {
                    ui.label("Button color:");
                    color_edit_button_srgba(ui, &mut style.overlay.button_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Button border color:");
                    color_edit_button_srgba(ui, &mut style.overlay.button_border_stroke.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Selection color:");
                    color_edit_button_srgba(ui, &mut style.overlay.selection_color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Button stroke color:");
                    color_edit_button_srgba(ui, &mut style.overlay.button_border_stroke.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Button stroke width:");
                    ui.add(Slider::new(&mut style.overlay.button_border_stroke.width, 0.0..=50.0));
                    ui.end_row();
                });
            });

            ui.collapsing("Hover highlight", |ui|{
                egui::Grid::new("leaf highlighting prefs").show(ui, |ui|{
                    ui.label("Fill color:");
                    color_edit_button_srgba(ui, &mut style.overlay.hovered_leaf_highlight.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Stroke color:");
                    color_edit_button_srgba(ui, &mut style.overlay.hovered_leaf_highlight.stroke.color, Alpha::OnlyBlend);
                    ui.end_row();

                    ui.label("Stroke width:");
                    ui.add(Slider::new(&mut style.overlay.hovered_leaf_highlight.stroke.width, 0.0..=50.0));
                    ui.end_row();

                    ui.label("Expansion:");
                    ui.add(Slider::new(&mut style.overlay.hovered_leaf_highlight.expansion, -50.0..=50.0));
                    ui.end_row();
                });
                ui.label("Rounding:");
                rounding_ui(ui, &mut style.overlay.hovered_leaf_highlight.rounding);
            })
        });
    }
}

impl TabViewer for MyContext {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "Style Editor" => self.style_editor(ui),
            "Ingredients" => {
                self.selected_category = None;

                if self.update_ingredients {
                    self.update_ingredients = false;
                    self.ingredients_list = self.database.get_ingredients();

                    //Upgrade log entries in case ingredient was deleted.
                    self.update_log_entries = true;
                }

                if self.update_categories {
                    self.update_categories = false;
                    self.categories_list = self.database.get_categories();
                }

                self.ingredients_view(ui);
            }
            "Categories" => {
                self.selected_ingredient = None;
                self.selected_ingredient_nutrition_info = None;

                if self.update_categories {
                    self.update_categories = false;
                    self.categories_list = self.database.get_categories();
                }

                self.categories_view(ui);
            }
            "Details" => self.details_view(ui),
            "Daily Log" => {
                self.selected_log_entry = None;

                if self.update_log_entries {
                    self.update_log_entries = false;
                    self.log_entry_list = self.database.get_log_entries(&self.date.unwrap());
                    self.log_entry_dates = self.database.get_log_entry_dates();
                }

                self.daily_log_view(ui)
            },
            _ => {
                ui.label(tab.as_str());
            }
        }
    }

    fn context_menu(
        &mut self,
        ui: &mut Ui,
        tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
        match tab.as_str() {
            _ => {
                ui.label(tab.to_string());
                ui.label("This is a context menu");
            }
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        ["Style Editor"].contains(&tab.as_str())
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_tabs.remove(tab);
        true
    }
}
