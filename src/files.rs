use std::fs;
use std::io::prelude::*;
use std::{error::Error, io::Write, path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy)]
pub struct SaveData {
    date: DateTime<Utc>,
    score: usize,
}
impl SaveData {
    pub fn new(date: DateTime<Utc>, score: usize) -> Self {
        SaveData { date, score }
    }
}

pub fn save_to_file(path: PathBuf, save_data: SaveData) -> Result<(), Box<dyn Error>> {
    let date = save_data.date.format("%d/%m/%Y %H:%M:%S");
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
        let mut file = fs::File::open(&path)?;
        write!(file, "\n{date},{score}")?;
    }

    Ok(())
}

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
