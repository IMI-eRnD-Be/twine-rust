use serde::Deserialize;
use serde_json::Result;
use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

// Valid color types
#[derive(Debug, Deserialize, PartialEq)]
pub enum Type {
    /// Primery colour
    Primary,
    /// Secondary colour
    Secondary,
}

impl Default for Type {
    fn default() -> Self { Type::Primary }
}

// Color codes structure
#[derive(Debug, Deserialize, PartialEq)]
pub struct Code {
    /// Color code as an rgba array
    pub rgba: Vec<u8>,
    /// Color code as a hex value
    pub hex: String,
}

// The colour structure
#[derive(Debug, Deserialize, PartialEq)]
pub struct Colour {
    pub colour_name: String,
    pub category: String,
    pub colour_type: Type,
    pub code: Code,
}

// The colours structure
#[derive(Debug, Deserialize, PartialEq)]
pub struct Colours {
    pub colour: Vec<Colour>,
}

pub fn read_colours<P>(path: P) -> Result<Colours>
where
    P:  AsRef<Path>,
{
    // Open the file in read-only mode with buffer
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Colours`.
    let colours: Colours = serde_json::from_reader(reader)?;

    // Return the `Colours` structure
    Ok(colours)
}
