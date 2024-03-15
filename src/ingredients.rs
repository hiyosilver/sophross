use rusqlite::types::{ToSql, ToSqlOutput, Value};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Grams,
    Teaspoons,
    Tablespoons,
    Pieces,
    Cups,
}

impl ToSql for Unit {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Integer(*self as i64)))
    }
}

impl Unit {
    pub fn from_uint(input: u32) -> Self {
        match input {
            0 => Self::Grams,
            1 => Self::Teaspoons,
            2 => Self::Tablespoons,
            3 => Self::Pieces,
            4 => Self::Cups,
            _ => panic!("{} is not a valid Unit!", input),
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Unit::Grams => write!(f, "g"),
            Unit::Teaspoons => write!(f, "tsp"),
            Unit::Tablespoons => write!(f, "Tbsp"),
            Unit::Pieces => write!(f, "pc"),
            Unit::Cups => write!(f, "cup"),
        }
    }
}

#[derive(Clone)]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub icon_name: String,
    pub icon_color: egui::Color32,
}

#[derive(Clone)]
pub struct Ingredient {
    pub id: u32,
    pub name: String,
    pub brand: String,
    pub categories: Vec<Category>,
    pub nutritional_info: Vec<NutritionalInfo>,
}

#[derive(Clone)]
pub struct LogEntry {
    pub id: u32,
    pub ingredient: StIngredientring,
    pub fraction: f32,
}

#[derive(Clone)]
pub struct NutritionalInfo {
    pub default_amount: f32,
    pub default_unit: Unit,

    pub kilocalories: f32,
    pub macronutrients: Macronutrients,
    pub micronutrients: Micronutrients,
}

impl NutritionalInfo {
    pub fn estimate_calories(&self) -> f32 {
        self.macronutrients.estimate_calories()
    }
}

#[derive(Clone)]
pub struct Macronutrients {
    pub proteins: Proteins,
    pub fats: Fats,
    pub carbohydrates: Carbohydrates,
}

impl Macronutrients {
    fn estimate_calories(&self) -> f32 {
        self.proteins.total_proteins() * 4.0
            + self.fats.total_fats() * 9.0
            + self.carbohydrates.net_carbs() * 4.0
    }
}

#[derive(Clone)]
pub struct Proteins {
    pub essential_amino_acids: EssentialAminoAcids,
    pub non_essential_amino_acids: NonEssentialAminoAcids,
}

impl Proteins {
    pub fn total_proteins(&self) -> f32 {
        self.essential_amino_acids.total() + self.non_essential_amino_acids.total()
    }
}

#[derive(Clone)]
pub struct EssentialAminoAcids {
    pub histidine: f32,
    pub isoleucine: f32,
    pub leucine: f32,
    pub lysine: f32,
    pub methionine: f32,
    pub phenylalanine: f32,
    pub threonine: f32,
    pub tryptophan: f32,
    pub valine: f32,
}

impl EssentialAminoAcids {
    fn total(&self) -> f32 {
        self.histidine
            + self.isoleucine
            + self.leucine
            + self.lysine
            + self.methionine
            + self.phenylalanine
            + self.threonine
            + self.tryptophan
            + self.valine
    }
}

#[derive(Clone)]
pub struct NonEssentialAminoAcids {
    pub alanine: f32,
    pub arginine: f32,
    pub asparagine: f32,
    pub aspartic_acid: f32,
    pub cysteine: f32,
    pub glutamic_acid: f32,
    pub glutamine: f32,
    pub glycine: f32,
    pub proline: f32,
    pub serine: f32,
    pub tyrosine: f32,
}

impl NonEssentialAminoAcids {
    fn total(&self) -> f32 {
        self.alanine
            + self.arginine
            + self.asparagine
            + self.aspartic_acid
            + self.cysteine
            + self.glutamic_acid
            + self.glutamine
            + self.glycine
            + self.proline
            + self.serine
            + self.tyrosine
    }
}

#[derive(Clone)]
pub struct Fats {
    pub saturated: f32,
    pub monounsaturated: f32,
    pub polyunsaturated: f32,
}

impl Fats {
    pub fn total_fats(&self) -> f32 {
        self.saturated + self.monounsaturated + self.polyunsaturated
    }
}

#[derive(Clone)]
pub struct Carbohydrates {
    pub starch: f32,
    pub fiber: f32,
    pub sugars: f32,
    pub sugar_alcohols: f32,
}

impl Carbohydrates {
    pub fn total_carbs(&self) -> f32 {
        self.starch + self.fiber + self.sugars + self.sugar_alcohols
    }
    pub fn net_carbs(&self) -> f32 {
        self.starch + self.sugars + 0.5 * self.sugar_alcohols
    }
}

#[derive(Clone)]
pub struct Micronutrients {
    pub vitamins: Vitamins,
    pub minerals: Minerals,
}

#[derive(Clone)]
pub struct Vitamins {
    pub vitamin_a: f32,
    pub vitamin_b1: f32,
    pub vitamin_b2: f32,
    pub vitamin_b3: f32,
    pub vitamin_b5: f32,
    pub vitamin_b6: f32,
    pub vitamin_b9: f32,
    pub vitamin_b12: f32,
    pub vitamin_c: f32,
    pub vitamin_d: f32,
    pub vitamin_e: f32,
    pub vitamin_k: f32,
    pub betaine: f32,
    pub choline: f32,
}

#[derive(Clone)]
pub struct Minerals {
    pub calcium: f32,
    pub copper: f32,
    pub iron: f32,
    pub magnesium: f32,
    pub manganese: f32,
    pub phosphorus: f32,
    pub potassium: f32,
    pub selenium: f32,
    pub sodium: f32,
    pub zinc: f32,
}
