use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let v_number = hash_sum(&content);
    println!("The verification number is {v_number}");

    let operations = parse(&content)?;
    let power = focusing_power(&operations);
    println!("The focusing power of the lens configuration is {power}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Op {
    Dash,
    Eq(u32),
}

fn parse(input: &str) -> Result<Vec<(&str, Op)>, String> {
    input
        .split(',')
        .map(|op| parse_operation(op.trim()))
        .collect()
}

fn parse_operation(input: &str) -> Result<(&str, Op), String> {
    if let Some(label) = input.strip_suffix('-') {
        Ok((label, Op::Dash))
    } else if let Some((label, focal_length)) = input.split_once('=') {
        Ok((
            label,
            Op::Eq(
                focal_length
                    .parse::<u32>()
                    .map_err(|e| format!("unable to parse focal length '{focal_length}': {e}"))?,
            ),
        ))
    } else {
        Err(format!("Unable to parse instruction '{input}'"))
    }
}

fn hash_sum(input: &str) -> u32 {
    input.split(',').map(hash).sum()
}

fn hash(input: &str) -> u32 {
    input
        .trim()
        .bytes()
        .fold(0u32, |current, c| ((current + c as u32) * 17) % 256)
}

fn run_operations<'a>(operations: &[(&'a str, Op)]) -> Vec<Vec<(&'a str, u32)>> {
    // no efficient data structures for now, maybe this is fast enough
    let mut boxes: Vec<Vec<(&str, u32)>> = (0..256).map(|_| vec![]).collect();

    for (label, op) in operations {
        let box_i = hash(label) as usize;
        match op {
            Op::Eq(focal_length) => {
                if let Some(i) = boxes[box_i].iter().position(|(l, _)| l == label) {
                    boxes[box_i][i].1 = *focal_length;
                } else {
                    boxes[box_i].push((label, *focal_length));
                }
            }
            Op::Dash => {
                if let Some(i) = boxes[box_i].iter().position(|(l, _)| l == label) {
                    boxes[box_i].remove(i);
                }
            }
        }
    }
    boxes
}

fn focusing_power(operations: &[(&str, Op)]) -> usize {
    let boxes = run_operations(operations);
    boxes
        .iter()
        .enumerate()
        .flat_map(|(box_i, lens_box)| {
            lens_box
                .iter()
                .enumerate()
                .map(move |(lens_i, (_, focal_length))| {
                    (box_i + 1) * (lens_i + 1) * *focal_length as usize
                })
        })
        .sum()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7\n";

    #[test]
    fn hash_sum_works_for_example() {
        // given
        let input = EXAMPLE;

        // when
        let sum = hash_sum(input);

        // then
        assert_eq!(sum, 1320);
    }

    #[test]
    fn focusing_power_works_for_example() {
        // given
        let operations = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let power = focusing_power(&operations);

        // then
        assert_eq!(power, 145);
    }
}
