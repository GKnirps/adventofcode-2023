use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let calibration_sum: u32 = content.lines().map(parse_line).sum();
    println!("The sum of the calibration values is {calibration_sum}");

    let correct_calibration_sum: u32 = content.lines().map(full_calibration).sum();
    println!("The sum of the correct calibration values is {correct_calibration_sum}");

    Ok(())
}

fn parse_line(line: &str) -> u32 {
    line.chars()
        .filter_map(|c| c.to_digit(10))
        .next()
        .unwrap_or(0)
        * 10
        + line
            .chars()
            .rev()
            .filter_map(|c| c.to_digit(10))
            .next()
            .unwrap_or(0)
}

fn full_calibration(line: &str) -> u32 {
    [
        line.char_indices()
            .filter_map(|(i, c)| c.to_digit(10).map(|d| (i, d)))
            .next(),
        line.find("one").map(|i| (i, 1u32)),
        line.find("two").map(|i| (i, 2u32)),
        line.find("three").map(|i| (i, 3u32)),
        line.find("four").map(|i| (i, 4u32)),
        line.find("five").map(|i| (i, 5u32)),
        line.find("six").map(|i| (i, 6u32)),
        line.find("seven").map(|i| (i, 7u32)),
        line.find("eight").map(|i| (i, 8u32)),
        line.find("nine").map(|i| (i, 9u32)),
    ]
    .iter()
    .filter_map(|d| *d)
    .min_by_key(|(i, _)| *i)
    .map(|(_, d)| d)
    .unwrap_or(0)
        * 10
        + [
            line.char_indices()
                .rev()
                .filter_map(|(i, c)| c.to_digit(10).map(|d| (i, d)))
                .next(),
            line.rfind("one").map(|i| (i, 1u32)),
            line.rfind("two").map(|i| (i, 2u32)),
            line.rfind("three").map(|i| (i, 3u32)),
            line.rfind("four").map(|i| (i, 4u32)),
            line.rfind("five").map(|i| (i, 5u32)),
            line.rfind("six").map(|i| (i, 6u32)),
            line.rfind("seven").map(|i| (i, 7u32)),
            line.rfind("eight").map(|i| (i, 8u32)),
            line.rfind("nine").map(|i| (i, 9u32)),
        ]
        .iter()
        .filter_map(|d| *d)
        .max_by_key(|(i, _)| *i)
        .map(|(_, d)| d)
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use super::*;
}
