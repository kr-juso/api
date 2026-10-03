use serde::Serialize;
use std::sync::OnceLock;

// build.rs가 폐지되지 않은 행만 골라 만든 "code\tname\n" 목록
const REGCODES_TSV: &str = include_str!(concat!(env!("OUT_DIR"), "/regcodes.tsv"));

/// 데이터 내용의 해시. 데이터가 바뀌면 캐시 키가 바뀌어 이전 캐시가 무효화된다.
pub const DATA_VERSION: &str = env!("DATA_VERSION");

/// 법정동 코드의 길이. 이보다 많은 글자를 요구하는 패턴은 어떤 코드와도 일치할 수 없다.
const CODE_LEN: usize = 10;

#[derive(Serialize, Debug, PartialEq)]
pub struct Regcode {
    pub code: &'static str,
    pub name: &'static str,
}

fn regcodes() -> &'static [Regcode] {
    static DATA: OnceLock<Vec<Regcode>> = OnceLock::new();
    DATA.get_or_init(|| {
        REGCODES_TSV
            .lines()
            .filter_map(|line| {
                let (code, name) = line.split_once('\t')?;
                Some(Regcode { code, name })
            })
            .collect()
    })
}

/// 연속된 '*'를 하나로 합친다. 어떤 코드와도 일치할 수 없는 패턴이면 None.
/// ('*'를 뺀 글자가 코드 길이보다 많거나 패턴이 비어 있는 경우)
/// 정규화된 패턴 길이는 최대 2 * CODE_LEN + 1 이므로 매칭 비용이 요청 입력 길이에 비례하지 않는다.
pub fn normalize_pattern(raw: &str) -> Option<String> {
    let mut out = String::with_capacity(raw.len().min(2 * CODE_LEN + 1));
    let mut literals = 0;
    for c in raw.chars() {
        if c == '*' {
            if out.ends_with('*') {
                continue;
            }
        } else {
            literals += 1;
            if literals > CODE_LEN {
                return None;
            }
        }
        out.push(c);
    }
    (!out.is_empty()).then_some(out)
}

/// '*'(0자 이상), '?'(1자) 와일드카드. 패턴 전체가 문자열 전체와 일치해야 한다.
/// 코드는 ASCII 숫자라서 바이트 단위로 비교한다.
pub fn is_match(s: &[u8], p: &[u8]) -> bool {
    let (mut si, mut pi) = (0, 0);
    let (mut star, mut mark) = (None, 0);
    while si < s.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == s[si]) {
            si += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == b'*' {
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
    p[pi..].iter().all(|&c| c == b'*')
}

/// 첫 '*' 위치부터 두 자리가 "00"이면 true ("1111*"에서 "1111000000" 제외용)
fn is_star_starts_with_zero(code: &[u8], pattern: &[u8]) -> bool {
    match pattern.iter().position(|&c| c == b'*') {
        Some(star) => code.get(star..star + 2) == Some(b"00".as_slice()),
        None => false,
    }
}

/// `pattern`은 normalize_pattern을 거친 값이어야 한다.
pub fn list_regcodes(pattern: &str, ignore_zero: bool) -> Vec<&'static Regcode> {
    let p = pattern.as_bytes();
    regcodes()
        .iter()
        .filter(|r| {
            let code = r.code.as_bytes();
            is_match(code, p) && !(ignore_zero && is_star_starts_with_zero(code, p))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(pattern: &str, ignore_zero: bool) -> Vec<&'static str> {
        let p = normalize_pattern(pattern).expect("matchable pattern");
        list_regcodes(&p, ignore_zero).iter().map(|r| r.code).collect()
    }

    #[test]
    fn wildcard() {
        assert!(is_match(b"1111000000", b"1111*"));
        assert!(is_match(b"1111000000", b"*000000"));
        assert!(is_match(b"1111000000", b"11??000000"));
        assert!(is_match(b"1111000000", b"1*1*0"));
        assert!(!is_match(b"1111000000", b"2111*"));
        assert!(!is_match(b"1111000000", b""));
    }

    #[test]
    fn question_mark_with_ignore_zero() {
        // '?'는 정확히 한 글자: 시군구 코드 "1111"로 시작하는 읍면동만
        let all = codes("1111??????", false);
        assert!(all.contains(&"1111000000"));
        assert!(!codes("1111*", true).contains(&"1111000000"));
        assert!(codes("1111*", true).contains(&"1111010100"));
    }

    #[test]
    fn seoul_children() {
        let all = codes("11*", false);
        assert!(all.contains(&"1100000000"));
        assert!(all.iter().all(|c| c.starts_with("11")));
    }

    #[test]
    fn ignore_zero_excludes_parent() {
        assert!(codes("1111*", false).contains(&"1111000000"));
        assert!(!codes("1111*", true).contains(&"1111000000"));
    }

    #[test]
    fn abolished_and_header_rows_are_excluded() {
        let all = codes("*", false);
        assert_eq!(all.len(), 20551);
        // 폐지된 "서울특별시 종로구 창신1동"
        assert!(!all.contains(&"1111090100"));
        assert!(all.iter().all(|c| c.len() == 10 && c.bytes().all(|b| b.is_ascii_digit())));
    }

    #[test]
    fn normalize() {
        assert_eq!(normalize_pattern("1111**").as_deref(), Some("1111*"));
        assert_eq!(normalize_pattern("*1***1*").as_deref(), Some("*1*1*"));
        assert_eq!(normalize_pattern(""), None);
        // 코드 길이(10)보다 많은 글자는 일치할 수 없다
        assert_eq!(normalize_pattern("11111111111"), None);
        assert_eq!(normalize_pattern(&"1".repeat(6000)), None);
        // '*'는 글자 수에 세지 않지만 연속된 '*'는 합쳐지므로 길이는 유한하다
        let long = "1*".repeat(6000);
        assert_eq!(normalize_pattern(&long), None);
        assert!(normalize_pattern(&"*".repeat(6000)).unwrap() == "*");
    }
}
