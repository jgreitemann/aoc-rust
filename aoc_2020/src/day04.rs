use anyhow::anyhow;
use aoc_companion::prelude::*;
use itertools::Itertools;
use regex::Regex;
use std::{collections::HashMap, ops::RangeBounds};

pub(crate) struct Door<'input> {
    passports: Vec<HashMap<&'input str, &'input str>>,
}

impl<'input> Solution<'input> for Door<'input> {
    fn parse(input: &'input str) -> Result<Self> {
        parse_passports(input).map(|passports| Door { passports })
    }

    fn part1(&self) -> impl door::IntoResult {
        self.passports
            .iter()
            .filter(|p| is_passport_complete(p))
            .count()
    }

    fn part2(&self) -> impl door::IntoResult {
        self.passports
            .iter()
            .filter(|p| is_passport_valid(p))
            .count()
    }
}

fn parse_passports(input: &str) -> Result<Vec<HashMap<&str, &str>>> {
    input
        .split("\n\n")
        .map(|paragraph| {
            paragraph
                .split_whitespace()
                .map(|prop| {
                    prop.split_once(':')
                        .ok_or_else(|| anyhow!("missing colon delimiting property key from value"))
                })
                .try_collect()
        })
        .try_collect()
}

fn is_passport_complete(passport: &HashMap<&str, &str>) -> bool {
    const REQUIRED_KEYS: &[&str; 7] = &["byr", "iyr", "eyr", "hgt", "hcl", "ecl", "pid"];

    REQUIRED_KEYS.iter().all(|key| passport.contains_key(key))
}

fn is_passport_valid(passport: &HashMap<&str, &str>) -> bool {
    is_passport_complete(passport)
        && passport.iter().all(|(&key, &val)| match key {
            "byr" => is_year_valid(val, 1920..=2002),
            "iyr" => is_year_valid(val, 2010..=2020),
            "eyr" => is_year_valid(val, 2020..=2030),
            "hgt" => is_height_valid(val),
            "hcl" => is_hair_color_valid(val),
            "ecl" => is_eye_color_valid(val),
            "pid" => is_passport_id_valid(val),
            "cid" => true,
            _ => panic!("unknown key {key:?}"),
        })
}

fn is_year_valid(s: &str, range: impl RangeBounds<i32>) -> bool {
    s.parse::<i32>()
        .map(|year| range.contains(&year))
        .unwrap_or(false)
}

fn is_height_valid(s: &str) -> bool {
    if s.ends_with("cm") {
        (150..=193).contains(&s.trim_end_matches("cm").parse::<i32>().unwrap_or(0))
    } else if s.ends_with("in") {
        (59..=76).contains(&s.trim_end_matches("in").parse::<i32>().unwrap_or(0))
    } else {
        false
    }
}

fn is_hair_color_valid(s: &str) -> bool {
    let re = Regex::new("#[a-f0-9]{6}").unwrap();
    re.is_match(s)
}

fn is_eye_color_valid(s: &str) -> bool {
    ["amb", "blu", "brn", "gry", "grn", "hzl", "oth"].contains(&s)
}

fn is_passport_id_valid(s: &str) -> bool {
    s.len() == 9 && s.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod test {
    use itertools::assert_equal;

    use super::*;

    const EXAMPLE_INPUT: &str = "\
ecl:gry pid:860033327 eyr:2020 hcl:#fffffd
byr:1937 iyr:2017 cid:147 hgt:183cm

iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884
hcl:#cfa07d byr:1929

hcl:#ae17e1 iyr:2013
eyr:2024
ecl:brn pid:760753108 byr:1931
hgt:179cm

hcl:#cfa07d eyr:2025 pid:166559648
iyr:2011 ecl:brn hgt:59in";

    fn example_passports() -> Vec<HashMap<&'static str, &'static str>> {
        vec![
            HashMap::from([
                ("ecl", "gry"),
                ("pid", "860033327"),
                ("eyr", "2020"),
                ("hcl", "#fffffd"),
                ("byr", "1937"),
                ("iyr", "2017"),
                ("cid", "147"),
                ("hgt", "183cm"),
            ]),
            HashMap::from([
                ("iyr", "2013"),
                ("ecl", "amb"),
                ("cid", "350"),
                ("eyr", "2023"),
                ("pid", "028048884"),
                ("hcl", "#cfa07d"),
                ("byr", "1929"),
            ]),
            HashMap::from([
                ("hcl", "#ae17e1"),
                ("iyr", "2013"),
                ("eyr", "2024"),
                ("ecl", "brn"),
                ("pid", "760753108"),
                ("byr", "1931"),
                ("hgt", "179cm"),
            ]),
            HashMap::from([
                ("hcl", "#cfa07d"),
                ("eyr", "2025"),
                ("pid", "166559648"),
                ("iyr", "2011"),
                ("ecl", "brn"),
                ("hgt", "59in"),
            ]),
        ]
    }

    #[test]
    fn parse_example_passports() {
        assert_eq!(parse_passports(EXAMPLE_INPUT).unwrap(), example_passports());
    }

    #[test]
    fn complete_passports() {
        assert_equal(
            example_passports().iter().map(is_passport_complete),
            [true, false, true, false],
        );
    }

    #[test]
    fn invalid_passports() {
        let passports = parse_passports(
            "eyr:1972 cid:100
hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926

iyr:2019
hcl:#602927 eyr:1967 hgt:170cm
ecl:grn pid:012533040 byr:1946

hcl:dab227 iyr:2012
ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277

hgt:59cm ecl:zzz
eyr:2038 hcl:74454a iyr:2023
pid:3556412378 byr:2007",
        )
        .unwrap();

        passports
            .iter()
            .for_each(|p| assert!(!is_passport_valid(p)));
    }

    #[test]
    fn valid_passports() {
        let passports = parse_passports(
            "pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980
hcl:#623a2f

eyr:2029 ecl:blu cid:129 byr:1989
iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm

hcl:#888785
hgt:164cm byr:2001 iyr:2015 cid:88
pid:545766238 ecl:hzl
eyr:2022

iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719",
        )
        .unwrap();

        passports.iter().for_each(|p| assert!(is_passport_valid(p)));
    }

    #[test]
    fn validate_birth_year() {
        assert!(is_year_valid("2002", 1920..=2002));
        assert!(!is_year_valid("2003", 1920..=2002));
    }

    #[test]
    fn validate_height() {
        assert!(is_height_valid("60in"));
        assert!(is_height_valid("190cm"));
        assert!(!is_height_valid("190in"));
        assert!(!is_height_valid("190"));
    }

    #[test]
    fn validate_hair_color() {
        assert!(is_hair_color_valid("#123abc"));
        assert!(!is_hair_color_valid("#123abz"));
        assert!(!is_hair_color_valid("123abc"));
    }

    #[test]
    fn validate_eye_color() {
        assert!(is_eye_color_valid("brn"));
        assert!(!is_eye_color_valid("wat"));
    }

    #[test]
    fn validate_passport_id() {
        assert!(is_passport_id_valid("000000001"));
        assert!(is_passport_id_valid("012345678"));
        assert!(is_passport_id_valid("123456789"));
        assert!(!is_passport_id_valid("01234a678"));
        assert!(!is_passport_id_valid("0123456789"));
        assert!(!is_passport_id_valid("12345678"));
    }
}
