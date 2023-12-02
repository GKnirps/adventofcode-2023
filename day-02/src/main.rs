use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let games = content
        .lines()
        .map(parse_game)
        .collect::<Result<Vec<Game>, String>>()?;

    let possible_game_sum = possible_games(&games);
    println!("The sum of the IDs of possible games is {possible_game_sum}");

    Ok(())
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct Selection {
    red: u32,
    green: u32,
    blue: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Game {
    id: u32,
    selections: Vec<Selection>,
}

fn parse_game(line: &str) -> Result<Game, String> {
    let (game_id, selections) = line
        .split_once(": ")
        .ok_or_else(|| format!("no separator between game ID and selections in line '{line}'"))?;
    let game_id: u32 = game_id
        .strip_prefix("Game ")
        .ok_or_else(|| format!("prefix missing in line '{line}'"))?
        .parse::<u32>()
        .map_err(|e| format!("unable to parse game ID in line '{line}': {e}"))?;

    let selections = selections
        .split("; ")
        .map(parse_selection)
        .collect::<Result<Vec<Selection>, String>>()?;

    Ok(Game {
        id: game_id,
        selections,
    })
}

fn parse_selection(line: &str) -> Result<Selection, String> {
    let mut selection = Selection::default();
    for cubes in line.split(", ") {
        let (num, color) = cubes
            .split_once(' ')
            .ok_or_else(|| format!("no separator between color and number in '{line}'"))?;
        let num = num
            .parse::<u32>()
            .map_err(|e| format!("unable to parse number in '{line}': {e}"))?;
        match color {
            "red" => {
                selection.red += num;
            }
            "green" => {
                selection.green += num;
            }
            "blue" => {
                selection.blue += num;
            }
            _ => {
                return Err(format!("unknown color: '{color}'"));
            }
        }
    }
    Ok(selection)
}

fn possible_games(games: &[Game]) -> u32 {
    games
        .iter()
        .filter(|game| {
            game.selections.iter().all(|selection| {
                selection.red <= 12 && selection.green <= 13 && selection.blue <= 14
            })
        })
        .map(|game| game.id)
        .sum::<u32>()
}

#[cfg(test)]
mod test {
    use super::*;
}
