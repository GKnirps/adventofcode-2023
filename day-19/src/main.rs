use std::collections::HashMap;
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let (parts, workflows) = parse(&content)?;

    let rating_sum = accepted_rating(&parts, &workflows);
    println!("The sum of the ratings of accepted parts is {rating_sum}");

    let accepted_combinations = find_combinations(&workflows, LOWEST_PART, HIGHEST_PART, "in");
    println!("There are {accepted_combinations} combinations of ratings that are accepted by the elves' workflows.");

    Ok(())
}

const LOWEST_PART: Part = Part {
    extremely_cool: 1,
    musical: 1,
    aerodynamic: 1,
    shiny: 1,
};
const HIGHEST_PART: Part = Part {
    extremely_cool: 4001,
    musical: 4001,
    aerodynamic: 4001,
    shiny: 4001,
};

// let's take no risks with the integer size for the combinations here
fn find_combinations(
    workflows: &HashMap<&str, Workflow>,
    mut lower: Part,
    mut upper: Part,
    name: &str,
) -> u128 {
    // if the lower bound reached the upper bound anywhere, there are no possible combinations
    if lower.extremely_cool >= upper.extremely_cool
        || lower.musical >= upper.musical
        || lower.aerodynamic >= upper.aerodynamic
        || lower.shiny >= upper.shiny
    {
        return 0;
    }
    // ignore missing workflows, assume that parts that go to a missing worklflow are rejected
    let workflow = match workflows.get(name) {
        Some(wf) => wf,
        None => {
            return 0;
        }
    };
    let mut combinations: u128 = 0;
    for (cond_op, cond_value, outcome) in &workflow.rules {
        let cond_op = *cond_op;
        let cond_value = *cond_value;
        let outcome = *outcome;
        let (matched_lower, matched_upper) = match cond_op {
            Condition::XGt => {
                let orig_upper = upper;
                upper.extremely_cool = cond_value + 1;
                (
                    Part {
                        extremely_cool: cond_value + 1,
                        ..lower
                    },
                    orig_upper,
                )
            }
            Condition::XLt => {
                let orig_lower = lower;
                lower.extremely_cool = cond_value;
                (
                    orig_lower,
                    Part {
                        extremely_cool: cond_value,
                        ..upper
                    },
                )
            }
            Condition::MGt => {
                let orig_upper = upper;
                upper.musical = cond_value + 1;
                (
                    Part {
                        musical: cond_value + 1,
                        ..lower
                    },
                    orig_upper,
                )
            }
            Condition::MLt => {
                let orig_lower = lower;
                lower.musical = cond_value;
                (
                    orig_lower,
                    Part {
                        musical: cond_value,
                        ..upper
                    },
                )
            }
            Condition::AGt => {
                let orig_upper = upper;
                upper.aerodynamic = cond_value + 1;
                (
                    Part {
                        aerodynamic: cond_value + 1,
                        ..lower
                    },
                    orig_upper,
                )
            }
            Condition::ALt => {
                let orig_lower = lower;
                lower.aerodynamic = cond_value;
                (
                    orig_lower,
                    Part {
                        aerodynamic: cond_value,
                        ..upper
                    },
                )
            }
            Condition::SGt => {
                let orig_upper = upper;
                upper.shiny = cond_value + 1;
                (
                    Part {
                        shiny: cond_value + 1,
                        ..lower
                    },
                    orig_upper,
                )
            }
            Condition::SLt => {
                let orig_lower = lower;
                lower.shiny = cond_value;
                (
                    orig_lower,
                    Part {
                        shiny: cond_value,
                        ..upper
                    },
                )
            }
        };
        combinations += match outcome {
            Outcome::Reject => 0,
            Outcome::Accept => part_combinations(matched_lower, matched_upper),
            Outcome::SendTo(name) => {
                find_combinations(workflows, matched_lower, matched_upper, name)
            }
        };
    }
    combinations
        + match workflow.default {
            Outcome::Reject => 0,
            Outcome::Accept => part_combinations(lower, upper),
            Outcome::SendTo(name) => find_combinations(workflows, lower, upper, name),
        }
}

