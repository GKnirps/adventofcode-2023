use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;

    let rows = parse(&content)?;

    let arrangements = sum_arrangements(&rows);
    println!("The sum of the possible arrangements of all rows is {arrangements}");

    Ok(())
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Spring {
    Op,
    Dmg,
    Unk,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Row {
    spring_conditions: Vec<(Spring, usize)>,
    damaged_groups: Vec<usize>,
}

fn parse(input: &str) -> Result<Vec<Row>, String> {
    input.lines().map(parse_row).collect()
}

fn parse_row(line: &str) -> Result<Row, String> {
    let (springs, groups) = line
        .split_once(' ')
        .ok_or_else(|| format!("unable to split line '{line}'"))?;

    let (mut spring_conditions, leftover_count, leftover_cond) = springs.chars().try_fold(
        (Vec::with_capacity(groups.len()), 0, Spring::Op),
        |(mut conditions, count, condition), c| {
            let new_condition = match c {
                '.' => Ok(Spring::Op),
                '#' => Ok(Spring::Dmg),
                '?' => Ok(Spring::Unk),
                _ => Err(format!("unknown spring condition '{c}' in '{springs}'")),
            }?;
            if condition == new_condition {
                Ok::<(Vec<(Spring, usize)>, usize, Spring), String>((
                    conditions,
                    count + 1,
                    condition,
                ))
            } else {
                if count != 0 {
                    conditions.push((condition, count));
                }
                Ok((conditions, 1, new_condition))
            }
        },
    )?;

    if leftover_count > 0 {
        spring_conditions.push((leftover_cond, leftover_count));
    }

    let damaged_groups = groups
        .split(',')
        .map(|s| {
            s.parse::<usize>()
                .map_err(|e| format!("unable to parse '{s}' as number in line '{line}': {e}"))
        })
        .collect::<Result<Vec<usize>, String>>()?;
    Ok(Row {
        spring_conditions,
        damaged_groups,
    })
}

fn sum_arrangements(rows: &[Row]) -> usize {
    rows.iter().map(count_arrangements).sum()
}

fn count_arrangements(row: &Row) -> usize {
    // Again, I have no good idea how to approach this, so I start with some brute forcing
    let unknown_count = row
        .spring_conditions
        .iter()
        .map(
            |(cond, count)| {
                if *cond == Spring::Unk {
                    *count
                } else {
                    0
                }
            },
        )
        .sum::<usize>();
    let damaged_count = row
        .spring_conditions
        .iter()
        .map(
            |(cond, count)| {
                if *cond == Spring::Dmg {
                    *count
                } else {
                    0
                }
            },
        )
        .sum::<usize>();
    let required_damaged_count = row.damaged_groups.iter().sum::<usize>();

    if damaged_count > required_damaged_count {
        return 0;
    }

    (0..2u64.pow(unknown_count as u32))
        .filter(|pattern| arrangement_matches(row, *pattern))
        .count()
}

// this is overly complicated, but my brain does not work right now
fn arrangement_matches(row: &Row, mut pattern: u64) -> bool {
    let mut springs_i = 0;
    let mut in_unknown_i = 0;
    for group in &row.damaged_groups {
        let mut damage_count: usize = 0;
        'springs: while springs_i < row.spring_conditions.len() {
            match row.spring_conditions[springs_i].0 {
                Spring::Op => {
                    if damage_count > 0 {
                        break;
                    }
                    springs_i += 1;
                    continue;
                }
                Spring::Dmg => {
                    damage_count += row.spring_conditions[springs_i].1;
                }
                Spring::Unk => {
                    for i in in_unknown_i..row.spring_conditions[springs_i].1 {
                        let operational = pattern & 1 == 0;
                        pattern = pattern >> 1;
                        in_unknown_i = i + 1;
                        if operational {
                            if damage_count > 0 {
                                break 'springs;
                            }
                            continue;
                        } else {
                            damage_count += 1;
                        }
                    }
                    in_unknown_i = 0;
                }
            }
            springs_i += 1;
        }

        if *group != damage_count {
            return false;
        }
    }
    pattern == 0
        && !row
            .spring_conditions
            .iter()
            .skip(springs_i)
            .any(|(condition, _)| *condition == Spring::Dmg)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1
"#;

    #[test]
    fn count_arrangements_works_for_examples() {
        assert_eq!(
            count_arrangements(&parse_row("???.### 1,1,3").expect("expected successful parsing")),
            1
        );
        assert_eq!(
            count_arrangements(
                &parse_row(".??..??...?##. 1,1,3").expect("expected successful parsing")
            ),
            4
        );
        assert_eq!(
            count_arrangements(
                &parse_row("?#?#?#?#?#?#?#? 1,3,1,6").expect("expected successful parsing")
            ),
            1
        );
        assert_eq!(
            count_arrangements(
                &parse_row("????.#...#... 4,1,1").expect("expected successful parsing")
            ),
            1
        );
        assert_eq!(
            count_arrangements(
                &parse_row("????.######..#####. 1,6,5").expect("expected successful parsing")
            ),
            4
        );
        assert_eq!(
            count_arrangements(
                &parse_row("?###???????? 3,2,1").expect("expected successful parsing")
            ),
            10
        );
    }

    #[test]
    fn sum_arrangements_works_for_example() {
        // given
        let rows = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let sum = sum_arrangements(&rows);

        // then
        assert_eq!(sum, 21);
    }
}
