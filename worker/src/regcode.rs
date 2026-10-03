use serde::Serialize;
use std::sync::OnceLock;

// 폐지여부 컬럼이 "존재"인 행만 사용한다. (code \t name \t 폐지여부)
const ADDRESS_TSV: &str = include_str!("../../pkg/csv/internal/address.csv");

#[derive(Serialize, Debug, PartialEq)]
pub struct Regcode {
    pub code: &'static str,
    pub name: &'static str,
}

fn regcodes() -> &'static [Regcode] {
    static DATA: OnceLock<Vec<Regcode>> = OnceLock::new();
    DATA.get_or_init(|| {
        ADDRESS_TSV
            .lines()
            .filter_map(|line| {
                let mut cols = line.split('\t');
                let (code, name, status) = (cols.next()?, cols.next()?, cols.next()?);
                (status.trim_end() == "존재").then_some(Regcode { code, name })
            })
            .collect()
    })
}

/// '*'(0자 이상), '?'(1자) 와일드카드. 패턴 전체가 문자열 전체와 일치해야 한다.
pub fn is_match(s: &str, p: &str) -> bool {
    let (s, p): (Vec<char>, Vec<char>) = (s.chars().collect(), p.chars().collect());
    let (mut si, mut pi) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            si += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            pi += 1;
            mark = si;
        } else if let Some(st) = star {
            pi = st + 1;
            mark += 1;
            si = mark;
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// 첫 '*' 위치부터 두 자리가 "00"이면 true ("1111*"에서 "1111000000" 제외용)
fn is_star_starts_with_zero(code: &str, pattern: &str) -> bool {
    match pattern.chars().position(|c| c == '*') {
        Some(star) => code.chars().skip(star).take(2).eq("00".chars()),
        None => false,
    }
}

pub fn list_regcodes(pattern: &str, ignore_zero: bool) -> Vec<&'static Regcode> {
    regcodes()
        .iter()
        .filter(|r| {
            is_match(r.code, pattern) && !(ignore_zero && is_star_starts_with_zero(r.code, pattern))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard() {
        assert!(is_match("1111000000", "1111*"));
        assert!(is_match("1111000000", "*000000"));
        assert!(is_match("1111000000", "11??000000"));
        assert!(!is_match("1111000000", "2111*"));
        assert!(!is_match("1111000000", ""));
    }

    #[test]
    fn seoul_children() {
        let all = list_regcodes("11*", false);
        assert!(all.iter().any(|r| r.code == "1100000000"));
        assert!(all.iter().all(|r| r.code.starts_with("11")));
    }

    #[test]
    fn ignore_zero_excludes_parent() {
        let a = list_regcodes("1111*", false);
        let b = list_regcodes("1111*", true);
        assert!(a.iter().any(|r| r.code == "1111000000"));
        assert!(!b.iter().any(|r| r.code == "1111000000"));
    }

    #[test]
    fn no_abolished_or_empty() {
        assert!(list_regcodes("*", false).iter().all(|r| !r.code.is_empty()));
    }
}