fn part_combinations(lower: Part, upper: Part) -> u128 {
    if lower.extremely_cool >= upper.extremely_cool
        || lower.musical >= upper.musical
        || lower.aerodynamic >= upper.aerodynamic
        || lower.shiny >= upper.shiny
    {
        return 0;
    }
    (upper.extremely_cool - lower.extremely_cool) as u128
        * (upper.musical - lower.musical) as u128
        * (upper.aerodynamic - lower.aerodynamic) as u128
        * (upper.shiny - lower.shiny) as u128
}

fn accepted_rating(parts: &[Part], workflows: &HashMap<&str, Workflow>) -> i64 {
    parts
        .iter()
        // yeah, I know, I just swallow possible errors here, it's bad.
        .filter(|part| check_part(part, workflows).unwrap_or(false))
        .map(|part| part.rating())
        .sum()
}

fn check_part(part: &Part, workflows: &HashMap<&str, Workflow>) -> Result<bool, String> {
    // Important: We have no loop detection here, so if the workflow results in a loop, we are
    // stuck. If that happens, we need to implement a loop detection after all
    let mut name: &str = "in";
    loop {
        let workflow = workflows
            .get(name)
            .ok_or_else(|| format!("unable to find workflow with name '{name}'"))?;
        name = match check_workflow(workflow, part) {
            Outcome::Accept => {
                return Ok(true);
            }
            Outcome::Reject => {
                return Ok(false);
            }
            Outcome::SendTo(n) => n,
        };
    }
}

fn check_workflow<'a>(workflow: &'a Workflow, part: &'a Part) -> Outcome<'a> {
    workflow
        .rules
        .iter()
        .filter(|(condition, value, _)| condition.apply(part, *value))
        .map(|(_, _, outcome)| *outcome)
        .next()
        .unwrap_or(workflow.default)
}

