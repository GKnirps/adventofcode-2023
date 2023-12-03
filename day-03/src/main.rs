use std::env;
use std::fs::read;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (numbers, symbols) = parse(&content);

    let part_numbers = part_numbers_sum(&numbers, &symbols);
    println!("The sum of the part numbers is {part_numbers}.");

    let gears = gear_ratio_sum(&numbers, &symbols);
    println!("The sum of gear ratios is {gears}.");

    Ok(())
}

#[derive(Debug)]
struct SchematicNumber {
    value: u32,
    line: isize,
    first: isize,
    length: isize,
}

#[derive(Debug)]
struct Symbol {
    s: u8,
    line: isize,
    col: isize,
}

fn parse(schematic: &[u8]) -> (Vec<SchematicNumber>, Vec<Symbol>) {
    // assume input is rectangular
    let width = schematic
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == b'\n')
        .map(|(i, _)| i + 1)
        .next()
        .unwrap_or(schematic.len());
    let height = schematic.len() / width;

    let mut numbers: Vec<SchematicNumber> = Vec::with_capacity(width * height / 3);
    let mut symbols: Vec<Symbol> = Vec::with_capacity(width * height);
    let mut current: Option<(u32, usize)> = None;

    for y in 0..height {
        for x in 0..width {
            let c = schematic[y * width + x];
            if c.is_ascii_digit() {
                let digit = (c - b'0') as u32;
                if let Some((number, start)) = current {
                    current = Some((number * 10 + digit, start));
                } else {
                    current = Some((digit, x));
                }
            } else {
                if let Some((number, start)) = current {
                    numbers.push(SchematicNumber {
                        value: number,
                        line: y as isize,
                        first: start as isize,
                        length: (x - start) as isize,
                    });
                    current = None;
                }
                if c != b'.' && c != b'\n' {
                    symbols.push(Symbol {
                        s: c,
                        line: y as isize,
                        col: x as isize,
                    })
                }
            }
        }
    }
    (numbers, symbols)
}

fn part_numbers_sum(numbers: &[SchematicNumber], symbols: &[Symbol]) -> u32 {
    numbers
        .iter()
        .filter(|number| {
            symbols
                .iter()
                .any(|symbol| number_symbol_adjacent(number, symbol))
        })
        .map(|number| number.value)
        .sum::<u32>()
}

fn number_symbol_adjacent(number: &SchematicNumber, symbol: &Symbol) -> bool {
    (symbol.line == number.line
        && (symbol.col == number.first - 1 || symbol.col == number.first + number.length))
        || ((symbol.line == number.line - 1 || symbol.line == number.line + 1)
            && symbol.col >= number.first - 1
            && symbol.col <= number.first + number.length)
}

fn gear_ratio_sum(numbers: &[SchematicNumber], symbols: &[Symbol]) -> u32 {
    symbols
        .iter()
        .filter(|symbol| symbol.s == b'*')
        .filter_map(|symbol| {
            let (n, gear_ratio) = numbers
                .iter()
                .filter(|number| number_symbol_adjacent(number, symbol))
                .fold((0usize, 1u32), |(n, gear_ratio), number| {
                    (n + 1, number.value * gear_ratio)
                });
            if n == 2 {
                Some(gear_ratio)
            } else {
                None
            }
        })
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    const SCHEMATIC: &[u8] = br#"467..114..
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

    #[test]
    fn part_number_sum_works_for_example() {
        // given
        let (numbers, symbols) = parse(SCHEMATIC);

        // when
        let result = part_numbers_sum(&numbers, &symbols);

        // then
        assert_eq!(result, 4361);
    }

    #[test]
    fn gear_ratio_sum_works_for_example() {
        // given
        let (numbers, symbols) = parse(SCHEMATIC);

        // when
        let result = gear_ratio_sum(&numbers, &symbols);

        // then
        assert_eq!(result, 467835);
    }
}
