#![feature(isqrt)]

use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let races = parse(&content)?;

    let beat_record_prod = ways_to_win_prod(&races);
    println!("The product of the number of ways to win the race is {beat_record_prod}");

    let (time, distance) = fix_bad_kerning(&races);
    let beat_record_long_race = ways_to_win(time, distance);
    println!("When there is only one long race, there are {beat_record_long_race} ways to win");

    Ok(())
}

fn parse(input: &str) -> Result<Vec<(u64, u64)>, String> {
    let (time, distance) = input
        .split_once('\n')
        .ok_or_else(|| "expected two lines in the input".to_owned())?;
    let times = time
        .strip_prefix("Time:")
        .ok_or_else(|| format!("prefix 'Time:' missing in line '{time}'"))?
        .split_whitespace()
        .map(|s| {
            s.parse::<u64>()
                .map_err(|e| format!("unable to parse number '{s}': {e}"))
        });
    let distances = distance
        .strip_prefix("Distance:")
        .ok_or_else(|| format!("prefix 'Distance:' missing in line '{distance}'"))?
        .split_whitespace()
        .map(|s| {
            s.parse::<u64>()
                .map_err(|e| format!("unable to parse number '{s}': {e}"))
        });

    times.zip(distances).map(|(t, d)| Ok((t?, d?))).collect()
}

fn fix_bad_kerning(races: &[(u64, u64)]) -> (u64, u64) {
    races.iter().fold((0, 0), |(t, d), (part_t, part_d)| {
        (
            t * 10u64.pow(part_t.ilog10() + 1) + part_t,
            d * 10u64.pow(part_d.ilog10() + 1) + part_d,
        )
    })
}

fn ways_to_win_prod(races: &[(u64, u64)]) -> u64 {
    races
        .iter()
        .map(|(time, distance)| ways_to_win(*time, *distance))
        .product()
}

fn ways_to_win(time: u64, distance: u64) -> u64 {
    // this function assumes reasonable input, which I have not checked for
    // usually, the puzzle inputs are reasonable, but you never know
    // Bad input may lead to integer overflows or it may break the sqrt function
    let det = ((time / 2 + time % 2).pow(2) - distance).isqrt();
    let mut min_to_win = time / 2 - det;
    let mut max_to_win = time / 2 + det;
    // There was a lot of rounding involved, but the result has to be somewhere around here, search
    // for it(this feels kind of stupid, but whatever)
    while min_to_win * (time - min_to_win) > distance {
        min_to_win -= 1;
    }
    while min_to_win * (time - min_to_win) <= distance {
        min_to_win += 1;
    }
    while max_to_win * (time - max_to_win) > distance {
        max_to_win += 1;
    }
    while max_to_win * (time - max_to_win) <= distance {
        max_to_win -= 1
    }
    max_to_win - min_to_win + 1
}

#[cfg(test)]
mod test {
    use super::*;

    const RACES: &str = r#"Time:      7  15   30
Distance:  9  40  200
"#;

    #[test]
    fn ways_to_win_prod_works_for_example() {
        // given
        let races = parse(RACES).expect("expected successful parsing");

        // when
        let result = ways_to_win_prod(&races);

        // then
        assert_eq!(result, 288);
    }

    #[test]
    fn fix_bad_kerning_works_for_example() {
        // given
        let races = parse(RACES).expect("expected successful parsing");

        // when
        let (t, d) = fix_bad_kerning(&races);

        // then
        assert_eq!(t, 71530);
        assert_eq!(d, 940200);
    }

    #[test]
    fn ways_to_win_works_for_examples() {
        assert_eq!(ways_to_win(7, 9), 4);
        assert_eq!(ways_to_win(15, 40), 8);
        assert_eq!(ways_to_win(30, 200), 9);
        assert_eq!(ways_to_win(71530, 940200), 71503);
    }
}
