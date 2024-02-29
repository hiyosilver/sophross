use rusqlite::types::{ToSql, ToSqlOutput, Value};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Grams,
    Teaspoons,
    Tablespoons,
    Pieces,
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
            _ => panic!("{} is not a valid Unit!", input)
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
        }
    }
}

#[derive(Clone)]
pub struct Category {
    pub name: String,
    pub icon_name: String,
    pub icon_color: egui::Color32,
}

#[derive(Clone)]
pub struct Ingredient {
    pub name: String,
    pub amount: u32,
    pub unit: Unit,
    pub categories: Vec<Category>
}