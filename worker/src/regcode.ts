import data from "./regcodes.json" with { type: "json" };

export interface Regcode {
  code: string;
  name: string;
}

const regcodes: Regcode[] = (data as [string, string][]).map(([code, name]) => ({ code, name }));

// '*'(0자 이상), '?'(1자) 와일드카드 매칭. 패턴 전체가 문자열 전체와 일치해야 한다.
export function isMatch(s: string, p: string): boolean {
  let si = 0, pi = 0, star = -1, mark = 0;
  while (si < s.length) {
    if (pi < p.length && (p[pi] === "?" || p[pi] === s[si])) {
      si++; pi++;
    } else if (pi < p.length && p[pi] === "*") {
      star = pi++; mark = si;
    } else if (star !== -1) {
      pi = star + 1; si = ++mark;
    } else {
      return false;
    }
  }
  while (pi < p.length && p[pi] === "*") pi++;
  return pi === p.length;
}

// 첫 '*' 위치부터 두 자리가 "00"이면 true ("1111*" 패턴에서 "1111000000" 제외용)
function isStarStartsWithZero(code: string, pattern: string): boolean {
  const star = pattern.indexOf("*");
  if (star === -1) return false;
  return code.slice(star, star + 2) === "00";
}

export function listRegcodes(pattern: string, ignoreZero: boolean): Regcode[] {
  return regcodes.filter(
    (r) => isMatch(r.code, pattern) && !(ignoreZero && isStarStartsWithZero(r.code, pattern)),
  );
}
