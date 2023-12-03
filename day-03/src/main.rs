use std::env;
use std::fs::read;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read(Path::new(&filename)).map_err(|e| e.to_string())?;

    let part_numbers = part_numbers_sum(&content);
    println!("The sum of the part numbers is {part_numbers}.");

    Ok(())
}

fn part_numbers_sum(schematic: &[u8]) -> u32 {
    // assume input is rectangular
    let width = schematic
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == b'\n')
        .map(|(i, _)| i + 1)
        .next()
        .unwrap_or(schematic.len());
    let height = schematic.len() / width;

    let mut sum: u32 = 0;
    let mut current: Option<u32> = None;
    let mut is_part = false;

    for y in 0..height {
        for x in 0..width {
            let c = schematic[y * width + x];
            if c.is_ascii_digit() {
                is_part = is_part || symbol_adjacent(schematic, width, height, x, y);
                let digit = (c - b'0') as u32;
                if let Some(number) = current {
                    current = Some(number * 10 + digit);
                } else {
                    current = Some(digit);
                }
            } else if let Some(number) = current {
                if is_part {
                    sum += number;
                }
                current = None;
                is_part = false;
            }
        }
    }
    sum
}

fn symbol_adjacent(schematic: &[u8], width: usize, height: usize, x: usize, y: usize) -> bool {
    let x = x as isize;
    let y = y as isize;
    [
        (x - 1, y - 1),
        (x - 1, y),
        (x - 1, y + 1),
        (x, y - 1),
        (x, y + 1),
        (x + 1, y - 1),
        (x + 1, y),
        (x + 1, y + 1),
    ]
    .iter()
    .filter_map(|(nx, ny)| get(schematic, width, height, *nx, *ny))
    .any(|c| c != b'.' && !c.is_ascii_digit() && c != b'\n')
}

fn get(schematic: &[u8], width: usize, height: usize, x: isize, y: isize) -> Option<u8> {
    if y < 0 || x < 0 || y as usize >= height || x as usize >= width {
        None
    } else {
        schematic.get(x as usize + y as usize * width).copied()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part_number_sum_work_for_example() {
        // given
        let schematic = br#"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..
"#;

        // when
        let result = part_numbers_sum(schematic);

        // then
        assert_eq!(result, 4361);
    }
}
