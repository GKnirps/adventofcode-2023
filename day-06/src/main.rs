use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let races = parse(&content)?;

    let beat_record_prod = ways_to_win(&races);
    println!("The product of the number of ways to win the race is {beat_record_prod}");

    let race = fix_bad_kerning(&races);
    // there is probably a smarter way to do this
    // e.g. using simple math to find the first and the last winning combination
    // and calculating the number of possibilities from there
    // with the `--release` flag, the brute force solution only takes about 125ms, so I'll skip the
    // optimization for now.
    let beat_record_long_race = ways_to_win(&[race]);
    println!("When there is only one long race, the product is {beat_record_long_race}");

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

fn ways_to_win(races: &[(u64, u64)]) -> usize {
    // for now, a simple brute force approach will do (too early in the morning).
    // I'll probably regret this in part 2
    races
        .iter()
        .map(|(time, distance)| {
            (1..*time)
                .map(move |a| a * (time - a))
                .filter(|d| d > distance)
                .count()
        })
        .product()
}

#[cfg(test)]
mod test {
    use super::*;

    const RACES: &str = r#"Time:      7  15   30
Distance:  9  40  200
"#;

    #[test]
    fn ways_to_win_works_for_example() {
        // given
        let races = parse(RACES).expect("expected successful parsing");

        // when
        let result = ways_to_win(&races);

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
}