fn parse(input: &str) -> Result<(Vec<Part>, HashMap<&str, Workflow>), String> {
    let (workflows, parts) = input
        .split_once("\n\n")
        .ok_or("unable to split workflow list from part list")?;
    Ok((
        parts
            .lines()
            .map(parse_part)
            .collect::<Result<_, String>>()?,
        workflows
            .lines()
            .map(parse_workflow)
            .map(|workflow| {
                let workflow = workflow?;
                Ok((workflow.name, workflow))
            })
            .collect::<Result<HashMap<&str, Workflow>, String>>()?,
    ))
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Part {
    extremely_cool: i64,
    musical: i64,
    aerodynamic: i64,
    shiny: i64,
}

impl Part {
    fn rating(&self) -> i64 {
        self.extremely_cool + self.musical + self.aerodynamic + self.shiny
    }
}

fn parse_part(line: &str) -> Result<Part, String> {
    let (extremely_cool, rest) = line
        .strip_prefix("{x=")
        .ok_or_else(|| format!("invalid start of part in line '{line}'"))?
        .split_once(",m=")
        .ok_or_else(|| {
            format!("unable to split extremely cool value from the rest in line '{line}'")
        })?;
    let extremely_cool: i64 = extremely_cool
        .parse()
        .map_err(|e| format!("unable to parse extremely cool value '{extremely_cool}': {e}"))?;

    let (musical, rest) = rest
        .split_once(",a=")
        .ok_or_else(|| format!("unable to split musical value from the rest in line '{line}'"))?;
    let musical: i64 = musical
        .parse()
        .map_err(|e| format!("unable to parse musical value '{musical}': {e}"))?;

    let (aerodynamic, shiny) = rest.split_once(",s=").ok_or_else(|| {
        format!("unable to split aerodynamic value from shiny value in line '{line}'")
    })?;
    let aerodynamic: i64 = aerodynamic
        .parse()
        .map_err(|e| format!("unable to parse aerodynamic value '{aerodynamic}': {e}"))?;

    let shiny = shiny
        .strip_suffix('}')
        .ok_or_else(|| format!("expected just '}}' after shiny value in line '{line}'"))?
        .parse()
        .map_err(|e| format!("unable to parse shiny value: {e}"))?;

    Ok(Part {
        extremely_cool,
        musical,
        aerodynamic,
        shiny,
    })
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Condition {
    XGt,
    XLt,
    MGt,
    MLt,
    AGt,
    ALt,
    SGt,
    SLt,
}

impl Condition {
    fn apply(&self, left: &Part, right: i64) -> bool {
        match self {
            Condition::XGt => left.extremely_cool > right,
            Condition::XLt => left.extremely_cool < right,
            Condition::MGt => left.musical > right,
            Condition::MLt => left.musical < right,
            Condition::AGt => left.aerodynamic > right,
            Condition::ALt => left.aerodynamic < right,
            Condition::SGt => left.shiny > right,
            Condition::SLt => left.shiny < right,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
enum Outcome<'a> {
    SendTo(&'a str),
    Reject,
    Accept,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Workflow<'a> {
    name: &'a str,
    rules: Vec<(Condition, i64, Outcome<'a>)>,
    default: Outcome<'a>,
}

fn parse_workflow(line: &str) -> Result<Workflow, String> {
    let (name, rules) = line
        .split_once('{')
        .ok_or_else(|| format!("missing rules for workflow '{line}'"))?;

    let (rules, default) = rules.rsplit_once(',').unwrap_or(("", rules));
    let default = parse_outcome(
        default
            .strip_suffix('}')
            .ok_or_else(|| "missing '}' at the end of the workflow.".to_string())?,
    );

    let rules: Vec<(Condition, i64, Outcome)> = rules
        .split(',')
        .map(|rule| {
            let (condition, outcome) = rule
                .split_once(':')
                .ok_or_else(|| format!("missing outcome in rule '{rule}'"))?;
            let outcome = parse_outcome(outcome);

            let (cond_op, cond_v) = parse_condition(condition)?;

            Ok((cond_op, cond_v, outcome))
        })
        .collect::<Result<_, String>>()?;

    Ok(Workflow {
        name,
        rules,
        default,
    })
}

fn parse_outcome(s: &str) -> Outcome {
    match s {
        "A" => Outcome::Accept,
        "R" => Outcome::Reject,
        _ => Outcome::SendTo(s),
    }
}

fn parse_condition(s: &str) -> Result<(Condition, i64), String> {
    if let Some((variable, value)) = s.split_once('<') {
        let condition = match variable {
            "x" => Ok(Condition::XLt),
            "m" => Ok(Condition::MLt),
            "a" => Ok(Condition::ALt),
            "s" => Ok(Condition::SLt),
            _ => Err(format!("unknown category '{variable}' in condition")),
        }?;
        let value: i64 = value
            .parse()
            .map_err(|e| format!("unable to parse value '{value}' in condition: {e}"))?;
        Ok((condition, value))
    } else if let Some((variable, value)) = s.split_once('>') {
        let condition = match variable {
            "x" => Ok(Condition::XGt),
            "m" => Ok(Condition::MGt),
            "a" => Ok(Condition::AGt),
            "s" => Ok(Condition::SGt),
            _ => Err(format!("unknown category '{variable}' in condition")),
        }?;
        let value: i64 = value
            .parse()
            .map_err(|e| format!("unable to parse value '{value}' in condition: {e}"))?;
        Ok((condition, value))
    } else {
        Err(format!("could not find operator in rule '{s}'"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = r#"px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}
"#;

    #[test]
    fn accepted_rating_works_for_example() {
        // given
        let (parts, workflows) = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let rating = accepted_rating(&parts, &workflows);

        // then
        assert_eq!(rating, 19114);
    }

    #[test]
    fn find_combinations_works_for_example() {
        // given
        let (_, workflows) = parse(EXAMPLE).expect("expected successful parsing");

        // when
        let n = find_combinations(&workflows, LOWEST_PART, HIGHEST_PART, "in");

        // then
        assert_eq!(n, 167409079868000);
    }
}
