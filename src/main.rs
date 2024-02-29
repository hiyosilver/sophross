#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ingredients;
mod toggle_image;

use ingredients::{Ingredient, Category, Unit};
use toggle_image::toggle_image;

use std::collections::{HashMap, HashSet};

use eframe::{egui, NativeOptions};
use eframe::epaint::textures::TextureFilter;
use egui::{color_picker::{color_edit_button_srgba, Alpha}, vec2, CentralPanel, ComboBox, Frame, Rounding, Slider, TopBottomPanel, Ui, ViewportBuilder, WidgetText, Style as BaseStyle, Visuals, Color32, Stroke, SizeHint, TextureOptions, ImageSource, TextureWrapMode};

use egui_dock::{
    AllowedSplits, DockArea, DockState, NodeIndex, OverlayType, Style, SurfaceIndex,
    TabInteractionStyle, TabViewer,
};

use egui_extras::{TableBuilder, Column};

use rusqlite::{params, Connection};

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

const DATABASE_NAME: &str = "data.db";
const ICON_NAMES: [&str; 2] = ["apple", "bread"];

fn get_icon_image_source(id: &str) -> ImageSource {
    match id {
        "apple" => egui::include_image!("../icons/categories/apple.png"),
        "bread" => egui::include_image!("../icons/categories/bread.png"),
        _ => egui::include_image!("../icons/categories/placeholder.png")
    }
}

