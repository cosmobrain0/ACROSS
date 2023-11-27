use std::fs;
use std::io::prelude::*;
use std::{error::Error, io::Write, path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};

/// Represents the data saved for one game
/// in a save file
#[derive(Debug, Clone, Copy)]
pub struct SaveData {
    pub date: DateTime<Utc>,
    pub score: usize,
}
impl SaveData {
    pub fn new(date: DateTime<Utc>, score: usize) -> Self {
        SaveData { date, score }
    }
}

/// Saves the given data to the end of the given save file,
/// or returns an error message if this fails
pub fn save_to_file(path: PathBuf, save_data: SaveData) -> Result<(), Box<dyn Error>> {
    let date = save_data.date.to_rfc3339();
    let score = save_data.score;

    if !path.exists()
        || std::fs::read_to_string(&path)?
            .lines()
            .filter(|x| x.trim().len() != 0)
            .count()
            == 0
    {
        fs::write(&path, format!("date,score\n{date},{score}"))?;
    } else {
        let mut file = std::fs::OpenOptions::new().append(true).open(&path)?;
        write!(file, "\n{date},{score}")?;
    }

    Ok(())
}

/// Returns the data saved in the given file,
/// validating each line and ignoring invalid ones
/// or returns an error message if this fails
pub fn load_from_file(path: PathBuf) -> Result<Vec<SaveData>, Box<dyn Error>> {
    Ok(std::fs::read_to_string(&path)?
        .lines()
        .map(|x| x.trim())
        .map(|x| x.split(',').collect::<Vec<_>>())
        .filter_map(|x| {
            if x.len() == 2 {
                Some((x[0], x[1]))
            } else {
                None
            }
        })
        .map(|(date, score)| (DateTime::parse_from_rfc3339(date), score.parse::<usize>()))
        .filter(|(date, score)| date.is_ok() && score.is_ok())
        .map(|(date, score)| (date.unwrap(), score.unwrap()))
        .map(|(date, score)| (date.with_timezone(&Utc), score))
        .map(|(date, score)| SaveData { date, score })
        .collect())
}
