use chrono::NaiveDate;
use crate::ingredients::*;
use eframe::epaint::Color32;
use rusqlite::{Connection, Error as RusqliteError, Statement};
use log::log;
use std::rc::Rc;

const NAME: &str = "data.db";

pub struct Database {
    db_connection: Option<Rc<Connection>>,
}

impl Database {
    pub fn new() -> Self {
        let mut db = Database {
            db_connection: None,
        };
        db.setup_tables();
        db
    }

    fn start_connection(&mut self) {
        if self.db_connection.is_none() {
            self.db_connection =
                Some(Rc::new(Connection::open(NAME).expect("Unable to establish database connection!")));
            self.db_connection
                .as_ref()
                .unwrap()
                .execute("PRAGMA foreign_keys = ON", [])
                .expect("Failed to enable foreign key pragma!");
        }
    }

    fn setup_tables(&mut self) {
        self.start_connection();

        let ingredients_create_query = "
            CREATE TABLE IF NOT EXISTS ingredients (
                id INTEGER PRIMARY KEY,
                name TEXT,
                brand TEXT
            );
        ";
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(ingredients_create_query, ())
            .expect("Failed to create table 'ingredients'!");

        let categories_create_query = "
            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY,
                name TEXT,
                icon_name TEXT,
                icon_color TEXT
            );
        ";
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(categories_create_query, ())
            .expect("Failed to create table 'categories'!");

        let ingredient_categories_create_query = "
            CREATE TABLE IF NOT EXISTS ingredient_categories (
                ingredient_id INTEGER,
                category_id INTEGER,
                FOREIGN KEY(ingredient_id) REFERENCES ingredients(id) ON DELETE CASCADE
                FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE CASCADE
            );
        ";
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(ingredient_categories_create_query, ())
            .expect("Failed to create table 'ingredient_categories'!");

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
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(nutritional_info_create_query, ())
            .expect("Failed to create table 'nutritional_info'!");