fn main() -> eframe::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(vec2(800.0, 600.0))
            .with_min_inner_size(vec2(640.0, 480.0)),
        ..Default::default()
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
        let mut dock_state =
            DockState::new(vec!["Ingredients View".to_owned(), "Categories View".to_owned(), "Style Editor".to_owned()]);
        dock_state.translations.tab_context_menu.eject_button = "Undock".to_owned();
        let [a, b] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec!["Inspector".to_owned()],
        );
        let [_, _] = dock_state.main_surface_mut().split_below(
            a,
            0.7,
            vec!["File Browser".to_owned(), "Asset Manager".to_owned()],
        );
        let [_, _] =
            dock_state
                .main_surface_mut()
                .split_below(b, 0.5, vec!["Hierarchy".to_owned()]);

        let mut open_tabs = HashSet::new();

        for node in dock_state[SurfaceIndex::main()].iter() {
            if let Some(tabs) = node.tabs() {
                for tab in tabs {
                    open_tabs.insert(tab.clone());
                }
            }
        }

        let db_connection = Connection::open(DATABASE_NAME).unwrap();

        let categories_create_query = "
            CREATE TABLE IF NOT EXISTS categories (category_id INTEGER PRIMARY KEY, name TEXT, icon_name TEXT, icon_color TEXT);
        ";
        let categories_result = db_connection.execute(categories_create_query, ()).unwrap();
        
        if categories_result > 0 {
            let mut category_insert_statement = db_connection.prepare("
                INSERT INTO categories (name, icon_name, icon_color) VALUES (?1, ?2, ?3);
            ").unwrap();
            category_insert_statement.insert(params!["Fruit", "apple", "#80ff20"]).unwrap();
        }

        let ingredients_create_query = "
            CREATE TABLE IF NOT EXISTS ingredients (ingredient_id INTEGER PRIMARY KEY, name TEXT, amount INTEGER, unit INTEGER, categories TEXT);
        ";
        let ingredients_result = db_connection.execute(ingredients_create_query, ()).unwrap();
        
        if ingredients_result > 0 {
            let mut ingredient_insert_statement = db_connection.prepare("
                INSERT INTO ingredients (name, amount, unit, categories) VALUES (?1, ?2, ?3, ?4);
            ").unwrap();
            ingredient_insert_statement.insert(params!["Rice, brown",     100,    0, "1"]).unwrap();
            ingredient_insert_statement.insert(params!["Tofu",            100,    0, "1"]).unwrap();
            ingredient_insert_statement.insert(params!["Soy sauce",       1,      1, "1"]).unwrap();
            ingredient_insert_statement.insert(params!["Olive oil",       1,      2, "1"]).unwrap();
        }


        let context = MyContext {
            style: None,
            open_tabs,

            show_window_close: true,
            show_window_collapse: true,
            show_close_buttons: true,
            show_add_buttons: false,
            draggable_tabs: true,
            show_tab_name_on_hover: false,
            allowed_splits: AllowedSplits::default(),

            db_connection,
            ingredients_list: Vec::new(),
            show_new_ingredient_dialog: false,
            update_ingredients: true,
            selected_ingredient: None,

            new_ingredient_name: String::from(""),
            new_ingredient_name_was_empty: false,
            new_ingredient_amount: 1,
            new_ingredient_unit: Unit::Grams,

            categories_list: Vec::new(),
            show_new_category_dialog: false,
            update_categories: true,
            selected_category: None,

            new_category_name: String::from(""),
            new_category_name_was_empty: false,
            new_category_icon_name: String::from(""),
            new_category_icon_color: Color32::WHITE,
        };

        Self {
            context,
            tree: dock_state,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

    show_close_buttons: bool,
    show_add_buttons: bool,
    draggable_tabs: bool,
    show_tab_name_on_hover: bool,
    allowed_splits: AllowedSplits,
    show_window_close: bool,
    show_window_collapse: bool,

    db_connection: rusqlite::Connection,

    ingredients_list: Vec<Ingredient>,
    show_new_ingredient_dialog: bool,
    update_ingredients: bool,
    selected_ingredient: Option<usize>,

    new_ingredient_name: String,
    new_ingredient_name_was_empty: bool,
    new_ingredient_amount: u32,
    new_ingredient_unit: Unit,

    categories_list: Vec<Category>,
    show_new_category_dialog: bool,
    update_categories: bool,
    selected_category: Option<usize>,

    new_category_name: String,
    new_category_name_was_empty: bool,
    new_category_icon_name: String,
    new_category_icon_color: Color32,
}

impl MyContext {
    fn new_ingredient(&mut self, ui: &mut Ui) {
        macro_rules! create {
            () => {
                if self.new_ingredient_name.len() == 0 {
                    self.new_ingredient_name_was_empty = true;
                }
                else {
                    let mut statement = self.db_connection.prepare(
                        "INSERT INTO ingredients (name, amount, unit, categories) VALUES (?1, ?2, ?3, 1);"
                    ).unwrap();
                    let _ = statement.insert(params![
                        self.new_ingredient_name,
                        self.new_ingredient_amount,
                        self.new_ingredient_unit
                    ]).unwrap();
                    self.update_ingredients = true;
                    drop(statement);
                    cancel!();
                }
            };
        }
        macro_rules! clear {
            () => {
                self.new_ingredient_name.clear();
                self.new_ingredient_amount = 0;
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
                ui.colored_label(Color32::from_rgb(192, 32, 16), "Name is required!");
            }
        });
        ui.horizontal(|ui| {
            ui.label("Amount: ");
            ui.add(Slider::new(&mut self.new_ingredient_amount, 1..=100));
            ComboBox::from_label("Default unit")
                .selected_text(self.new_ingredient_unit.to_string())
                .show_ui(ui, |ui| {
                    for unit in [Unit::Grams, Unit::Teaspoons, Unit::Tablespoons, Unit::Pieces] {
                        ui.selectable_value(&mut self.new_ingredient_unit, unit, unit.to_string());
                    }
                })
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

    fn ingredients_view(&mut self, ui: &mut Ui, ingredients_data: Vec<Ingredient>) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.add_enabled_ui(!self.show_new_ingredient_dialog, |ui| {
                if ui.add(
                    egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../icons/plus-square.png"))
                            .tint(Color32::GRAY)
                            .fit_to_exact_size(vec2(16.0, 16.0)),
                        "New ingredient"
                    ).min_size(vec2(0.0, 24.0))
                ).clicked() {
                    self.show_new_ingredient_dialog = true;
                }
            });
            ui.label(format!("{} entries", self.ingredients_list.len()));
        });
        if self.show_new_ingredient_dialog {
            self.new_ingredient(ui);
        }
        ui.separator();
        TableBuilder::new(ui)
            .sense(egui::Sense::click())
            .striped(true)
            .cell_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                .with_main_align(egui::Align::LEFT)
            )
            .columns(Column::remainder().resizable(true), 4)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Categories");
                });
                header.col(|ui| {
                    ui.heading("Name");
                });
                header.col(|ui| {
                    ui.heading("Amount");
                });
                header.col(|ui| {
                    ui.heading("Unit");
                });
            })
            .body(|body| {
                body.rows(30.0, ingredients_data.len(), |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selected_ingredient.is_some_and(|idx| idx == row_index));

                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            for category in &ingredients_data[row_index].categories {
                                let icon_name = &category.icon_name;

                                ui.add(egui::Image::new(get_icon_image_source(icon_name).clone())
                                    .tint(category.icon_color)
                                    .texture_options(TextureOptions { 
                                        magnification: TextureFilter::Nearest,
                                        minification: TextureFilter::Nearest,
                                        wrap_mode: TextureWrapMode::ClampToEdge,
                                    } ));
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.label(&ingredients_data[row_index].name);
                    });
                    row.col(|ui| {
                        ui.label(&ingredients_data[row_index].amount.to_string());
                    });
                    row.col(|ui| {
                        ui.label(&ingredients_data[row_index].unit.to_string());
                    });

                    if row.response().clicked() {
                        self.selected_ingredient = Some(row_index);
                    }
                });
            });
    }

    fn new_category(&mut self, ui: &mut Ui) {
        macro_rules! create_category {
            () => {
                if self.new_category_name.len() == 0 {
                    self.new_category_name_was_empty = true;
                }
                else {
                    let mut statement = self.db_connection.prepare(
                        "INSERT INTO categories (name, icon_name, icon_color) VALUES (?1, ?2, ?3);"
                    ).unwrap();
                    let _ = statement.insert(params![
                        self.new_category_name,
                        self.new_category_icon_name,
                        self.new_category_icon_color.to_hex()
                    ]).unwrap();
                    self.update_categories = true;
                    drop(statement);
                    cancel_category!();
                }
            };
        }
        macro_rules! clear_category {
            () => {
                self.new_category_name.clear();
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
            let result = ui.text_edit_singleline(&mut self.new_category_name);
            if result.changed() && self.new_category_name.len() > 0 {
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
                    for name in ICON_NAMES {
                        ui.add(toggle_image::toggle_image(&mut true, &get_icon_image_source(name), self.new_category_icon_color));
                        /*
                        ui.add(egui::Image::new(get_icon_image_source(name))
                            .tint(self.new_category_icon_color)
                            .texture_options(TextureOptions { 
                                magnification: TextureFilter::Nearest,
                                minification: TextureFilter::Nearest,
                                wrap_mode: TextureWrapMode::ClampToEdge,
                            })
                        );
                        */
                    }
                });
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

    fn categories_view(&mut self, ui: &mut Ui, category_data: Vec<Category>) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
            ui.add_enabled_ui(!self.show_new_category_dialog, |ui| {
                if ui.add(
                    egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../icons/plus-square.png"))
                            .tint(Color32::GRAY)
                            .fit_to_exact_size(vec2(16.0, 16.0))
                            .texture_options(TextureOptions { 
                                magnification: TextureFilter::Nearest,
                                minification: TextureFilter::Nearest,
                                wrap_mode: TextureWrapMode::ClampToEdge,
                            }),
                        "New category"
                    ).min_size(vec2(0.0, 24.0))
                ).clicked() {
                    self.show_new_category_dialog = true;
                }
            });
            ui.label(format!("{} entries", self.ingredients_list.len()));
        });
        if self.show_new_category_dialog {
            self.new_category(ui);
        }
        ui.separator();
        TableBuilder::new(ui)
            .sense(egui::Sense::click())
            .striped(true)
            .cell_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                .with_main_align(egui::Align::LEFT)
            )
            .columns(Column::remainder().resizable(true), 2)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Icon");
                });
                header.col(|ui| {
                    ui.heading("Name");
                });
            })
            .body(|body| {
                body.rows(30.0, category_data.len(), |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selected_category.is_some_and(|idx| idx == row_index));

                    row.col(|ui| {
                        ui.add(egui::Image::new(get_icon_image_source(&category_data[row_index].icon_name).clone())
                            .tint(category_data[row_index].icon_color)
                            .texture_options(TextureOptions { 
                                magnification: TextureFilter::Nearest,
                                minification: TextureFilter::Nearest,
                                wrap_mode: TextureWrapMode::ClampToEdge,
                            }));
                    });
                    row.col(|ui| {
                        ui.label(&category_data[row_index].name);
                    });

                    if row.response().clicked() {
                        self.selected_category = Some(row_index);
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

                /*
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.nw, 0.0..=15.0),
                    "North-West"
                );
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.ne, 0.0..=15.0),
                    "North-East"
                );
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.sw, 0.0..=15.0),
                    "South-West"
                );
                labeled_widget!(
                    ui,
                    Slider::new(&mut tab_style.rounding.se, 0.0..=15.0),
                    "South-East"
                );
                */
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
            "Ingredients View" => {
                if !self.update_ingredients {
                    self.ingredients_view(ui, self.ingredients_list.clone())
                }
                else {
                    self.update_ingredients = false;
                    let ingredient_data: Vec<Ingredient> = {
                        let mut statement = self.db_connection.prepare("SELECT name, amount, unit, categories FROM ingredients").unwrap();
                        let ingredients_iter = statement.query_map([], |row| {
                            let categories: Vec<Category> = {
                                let mut statement2 = self.db_connection.prepare("SELECT name, icon_name, icon_color FROM categories WHERE category_id IN (1)").unwrap();
                                let categories_iter = statement2.query_map([], |row2| {
                                    Ok(Category {
                                        name: row2.get(0)?,
                                        icon_name: row2.get(1)?,
                                        icon_color: Color32::from_hex(&row2.get::<usize, String>(2).unwrap()).unwrap(),
                                    })
                                }).unwrap();

                                let mut data: Vec<Category> = Vec::new();
                                for category in categories_iter {
                                    data.push(category.unwrap());
                                }

                                data
                            };
                            Ok(Ingredient {
                                name: row.get(0)?,
                                amount: row.get(1)?,
                                unit: Unit::from_uint(row.get::<usize, i64>(2).unwrap() as u32),
                                categories
                            })
                        }).unwrap();

                        let mut data: Vec<Ingredient> = Vec::new();
                        for ingredient in ingredients_iter {
                            data.push(ingredient.unwrap());
                        }

                        data
                    };
                    self.ingredients_list = ingredient_data.clone();
                    self.ingredients_view(ui, ingredient_data)
                }
            },
            "Categories View" => {
                if !self.update_categories {
                    self.categories_view(ui, self.categories_list.clone())
                }
                else {
                    self.update_categories = false;
                    let categories_data: Vec<Category> = {
                        let mut statement = self.db_connection.prepare("SELECT name, icon_name, icon_color FROM categories").unwrap();
                        let categories_iter = statement.query_map([], |row| {
                            Ok(Category {
                                name: row.get(0)?,
                                icon_name: row.get(1)?,
                                icon_color: Color32::from_hex(&row.get::<usize, String>(2).expect("Could not parse hex value into color!")).unwrap(),
                            })
                        }).unwrap();

                        let mut data: Vec<Category> = Vec::new();
                        for category in categories_iter {
                            data.push(category.unwrap());
                        }

                        data
                    };
                    self.categories_list = categories_data.clone();
                    self.categories_view(ui, categories_data)
                }
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