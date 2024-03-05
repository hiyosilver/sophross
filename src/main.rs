#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ingredients;
mod toggle_image;
mod pie_chart;

use ingredients::*;
use toggle_image::toggle_image;
use pie_chart::pie_chart;

use std::collections::{HashSet};

use eframe::{egui, NativeOptions};
use eframe::epaint::textures::TextureFilter;
use egui::{color_picker::{color_edit_button_srgba, Alpha}, vec2, CentralPanel, ComboBox, Frame, Rounding, Slider, TopBottomPanel, Ui, ViewportBuilder, WidgetText, Style as BaseStyle, Visuals, Color32, Stroke, SizeHint, TextureOptions, ImageSource, TextureWrapMode, CursorIcon};

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

const ICON_NAMES: [&str; 6] = ["apple", "bean", "bread", "candy", "drink", "drop"];

fn get_icon_image_source(id: &str) -> ImageSource {
    match id {
        "apple" => egui::include_image!("../icons/categories/apple.png"),
        //"bean" => egui::include_image!("../icons/categories/bean.png"),
        "bread" => egui::include_image!("../icons/categories/bread.png"),
        //"candy" => egui::include_image!("../icons/categories/candy.png"),
        //"drink" => egui::include_image!("../icons/categories/drink.png"),
        //"drop" => egui::include_image!("../icons/categories/drop.png"),
        _ => egui::include_image!("../icons/categories/placeholder.png")
    }
}

