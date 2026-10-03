// address.csv(TSV)에서 폐지되지 않은("존재") 행만 골라 OUT_DIR/regcodes.tsv 로 내보낸다.
// 폐지 행(전체의 절반 이상)을 wasm에 넣지 않고, 데이터 해시를 캐시 키 버전(DATA_VERSION)으로 쓴다.
use std::{env, fs, path::Path};

const SRC: &str = "../pkg/csv/internal/address.csv";

fn main() {
    println!("cargo:rerun-if-changed={SRC}");
    println!("cargo:rerun-if-changed=build.rs");

    let text = fs::read_to_string(SRC).expect("address.csv를 읽을 수 없습니다");

    let mut out = String::new();
    for line in text.lines() {
        let mut cols = line.split('\t');
        if let (Some(code), Some(name), Some(status)) = (cols.next(), cols.next(), cols.next())
            && status.trim_end() == "존재"
        {
            out.push_str(code);
            out.push('\t');
            out.push_str(name);
            out.push('\n');
        }
    }

    // FNV-1a 64bit
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in out.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("regcodes.tsv");
    fs::write(dest, out).unwrap();
    println!("cargo:rustc-env=DATA_VERSION={hash:016x}");
}
