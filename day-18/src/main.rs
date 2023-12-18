use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let instructions = parse(&content)?;

    let lagoon_area = dig(&instructions);
    println!("The lagoon can hold {lagoon_area} m³ lava");

    let instructions = fix_instructions(&instructions)?;
    let large_lagoon_area = dig(&instructions);
    println!("The lagoon with correctedinstructions can hold {large_lagoon_area} m³ lava");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Dir {
    Up,
    Right,
    Down,
    Left,
}

impl Dir {
    fn outer_angle(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Dir::Up, Dir::Right)
                | (Dir::Down, Dir::Left)
                | (Dir::Right, Dir::Down)
                | (Dir::Left, Dir::Up)
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Instruction {
    dir: Dir,
    length: u32,
    color: u32,
}

fn parse(input: &str) -> Result<Vec<Instruction>, String> {
    input.lines().map(parse_instruction).collect()
}

fn parse_instruction(line: &str) -> Result<Instruction, String> {
    let (dir, rest) = line
        .split_once(' ')
        .ok_or_else(|| format!("unable to split direction from rest in line '{line}'"))?;
    let dir = match dir {
        "U" => Ok(Dir::Up),
        "R" => Ok(Dir::Right),
        "D" => Ok(Dir::Down),
        "L" => Ok(Dir::Left),
        _ => Err(format!("unknown direction '{dir}' in line '{line}'")),
    }?;

    let (length, color) = rest
        .split_once(" (#")
        .ok_or_else(|| format!("unable to split length fromcolor in line '{line}'"))?;
    let length = length
        .parse::<u32>()
        .map_err(|e| format!("unable to parse length '{length}' in line '{line}': {e}"))?;
    let color = u32::from_str_radix(
        color
            .strip_suffix(')')
            .ok_or_else(|| format!("missing closing paranthesis after color in line '{line}'"))?,
        16,
    )
    .map_err(|e| format!("unable to parse color '{color}' in line '{line}': {e}"))?;

    Ok(Instruction { dir, length, color })
}

fn fix_instruction(instruction: &Instruction) -> Result<Instruction, String> {
    let dir = match instruction.color % 16 {
        0 => Ok(Dir::Right),
        1 => Ok(Dir::Down),
        2 => Ok(Dir::Left),
        3 => Ok(Dir::Up),
        n => Err(format!("unknown direction: '{n}'")),
    }?;

    let length = instruction.color / 16;

    Ok(Instruction {
        dir,
        length,
        color: instruction.color,
    })
}

fn fix_instructions(instructions: &[Instruction]) -> Result<Vec<Instruction>, String> {
    instructions.iter().map(fix_instruction).collect()
}

fn dig(instructions: &[Instruction]) -> i64 {
    if instructions.is_empty() {
        return 0;
    }

    let mut pos: (i64, i64) = (0, 0);
    let mut previous_dir = instructions[instructions.len() - 1].dir;
    let mut corners: Vec<(i64, i64)> = Vec::with_capacity(instructions.len() + 1);
    corners.push(pos);

    // assumption: trenches are dug clockwise
    for pair in instructions.windows(2) {
        let Instruction {
            dir,
            length,
            color: _,
        } = pair[0];
        let next_dir = pair[1].dir;
        match dir {
            Dir::Up => {
                pos.1 -= length as i64 - 1
                    + if previous_dir.outer_angle(dir) { 1 } else { 0 }
                    + if dir.outer_angle(next_dir) { 1 } else { 0 };
                corners.push(pos);
            }
            Dir::Right => {
                pos.0 += length as i64 - 1
                    + if previous_dir.outer_angle(dir) { 1 } else { 0 }
                    + if dir.outer_angle(next_dir) { 1 } else { 0 };
                corners.push(pos);
            }
            Dir::Down => {
                pos.1 += length as i64 - 1
                    + if previous_dir.outer_angle(dir) { 1 } else { 0 }
                    + if dir.outer_angle(next_dir) { 1 } else { 0 };
                corners.push(pos);
            }
            Dir::Left => {
                pos.0 -= length as i64 - 1
                    + if previous_dir.outer_angle(dir) { 1 } else { 0 }
                    + if dir.outer_angle(next_dir) { 1 } else { 0 };
                corners.push(pos);
            }
        }
        previous_dir = dir;
    }

    corners.push((0, 0));

    (corners
        .windows(2)
        .map(|win| {
            let (x1, y1) = win[0];
            let (x2, y2) = win[1];
            (y1 + y2) * (x1 - x2)
        })
        .sum::<i64>()
        / 2)
    .abs()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)
"#;

    #[test]
    fn dig_works_for_example() {
        // given
        let instructions = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let dug_out = dig(&instructions);

        // then
        assert_eq!(dug_out, 62);
    }

    #[test]
    fn dig_works_for_fixed_example() {
        // given
        let instructions = fix_instructions(&parse(EXAMPLE).expect("expected successful parsing"))
            .expect("expected successful correction");

        // when
        let dug_out = dig(&instructions);

        // then
        assert_eq!(dug_out, 952408144115);
    }
}
