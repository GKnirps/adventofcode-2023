use std::collections::HashSet;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let cards = parse(&content)?;

    let points = total_points(&cards);
    println!("Total points: {points}");

    Ok(())
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Card {
    winning: HashSet<u32>,
    given: HashSet<u32>,
}

fn parse_card(line: &str) -> Result<Card, String> {
    let (_, numbers) = line
        .split_once(": ")
        .ok_or_else(|| format!("Unable to split off numbers part of line '{line}'"))?;
    let (winning, given) = numbers.split_once(" | ").ok_or_else(|| {
        format!("unable to split winning numbers from given numbers in line {line}")
    })?;

    let winning: HashSet<u32> = winning
        .split_whitespace()
        .map(|s| {
            s.parse::<u32>()
                .map_err(|e| format!("unable to parse '{s}' as number: {e}"))
        })
        .collect::<Result<HashSet<u32>, String>>()?;
    let given: HashSet<u32> = given
        .split_whitespace()
        .map(|s| {
            s.parse::<u32>()
                .map_err(|e| format!("unable to parse '{s}' as number: {e}"))
        })
        .collect::<Result<HashSet<u32>, String>>()?;

    Ok(Card { winning, given })
}

fn parse(input: &str) -> Result<Vec<Card>, String> {
    input.lines().map(parse_card).collect()
}

fn total_points(cards: &[Card]) -> u32 {
    cards
        .iter()
        .map(|card| card.winning.intersection(&card.given).count() as u32)
        .filter(|matches| *matches > 0)
        .map(|matches| 2u32.pow(matches - 1))
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11
"#;

    #[test]
    fn total_points_works_for_example() {
        // given
        let cards = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let points = total_points(&cards);

        // then
        assert_eq!(points, 13);
    }
}