        let protein_sets_create_query = "
            CREATE TABLE IF NOT EXISTS protein_sets (
                id INTEGER PRIMARY KEY,
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
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(protein_sets_create_query, ())
            .expect("Failed to create table 'protein_sets'!");

        let fat_sets_create_query = "
            CREATE TABLE IF NOT EXISTS fat_sets (
                id INTEGER PRIMARY KEY,
                nutrition_info_id INTEGER,
                saturated REAL,
                monounsaturated REAL,
                polyunsaturated REAL,
                FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
            );
        ";
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(fat_sets_create_query, ())
            .expect("Failed to create table 'fat_sets'!");

        let carbohydrate_sets_create_query = "
            CREATE TABLE IF NOT EXISTS carbohydrate_sets (
                id INTEGER PRIMARY KEY,
                nutrition_info_id INTEGER,
                starch REAL,
                fiber REAL,
                sugars REAL,
                sugar_alcohols REAL,
                FOREIGN KEY(nutrition_info_id) REFERENCES nutritional_info(id) ON DELETE CASCADE
            );
        ";
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(carbohydrate_sets_create_query, ())
            .expect("Failed to create table 'carbohydrate_sets'!");

        let vitamin_sets_create_query = "
            CREATE TABLE IF NOT EXISTS vitamin_sets (
                id INTEGER PRIMARY KEY,
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
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(vitamin_sets_create_query, ())
            .expect("Failed to create table 'vitamin_sets'!");

        let mineral_sets_create_query = "
            CREATE TABLE IF NOT EXISTS mineral_sets (
                id INTEGER PRIMARY KEY,
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
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(mineral_sets_create_query, ())
            .expect("Failed to create table 'mineral_sets'!");

        let daily_logs_create_query = "
            CREATE TABLE IF NOT EXISTS daily_logs (
                id INTEGER PRIMARY KEY,
                date TEXT,
                ingredient_id INTEGER,
                fraction REAL,
                FOREIGN KEY(ingredient_id) REFERENCES ingredients(id) ON DELETE CASCADE
            );
        ";
        self.db_connection
            .as_ref()
            .unwrap()
            .execute(daily_logs_create_query, ())
            .expect("Failed to create table 'daily_logs'!");
    }

    pub fn insert_category(&mut self, category: &Category) {
        self.start_connection();

        let mut statement = self
            .db_connection
            .as_ref()
            .unwrap()
            .prepare("INSERT INTO categories (name, icon_name, icon_color) VALUES (?1, ?2, ?3);")
            .unwrap();
        let _ = statement
            .insert(rusqlite::params![
                category.name,
                category.icon_name,
                category.icon_color.to_hex()
            ])
            .unwrap();
    }

    pub fn get_categories(&mut self) -> Vec<Category> {
        self.start_connection();

        let mut statement = self
            .db_connection
            .as_ref()
            .unwrap()
            .prepare("SELECT id, name, icon_name, icon_color FROM categories")
            .unwrap();
        let categories_iter = statement
            .query_map([], |row| {
                Ok(Category {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    icon_name: row.get("icon_name")?,
                    icon_color: Color32::from_hex(
                        &row.get::<&str, String>("icon_color")
                            .expect("Could not parse hex value into color!"),
                    )
                    .unwrap(),
                })
            })
            .unwrap();
        let mut data: Vec<Category> = Vec::new();
        for category in categories_iter {
            data.push(category.unwrap());
        }

        data
    }

    pub fn delete_category(&mut self, category: &Category) -> Result<usize, RusqliteError> {
        self.start_connection();

        let category_delete_query = format!(
            "
            DELETE FROM categories
            WHERE id = {}
            ",
            category.id
        );
        let mut delete_statement: Statement = self
            .db_connection
            .as_ref()
            .unwrap()
            .prepare(&category_delete_query)
            .expect("Failed to prepare delete statement!");
        delete_statement.execute([])
    }

    pub fn delete_categories(&mut self, categories: &[Category]) -> Result<usize, RusqliteError> {
        self.start_connection();

        let mut row_count: usize = 0;

        for category in categories {
            match self.delete_category(category) {
                Ok(value) => row_count += value,
                Err(error) => return Err(error),
            }
        }

        Ok(row_count)
    }

    pub fn insert_ingredient(&mut self, ingredient: &Ingredient) {
        self.start_connection();

        fn category_inserts(ingredient: &Ingredient) -> String {
            ingredient
                .categories
                .iter()
                .map(|n| {
                    format!(
                        "INSERT INTO ingredient_categories (
                            ingredient_id, category_id
                        )
                        VALUES({}, {});",
                        "(SELECT value FROM _variables WHERE var_name = 'ingredient_id' LIMIT 1)",
                        n.id.to_string()
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        }

        let ingredient_insert_query = format!(
            "
            BEGIN TRANSACTION;

            CREATE TEMP TABLE IF NOT EXISTS _variables(var_name TEXT, value INTEGER);

            INSERT INTO ingredients (
                name, brand
            )
            VALUES ('{}', '{}');

            INSERT INTO _variables (var_name, value) VALUES ('ingredient_id', last_insert_rowid());

            {}

            COMMIT;
            ",
            ingredient.name,
            ingredient.brand,
            category_inserts(&ingredient)
        );

        let _ = self
            .db_connection
            .as_ref()
            .unwrap()
            .execute_batch(&ingredient_insert_query);

        for nutritional_info in &ingredient.nutritional_info {
            let nutritional_info_insert_query = format!(
                "
                BEGIN TRANSACTION;

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
                nutritional_info.default_amount,
                nutritional_info.default_unit as u8,
                nutritional_info.kilocalories,
                nutritional_info.macronutrients.proteins.essential_amino_acids.histidine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.isoleucine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.leucine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.lysine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.methionine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.phenylalanine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.threonine,
                nutritional_info.macronutrients.proteins.essential_amino_acids.tryptophan,
                nutritional_info.macronutrients.proteins.essential_amino_acids.valine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.alanine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.arginine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.asparagine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.aspartic_acid,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.cysteine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.glutamic_acid,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.glutamine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.glycine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.proline,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.serine,
                nutritional_info.macronutrients.proteins.non_essential_amino_acids.tyrosine,
                nutritional_info.macronutrients.fats.saturated,
                nutritional_info.macronutrients.fats.monounsaturated,
                nutritional_info.macronutrients.fats.polyunsaturated,
                nutritional_info.macronutrients.carbohydrates.starch,
                nutritional_info.macronutrients.carbohydrates.fiber,
                nutritional_info.macronutrients.carbohydrates.sugars,
                nutritional_info.macronutrients.carbohydrates.sugar_alcohols,
                nutritional_info.micronutrients.vitamins.vitamin_a,
                nutritional_info.micronutrients.vitamins.vitamin_b1,
                nutritional_info.micronutrients.vitamins.vitamin_b2,
                nutritional_info.micronutrients.vitamins.vitamin_b3,
                nutritional_info.micronutrients.vitamins.vitamin_b5,
                nutritional_info.micronutrients.vitamins.vitamin_b6,
                nutritional_info.micronutrients.vitamins.vitamin_b9,
                nutritional_info.micronutrients.vitamins.vitamin_b12,
                nutritional_info.micronutrients.vitamins.vitamin_c,
                nutritional_info.micronutrients.vitamins.vitamin_d,
                nutritional_info.micronutrients.vitamins.vitamin_e,
                nutritional_info.micronutrients.vitamins.vitamin_k,
                nutritional_info.micronutrients.vitamins.betaine,
                nutritional_info.micronutrients.vitamins.choline,
                nutritional_info.micronutrients.minerals.calcium,
                nutritional_info.micronutrients.minerals.copper,
                nutritional_info.micronutrients.minerals.iron,
                nutritional_info.micronutrients.minerals.magnesium,
                nutritional_info.micronutrients.minerals.manganese,
                nutritional_info.micronutrients.minerals.phosphorus,
                nutritional_info.micronutrients.minerals.potassium,
                nutritional_info.micronutrients.minerals.selenium,
                nutritional_info.micronutrients.minerals.sodium,
                nutritional_info.micronutrients.minerals.zinc
            );

            let _ = self
                .db_connection
                .as_ref()
                .unwrap()
                .execute_batch(&nutritional_info_insert_query);
        }
    }

    pub fn get_ingredient_by_id(&mut self, id: u32) -> Vec<Rc<Ingredient>> {
        self.start_connection();

        let query = format!("
            SELECT
                ing.id, name, brand,
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
                ON ni.id = fs.nutrition_info_id
            INNER JOIN carbohydrate_sets cs
                ON ni.id = cs.nutrition_info_id
            INNER JOIN vitamin_sets vs
                ON ni.id = vs.nutrition_info_id
            INNER JOIN mineral_sets ms
                ON ni.id = ms.nutrition_info_id
            WHERE ing.id = {} LIMIT 1;
            ", id);

        let mut statement = self.db_connection.as_ref().unwrap().prepare(&query).unwrap();
        let ingredients_iter = statement
            .query_map([], |row| {
                let categories: Vec<Category> = {
                    let sql = format!(
                        "
                        SELECT id, name, icon_name, icon_color
                        FROM categories c
                        INNER JOIN ingredient_categories ic
                            ON ic.category_id = c.id
                            AND ic.ingredient_id = {};
                        ",
                        &row.get::<usize, u32>(0).unwrap()
                    );
                    let mut category_statement =
                        self.db_connection.as_ref().unwrap().prepare(&sql).unwrap();
                    let categories_iter = category_statement
                        .query_map([], |category_row| {
                            Ok(Category {
                                id: category_row.get("id")?,
                                name: category_row.get("name")?,
                                icon_name: category_row.get("icon_name")?,
                                icon_color: Color32::from_hex(
                                    &category_row
                                        .get::<&str, String>("icon_color")
                                        .expect("Could not parse hex value into color!"),
                                )
                                    .unwrap(),
                            })
                        })
                        .unwrap();

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
                            },
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
                            },
                        },
                    }
                };
                Ok(Ingredient {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    brand: row.get("brand")?,
                    categories,
                    nutritional_info: vec![nutritional_info],
                })
            })
            .unwrap();

        let mut data = Vec::new();
        for ingredient in ingredients_iter {
            data.push(Rc::new(ingredient.unwrap()));
            break;
        }

        data
    }

    pub fn get_ingredients(&mut self) -> Vec<Rc<Ingredient>> {
        self.start_connection();

        let query = "
            SELECT
                ing.id, name, brand,
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
                ON ni.id = fs.nutrition_info_id
            INNER JOIN carbohydrate_sets cs
                ON ni.id = cs.nutrition_info_id
            INNER JOIN vitamin_sets vs
                ON ni.id = vs.nutrition_info_id
            INNER JOIN mineral_sets ms
                ON ni.id = ms.nutrition_info_id;
            ";

        let mut statement = self.db_connection.as_ref().unwrap().prepare(query).unwrap();
        let ingredients_iter = statement
            .query_map([], |row| {
                let categories: Vec<Category> = {
                    let sql = format!(
                        "
                        SELECT id, name, icon_name, icon_color
                        FROM categories c
                        INNER JOIN ingredient_categories ic
                            ON ic.category_id = c.id
                            AND ic.ingredient_id = {};
                        ",
                        &row.get::<usize, u32>(0).unwrap()
                    );
                    let mut category_statement =
                        self.db_connection.as_ref().unwrap().prepare(&sql).unwrap();
                    let categories_iter = category_statement
                        .query_map([], |category_row| {
                            Ok(Category {
                                id: category_row.get("id")?,
                                name: category_row.get("name")?,
                                icon_name: category_row.get("icon_name")?,
                                icon_color: Color32::from_hex(
                                    &category_row
                                        .get::<&str, String>("icon_color")
                                        .expect("Could not parse hex value into color!"),
                                )
                                .unwrap(),
                            })
                        })
                        .unwrap();

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
                            },
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
                            },
                        },
                    }
                };
                Ok(Ingredient {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    brand: row.get("brand")?,
                    categories,
                    nutritional_info: vec![nutritional_info],
                })
            })
            .unwrap();

        let mut data: Vec<Rc<Ingredient>> = Vec::new();
        for ingredient in ingredients_iter {
            data.push(Rc::new(ingredient.unwrap()));
        }

        data
    }

    pub fn delete_ingredient(&mut self, ingredient: &Ingredient) -> Result<usize, RusqliteError> {
        self.start_connection();

        let ingredient_delete_query = format!(
            "
            DELETE FROM ingredients
            WHERE id = {}
            ",
            ingredient.id
        );
        let mut delete_statement: Statement = self
            .db_connection
            .as_ref()
            .unwrap()
            .prepare(&ingredient_delete_query)
            .expect("Failed to prepare delete statement!");
        delete_statement.execute([])
    }

    pub fn delete_ingredients(&mut self, ingredients: &[Ingredient]) -> Result<usize, RusqliteError> {
        self.start_connection();

        let mut row_count: usize = 0;

        for ingredient in ingredients {
            match self.delete_ingredient(ingredient) {
                Ok(value) => row_count += value,
                Err(error) => return Err(error),
            }
        }

        Ok(row_count)
    }

    pub fn insert_log_entry(&mut self, date: &NaiveDate, log_entry: &LogEntry) {
        self.start_connection();

        let mut statement = self
            .db_connection
            .as_ref()
            .unwrap()
            .prepare("INSERT INTO daily_logs (date, ingredient_id, fraction) VALUES (?1, ?2, ?3);")
            .unwrap();
        let _ = statement
            .insert(rusqlite::params![
                date,
                log_entry.ingredient.id,
                log_entry.fraction
            ])
            .unwrap();
    }

    pub fn get_log_entries(&mut self, date: &NaiveDate) -> Vec<LogEntry> {
        self.start_connection();

        let query = format!("
            SELECT
                id, date, ingredient_id, fraction
            FROM daily_logs WHERE date = '{}';
            ", date);

        let binding = self.db_connection.clone().unwrap();
        let mut statement = binding.prepare(&query).unwrap();

        let log_entries_iter = statement
            .query_map([], |row| {
                let ingredient: Rc<Ingredient> = self.get_ingredient_by_id(row.get("ingredient_id")?)[0].to_owned();
                Ok(LogEntry {
                    id: row.get("id")?,
                    ingredient,
                    fraction: row.get("fraction")?,
                })
            })
            .unwrap();

        let mut data: Vec<LogEntry> = Vec::new();
        for log_entry in log_entries_iter {
            data.push(log_entry.unwrap());
        }

        data
    }
}