fn setup_database() -> Connection {
    let database_name: &str = "data.db";
    let db_connection = Connection::open(database_name).unwrap();

    db_connection.execute("PRAGMA foreign_keys = ON", []).unwrap();

    let ingredients_create_query = "
        CREATE TABLE IF NOT EXISTS ingredients (
            id INTEGER PRIMARY KEY,
            name TEXT
        );
    ";
    db_connection.execute(ingredients_create_query, ()).unwrap();

    let categories_create_query = "
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY,
            name TEXT,
            icon_name TEXT,
            icon_color TEXT
        );
    ";
    db_connection.execute(categories_create_query, ()).unwrap();

    let ingredient_categories_create_query = "
        CREATE TABLE IF NOT EXISTS ingredient_categories (
            ingredient_id INTEGER,
            category_id INTEGER,
            FOREIGN KEY(ingredient_id) REFERENCES ingredients(id) ON DELETE CASCADE
            FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(ingredient_categories_create_query, ()).unwrap();

    let nutritional_info_create_query = "
        CREATE TABLE IF NOT EXISTS nutritional_info (
            id INTEGER PRIMARY KEY,
            default_amount REAL,
            default_unit INTEGER,
            kilocalories REAL,
            ingredient_id INTEGER,
            FOREIGN KEY(ingredient_id) REFERENCES ingredients(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(nutritional_info_create_query, ()).unwrap();

    let protein_sets_create_query = "
        CREATE TABLE IF NOT EXISTS protein_sets (
            protein_set_id INTEGER PRIMARY KEY,
            nutrition_info_id INTEGER,
        --essentials
            histidine REAL,
            isoleucine REAL,
            leucine REAL,
            lysine REAL,
            methionine REAL,
            phenylalanine REAL,
            threonine REAL,
            tryptophan REAL,
            valine REAL,
        --non-essentials
            alanine REAL,
            arginine REAL,
            asparagine REAL,
            aspartic_acid REAL,
            cysteine REAL,
            glutamic_acid REAL,
            glutamine REAL,
            glycine REAL,
            proline REAL,
            serine REAL,
            tyrosine REAL,
            FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(protein_sets_create_query, ()).unwrap();

    let fat_sets_create_query = "
        CREATE TABLE IF NOT EXISTS fat_sets (
            fat_set_id INTEGER PRIMARY KEY,
            nutrition_info_id INTEGER,
            saturated REAL,
            monounsaturated REAL,
            polyunsaturated REAL,
            FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(fat_sets_create_query, ()).unwrap();

    let carbohydrate_sets_create_query = "
        CREATE TABLE IF NOT EXISTS carbohydrate_sets (
            carbohydrate_set_id INTEGER PRIMARY KEY,
            nutrition_info_id INTEGER,
            starch REAL,
            fiber REAL,
            sugars REAL,
            sugar_alcohols REAL,
            FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(carbohydrate_sets_create_query, ()).unwrap();

    let vitamin_sets_create_query = "
        CREATE TABLE IF NOT EXISTS vitamin_sets (
            vitamin_set_id INTEGER PRIMARY KEY,
            nutrition_info_id INTEGER,
            vitamin_a REAL,
            vitamin_b1 REAL,
            vitamin_b2 REAL,
            vitamin_b3 REAL,
            vitamin_b5 REAL,
            vitamin_b6 REAL,
            vitamin_b9 REAL,
            vitamin_b12 REAL,
            vitamin_c REAL,
            vitamin_d REAL,
            vitamin_e REAL,
            vitamin_k REAL,
            betaine REAL,
            choline REAL,
            FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(vitamin_sets_create_query, ()).unwrap();

    let mineral_sets_create_query = "
        CREATE TABLE IF NOT EXISTS mineral_sets (
            mineral_set_id INTEGER PRIMARY KEY,
            nutrition_info_id INTEGER,
            calcium REAL,
            copper REAL,
            iron REAL,
            magnesium REAL,
            manganese REAL,
            phosphorus REAL,
            potassium REAL,
            selenium REAL,
            sodium REAL,
            zinc REAL,
            FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
        );
    ";
    db_connection.execute(mineral_sets_create_query, ()).unwrap();

    db_connection
}

fn get_ingredient_select_query() -> &'static str {
    "
    SELECT
        ing.id, name,
        default_amount, default_unit, kilocalories,
        --essentials
        histidine, isoleucine, leucine, lysine, methionine, phenylalanine,
        threonine, tryptophan, valine,
        --non-essentials
        alanine, arginine, asparagine, aspartic_acid, cysteine, glutamic_acid, glutamine,
        glycine, proline, serine, tyrosine,
        --fats
        saturated, monounsaturated, polyunsaturated,
        --carbohydrates
        starch, fiber, sugars, sugar_alcohols,
        --vitamins
        vitamin_a, vitamin_b1, vitamin_b2, vitamin_b3, vitamin_b5, vitamin_b6, vitamin_b9,
        vitamin_b12, vitamin_c, vitamin_d, vitamin_e, vitamin_k, betaine, choline,
        --minerals
        calcium, copper, iron, magnesium, manganese,
        phosphorus, potassium, selenium, sodium, zinc
    FROM ingredients ing
    INNER JOIN nutritional_info ni
        ON ing.id = ni.ingredient_id
    INNER JOIN protein_sets ps
        ON ni.id = ps.nutrition_info_id
    INNER JOIN fat_sets fs
        ON ni.id = ps.nutrition_info_id
    INNER JOIN carbohydrate_sets cs
        ON ni.id = ps.nutrition_info_id
    INNER JOIN vitamin_sets vs
        ON ni.id = ps.nutrition_info_id
    INNER JOIN mineral_sets ms
        ON ni.id = ps.nutrition_info_id;
    "
}

fn get_ingredient_insert_query(ingredient: Ingredient) -> String {
    fn category_inserts(ingredient: &Ingredient) -> String {
        ingredient.categories.iter().map(|n| format!(
            "{}, {}",
            "(SELECT value FROM _variables WHERE var_name = 'ingredient_id' LIMIT 1)",
            n.id.to_string()
        )).collect::<Vec<String>>().join(",\n")
    }

    format!(
        "
        BEGIN TRANSACTION;

        --PRAGMA temp_store = 2;

        --errors without this for some godforsaken reason

        CREATE TEMP TABLE IF NOT EXISTS _variables(var_name TEXT, value INTEGER);

        INSERT INTO ingredients (
            name
        )
        VALUES ('{}');

        INSERT INTO _variables (var_name, value) VALUES ('ingredient_id', last_insert_rowid());

        INSERT INTO ingredient_categories (
            ingredient_id, category_id
        )
        VALUES({});

        INSERT INTO nutritional_info (
            default_amount, default_unit, kilocalories, ingredient_id
        )
        VALUES ({:.1}, {}, {:.1}, (SELECT value FROM _variables WHERE var_name = 'ingredient_id' LIMIT 1));

        INSERT INTO _variables (var_name, value) VALUES ('nutritional_info_id', last_insert_rowid());

        INSERT INTO protein_sets (
            nutrition_info_id,
            --essentials
            histidine, isoleucine, leucine, lysine, methionine, phenylalanine,
            threonine, tryptophan, valine,
            --non-essentials
            alanine, arginine, asparagine, aspartic_acid, cysteine, glutamic_acid, glutamine,
            glycine, proline, serine, tyrosine
        )
        VALUES ((SELECT value FROM _variables WHERE var_name = 'nutritional_info_id' LIMIT 1),
            {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1});

        INSERT INTO fat_sets (
            nutrition_info_id,
            saturated, monounsaturated, polyunsaturated
        )
        VALUES ((SELECT value FROM _variables WHERE var_name = 'nutritional_info_id' LIMIT 1),
            {:.1}, {:.1}, {:.1});

        INSERT INTO carbohydrate_sets (
            nutrition_info_id,
            starch, fiber, sugars, sugar_alcohols
        )
        VALUES ((SELECT value FROM _variables WHERE var_name = 'nutritional_info_id' LIMIT 1),
            {:.1}, {:.1}, {:.1}, {:.1});

        INSERT INTO vitamin_sets (
            nutrition_info_id,
            vitamin_a, vitamin_b1, vitamin_b2, vitamin_b3, vitamin_b5, vitamin_b6, vitamin_b9,
            vitamin_b12, vitamin_c, vitamin_d, vitamin_e, vitamin_k, betaine, choline
        )
        VALUES ((SELECT value FROM _variables WHERE var_name = 'nutritional_info_id' LIMIT 1),
            {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1});

        INSERT INTO mineral_sets (
            nutrition_info_id,
            calcium, copper, iron, magnesium, manganese,
            phosphorus, potassium, selenium, sodium, zinc
        )
        VALUES ((SELECT value FROM _variables WHERE var_name = 'nutritional_info_id' LIMIT 1),
            {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1}, {:.1});

        DROP TABLE IF EXISTS _variables;

        COMMIT;
        ",
        ingredient.name,
        category_inserts(&ingredient),
        ingredient.nutritional_info.default_amount,
        ingredient.nutritional_info.default_unit as u8,
        ingredient.nutritional_info.kilocalories,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.histidine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.isoleucine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.leucine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.lysine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.methionine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.phenylalanine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.threonine,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.tryptophan,
        ingredient.nutritional_info.macronutrients.proteins.essential_amino_acids.valine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.alanine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.arginine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.asparagine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.aspartic_acid,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.cysteine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.glutamic_acid,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.glutamine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.glycine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.proline,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.serine,
        ingredient.nutritional_info.macronutrients.proteins.non_essential_amino_acids.tyrosine,
        ingredient.nutritional_info.macronutrients.fats.saturated,
        ingredient.nutritional_info.macronutrients.fats.monounsaturated,
        ingredient.nutritional_info.macronutrients.fats.polyunsaturated,
        ingredient.nutritional_info.macronutrients.carbohydrates.starch,
        ingredient.nutritional_info.macronutrients.carbohydrates.fiber,
        ingredient.nutritional_info.macronutrients.carbohydrates.sugars,
        ingredient.nutritional_info.macronutrients.carbohydrates.sugar_alcohols,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_a,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b1,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b2,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b3,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b5,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b6,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b9,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_b12,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_c,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_d,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_e,
        ingredient.nutritional_info.micronutrients.vitamins.vitamin_k,
        ingredient.nutritional_info.micronutrients.vitamins.betaine,
        ingredient.nutritional_info.micronutrients.vitamins.choline,
        ingredient.nutritional_info.micronutrients.minerals.calcium,
        ingredient.nutritional_info.micronutrients.minerals.copper,
        ingredient.nutritional_info.micronutrients.minerals.iron,
        ingredient.nutritional_info.micronutrients.minerals.magnesium,
        ingredient.nutritional_info.micronutrients.minerals.manganese,
        ingredient.nutritional_info.micronutrients.minerals.phosphorus,
        ingredient.nutritional_info.micronutrients.minerals.potassium,
        ingredient.nutritional_info.micronutrients.minerals.selenium,
        ingredient.nutritional_info.micronutrients.minerals.sodium,
        ingredient.nutritional_info.micronutrients.minerals.zinc
    )
}

fn get_ingredient_delete_query(ingredient: &Ingredient) -> String {
    format!(
        "
        DELETE FROM ingredients
        WHERE id = {}
        ",
        ingredient.id
    )
}

fn get_category_delete_query(category: &Category) -> String {
    format!(
        "
        DELETE FROM categories
        WHERE id = {}
        ",
        category.id
    )
}

fn main() -> eframe::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(vec2(1200.0, 800.0))
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
        let [a, b] =
            dock_state
                .main_surface_mut()
                .split_left(NodeIndex::root(), 0.3, vec!["Inspector".to_owned()],
        );
        let [_, _] =
            dock_state
                .main_surface_mut()
                .split_below(a,2.0/3.0, vec!["Details".to_owned()],
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

        let db_connection = setup_database();

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
}

impl MyContext {
    fn new_ingredient(&mut self, ui: &mut Ui) {
        macro_rules! create {
            () => {
                if self.new_ingredient_name.len() == 0 {
                    self.new_ingredient_name_was_empty = true;
                }
                else {
                    self.new_ingredient_nutritional_info.as_mut().unwrap().default_amount = self.new_ingredient_amount;
                    self.new_ingredient_nutritional_info.as_mut().unwrap().default_unit = self.new_ingredient_unit;
                    self.new_ingredient_nutritional_info.as_mut().unwrap().kilocalories = self.new_ingredient_calories;

                    let _ = self.db_connection.execute_batch(&get_ingredient_insert_query(
                        Ingredient {
                            id: 0,
                            name: self.new_ingredient_name.clone(),
                            categories: self.new_ingredient_selected_categories.iter().map(|n| self.categories_list[*n].clone()).collect(),
                            nutritional_info: self.new_ingredient_nutritional_info.clone().unwrap()
                        }
                    ));

                    self.update_ingredients = true;
                    cancel!();
                }
            };
        }
        macro_rules! clear {
            () => {
                self.new_ingredient_name.clear();
                self.new_ingredient_name_was_empty = false;
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
                    egui::RichText::new("Name is required!").strong());
            }
        });
        ui.horizontal(|ui| {
            ui.label("Amount: ");
            ui.add(egui::DragValue::new(&mut self.new_ingredient_amount));
            ComboBox::from_label("Default unit")
                .selected_text(self.new_ingredient_unit.to_string())
                .show_ui(ui, |ui| {
                    for unit in [Unit::Grams, Unit::Teaspoons, Unit::Tablespoons, Unit::Pieces] {
                        ui.selectable_value(&mut self.new_ingredient_unit, unit, unit.to_string());
                    }
                })
        });
        ui.horizontal(|ui| {
            ui.label("Calories: ");
            ui.add(egui::DragValue::new(&mut self.new_ingredient_calories));
        });
        ui.horizontal(|ui| {
            egui::Grid::new("category_icon_grid")
                .spacing(vec2(-4.0, 0.0))
                .show(ui, |ui| {
                    for idx in 0..self.categories_list.len() {
                        let category = &self.categories_list[idx];
                        let category_selected = self.new_ingredient_selected_categories.contains(&idx);

                        if ui.add(toggle_image::toggle_image(category_selected, true, &get_icon_image_source(&category.icon_name), category.icon_color, vec2(16.0, 16.0)))
                            .on_hover_text_at_pointer(category.name.clone())
                            .changed() {
                            if category_selected {
                                let index = self.new_ingredient_selected_categories.iter().position(|x| *x == idx).unwrap();
                                self.new_ingredient_selected_categories.remove(index);
                            }
                            else {
                                self.new_ingredient_selected_categories.push(idx);
                            }
                        }
                    }
                });
            if self.new_ingredient_selected_categories.len() == 0 {
                ui.colored_label(
                    Color32::from_rgb(192, 192, 16),
                    egui::RichText::new("Consider adding a category."));
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
                        "New ingredient"
                    ).min_size(vec2(0.0, 24.0))
                ).clicked() {
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
                                }
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
                            }
                        },
                    });
                }
            });
            ui.label(format!("{} {}", self.ingredients_list.len(), if self.ingredients_list.len() == 1 { "entry" } else { "entries" }));
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
            .column(Column::auto().resizable(true))
            .columns(Column::remainder().resizable(true), 2)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Categories");
                });
                header.col(|ui| {
                    ui.heading("Name");
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

                                let response= ui.add(egui::Image::new(get_icon_image_source(icon_name).clone())
                                    .tint(category.icon_color)
                                    .fit_to_exact_size(vec2(16.0, 16.0))
                                    .texture_options(TextureOptions {
                                        magnification: TextureFilter::Nearest,
                                        minification: TextureFilter::Nearest,
                                        wrap_mode: TextureWrapMode::ClampToEdge,
                                    } ));

                                ui.add_space(-4.0);

                                //Workaround to allow for both clicking rows and showing tooltip
                                if ui.rect_contains_pointer(response.rect) {
                                    egui::show_tooltip(ui.ctx(), egui::Id::new("category_tooltip"), |ui| {
                                        ui.label(category.name.clone());
                                    });
                                }
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.label(&self.ingredients_list[row_index].name);
                    });
                    row.col(|ui| {
                        ui.label(&self.ingredients_list[row_index].nutritional_info.kilocalories.to_string());
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
                let valid_name = self.new_category_name.len() > 0;
                let valid_icon = self.new_category_selected_icon.is_some();
                if !valid_name {
                    self.new_category_name_was_empty = true;
                }
                if !valid_icon {
                    self.new_category_selected_icon_was_invalid = true;
                }
                if valid_name && valid_icon {
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
                    for idx in 0..ICON_NAMES.len() {
                        let active: bool = {
                            if let Some(index) = self.new_category_selected_icon {
                                idx == index
                            }
                            else {
                                false
                            }
                        };
                        if ui.add(toggle_image::toggle_image(active, false, &get_icon_image_source(ICON_NAMES[idx]), self.new_category_icon_color, vec2(16.0, 16.0))).changed() {
                            self.new_category_selected_icon = Some(idx);
                            self.new_category_icon_name = ICON_NAMES[idx].to_owned();
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
            ui.label(format!("{} {}", self.categories_list.len(), if self.categories_list.len() == 1 { "entry" } else { "entries" }));
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
                body.rows(30.0, category_data.len(), |mut row| {
                    let row_index = row.index();

                    row.set_selected(self.selected_category.is_some_and(|idx| idx == row_index));

                    row.col(|ui| {
                        ui.add(egui::Image::new(get_icon_image_source(&category_data[row_index].icon_name).clone())
                            .tint(category_data[row_index].icon_color)
                            .fit_to_exact_size(vec2(16.0, 16.0))
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

    fn details_view(&mut self, ui: &mut Ui) {
        fn nutritional_info_view(ui: &mut Ui, ingredient: &Ingredient) {
            ui.collapsing("Nutritional info", |ui| {
                ui.label(format!("per {}{}:",
                                 ingredient.nutritional_info.default_amount,
                                 ingredient.nutritional_info.default_unit));
                ui.label(format!("Calories: {}",
                                 ingredient.nutritional_info.kilocalories));
                ui.collapsing("Macronutrients", |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(format!("Proteins: {}", ingredient.nutritional_info.macronutrients.proteins.total_proteins()));
                        ui.label(format!("Fats: {}", ingredient.nutritional_info.macronutrients.fats.total_fats()));
                        ui.label(format!("Carbohydrates (total/net): {}/{}",
                                         ingredient.nutritional_info.macronutrients.carbohydrates.total_carbs(),
                                         ingredient.nutritional_info.macronutrients.carbohydrates.net_carbs()));
                                         
                        ui.add(pie_chart::pie_chart(vec2(4.0, 4.0)))
                            .on_hover_text_at_pointer("This is a pie chart!");
                    });
                });
                ui.collapsing("Micronutrients", |ui| {
                });
            });
        }

        if let Some(idx) = self.selected_ingredient {
            let ingredient = &self.ingredients_list[idx];
            ui.horizontal(|ui| {
                for category in &ingredient.categories {
                    let icon_name = &category.icon_name;

                    ui.add(egui::Image::new(get_icon_image_source(icon_name).clone())
                        .tint(category.icon_color)
                        .fit_to_exact_size(vec2(16.0, 16.0))
                        .texture_options(TextureOptions {
                            magnification: TextureFilter::Nearest,
                            minification: TextureFilter::Nearest,
                            wrap_mode: TextureWrapMode::ClampToEdge,
                        } )).on_hover_text(category.name.clone());

                    ui.add_space(-4.0);
                }
                ui.heading(&ingredient.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.add(
                        egui::Button::image_and_text(
                            //egui::Image::new(egui::include_image!("../icons/delete.png"))
                            egui::Image::new(egui::include_image!("../icons/categories/placeholder.png"))
                                .tint(Color32::RED)
                                .fit_to_exact_size(vec2(16.0, 16.0))
                                .texture_options(TextureOptions {
                                    magnification: TextureFilter::Nearest,
                                    minification: TextureFilter::Nearest,
                                    wrap_mode: TextureWrapMode::ClampToEdge,
                                }),
                            "Delete"
                        ).min_size(vec2(0.0, 24.0))
                    ).clicked() {
                        let mut delete_statement = self.db_connection.prepare(&get_ingredient_delete_query(ingredient)).unwrap();
                        let delete_result = delete_statement.execute([]);
                        if let Ok(_) = delete_result {
                            self.update_ingredients = true;
                            if self.ingredients_list.len() == 1 {
                                self.selected_ingredient = None;
                            }
                            else if let Some(selected) = self.selected_ingredient {
                                if selected == self.ingredients_list.len() - 1 {
                                    self.selected_ingredient = Some(selected - 1);
                                }
                            }
                        }
                    }
                });
            });
            nutritional_info_view(ui, ingredient);
        }
        else if let Some(idx) = self.selected_category {
            let category = &self.categories_list[idx];

            ui.horizontal(|ui| {
                ui.add_space(-4.0);
                ui.add(egui::Image::new(get_icon_image_source(&category.icon_name).clone())
                    .tint(category.icon_color)
                    .fit_to_exact_size(vec2(16.0, 16.0))
                    .texture_options(TextureOptions {
                        magnification: TextureFilter::Nearest,
                        minification: TextureFilter::Nearest,
                        wrap_mode: TextureWrapMode::ClampToEdge,
                    } )).on_hover_text(category.name.clone());

                ui.heading(&category.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.add(
                        egui::Button::image_and_text(
                            //egui::Image::new(egui::include_image!("../icons/delete.png"))
                            egui::Image::new(egui::include_image!("../icons/categories/placeholder.png"))
                                .tint(Color32::RED)
                                .fit_to_exact_size(vec2(16.0, 16.0))
                                .texture_options(TextureOptions {
                                    magnification: TextureFilter::Nearest,
                                    minification: TextureFilter::Nearest,
                                    wrap_mode: TextureWrapMode::ClampToEdge,
                                }),
                            "Delete"
                        ).min_size(vec2(0.0, 24.0))
                    ).clicked() {
                        let mut delete_statement = self.db_connection.prepare(&get_category_delete_query(category)).unwrap();
                        let delete_result = delete_statement.execute([]);
                        if let Ok(result) = delete_result {
                            self.update_categories = true;
                            if result > 0 {
                                self.update_ingredients = true;
                            }
                            if self.categories_list.len() == 1 {
                                self.selected_category = None;
                            }
                            else if let Some(selected) = self.selected_category {
                                if selected == self.categories_list.len() - 1 {
                                    self.selected_category = Some(selected - 1);
                                }
                            }
                        }
                    }
                });
            });
        }
        else {
            ui.centered_and_justified(|ui| {
                ui.label("-nothing selected-");
            });
        }
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
        fn get_category_data(context: &mut MyContext) -> Vec<Category> {
            let mut statement = context.db_connection.prepare("SELECT id, name, icon_name, icon_color FROM categories").unwrap();
            let categories_iter = statement.query_map([], |row| {
                Ok(Category {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    icon_name: row.get("icon_name")?,
                    icon_color: Color32::from_hex(&row.get::<&str, String>("icon_color").expect("Could not parse hex value into color!")).unwrap(),
                })
            }).unwrap();
            let mut data: Vec<Category> = Vec::new();
            for category in categories_iter {
                data.push(category.unwrap());
            }

            data
        }

        match tab.as_str() {
            "Style Editor" => self.style_editor(ui),
            "Ingredients View" => {
                self.selected_category = None;
                if self.update_ingredients {
                    self.update_ingredients = false;
                    let ingredient_data: Vec<Ingredient> = {
                        let mut statement = self.db_connection.prepare(get_ingredient_select_query()).unwrap();
                        let ingredients_iter = statement.query_map([], |row| {
                            let categories: Vec<Category> = {
                                let sql = format!(
                                    "
                                        SELECT id, name, icon_name, icon_color
                                        FROM categories c
                                        INNER JOIN ingredient_categories ic
                                            ON ic.category_id = c.id
                                            AND ic.ingredient_id = {};
                                    ",
                                    &row.get::<usize, u32>(0).unwrap());
                                let mut category_statement = self.db_connection.prepare(&sql).unwrap();
                                let categories_iter = category_statement.query_map([], |category_row| {
                                    Ok(Category {
                                        id: category_row.get("id")?,
                                        name: category_row.get("name")?,
                                        icon_name: category_row.get("icon_name")?,
                                        icon_color: Color32::from_hex(&category_row.get::<&str, String>("icon_color").expect("Could not parse hex value into color!")).unwrap(),
                                    })
                                }).unwrap();

                                let mut data: Vec<Category> = Vec::new();
                                for category in categories_iter {
                                    data.push(category.unwrap());
                                }

                                data
                            };
                            let nutritional_info: NutritionalInfo = {
                                NutritionalInfo {
                                    default_amount: row.get("default_amount")?,
                                    default_unit: Unit::from_uint(row.get("default_unit")?),
                                    kilocalories: row.get("kilocalories")?,
                                    macronutrients: Macronutrients {
                                        proteins: Proteins {
                                            essential_amino_acids: EssentialAminoAcids {
                                                histidine: row.get("histidine")?,
                                                isoleucine: row.get("isoleucine")?,
                                                leucine: row.get("leucine")?,
                                                lysine: row.get("lysine")?,
                                                methionine: row.get("methionine")?,
                                                phenylalanine: row.get("phenylalanine")?,
                                                threonine: row.get("threonine")?,
                                                tryptophan: row.get("tryptophan")?,
                                                valine: row.get("valine")?,
                                            },
                                            non_essential_amino_acids: NonEssentialAminoAcids {
                                                alanine: row.get("alanine")?,
                                                arginine: row.get("arginine")?,
                                                asparagine: row.get("asparagine")?,
                                                aspartic_acid: row.get("aspartic_acid")?,
                                                cysteine: row.get("cysteine")?,
                                                glutamic_acid: row.get("glutamic_acid")?,
                                                glutamine: row.get("glutamine")?,
                                                glycine: row.get("glycine")?,
                                                proline: row.get("proline")?,
                                                serine: row.get("serine")?,
                                                tyrosine: row.get("tyrosine")?,
                                            },
                                        },
                                        fats: Fats {
                                            saturated: row.get("saturated")?,
                                            monounsaturated: row.get("monounsaturated")?,
                                            polyunsaturated: row.get("polyunsaturated")?,
                                        },
                                        carbohydrates: Carbohydrates {
                                            starch: row.get("starch")?,
                                            fiber: row.get("fiber")?,
                                            sugars: row.get("sugars")?,
                                            sugar_alcohols: row.get("sugar_alcohols")?,
                                        }
                                    },
                                    micronutrients: Micronutrients {
                                        vitamins: Vitamins {
                                            vitamin_a: row.get("vitamin_a")?,
                                            vitamin_b1: row.get("vitamin_b1")?,
                                            vitamin_b2: row.get("vitamin_b2")?,
                                            vitamin_b3: row.get("vitamin_b3")?,
                                            vitamin_b5: row.get("vitamin_b5")?,
                                            vitamin_b6: row.get("vitamin_b6")?,
                                            vitamin_b9: row.get("vitamin_b9")?,
                                            vitamin_b12: row.get("vitamin_b12")?,
                                            vitamin_c: row.get("vitamin_c")?,
                                            vitamin_d: row.get("vitamin_d")?,
                                            vitamin_e: row.get("vitamin_e")?,
                                            vitamin_k: row.get("vitamin_k")?,
                                            betaine: row.get("betaine")?,
                                            choline: row.get("choline")?,
                                        },
                                        minerals: Minerals {
                                            calcium: row.get("calcium")?,
                                            copper: row.get("copper")?,
                                            iron: row.get("iron")?,
                                            magnesium: row.get("magnesium")?,
                                            manganese: row.get("manganese")?,
                                            phosphorus: row.get("phosphorus")?,
                                            potassium: row.get("potassium")?,
                                            selenium: row.get("selenium")?,
                                            sodium: row.get("sodium")?,
                                            zinc: row.get("zinc")?,
                                        }
                                    }
                                }
                            };
                            Ok(Ingredient {
                                id: row.get("id")?,
                                name: row.get("name")?,
                                categories,
                                nutritional_info,
                            })
                        }).unwrap();

                        let mut data: Vec<Ingredient> = Vec::new();
                        for ingredient in ingredients_iter {
                            data.push(ingredient.unwrap());
                        }

                        data
                    };
                    self.ingredients_list = ingredient_data;
                }

                if self.update_categories {
                    self.update_categories = false;
                    //let categories_data: Vec<Category> = get_category_data(self);
                    self.categories_list = get_category_data(self);
                }

                self.ingredients_view(ui);
            },
            "Categories View" => {
                self.selected_ingredient = None;
                if !self.update_categories {
                    self.categories_view(ui, self.categories_list.clone())
                }
                else {
                    self.update_categories = false;
                    let categories_data: Vec<Category>  = get_category_data(self);
                    self.categories_list = categories_data.clone();
                    self.categories_view(ui, categories_data)
                }
            },
            "Details" => {
                self.details_view(ui)
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