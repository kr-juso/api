mod regcode;

use serde::Serialize;
use worker::*;

#[derive(Serialize)]
struct ListRegcodesResponse {
    regcodes: Vec<&'static regcode::Regcode>,
}

// 법정동 데이터는 배포 때만 바뀌고, 캐시 키에 데이터 버전이 들어가므로 하루 캐시해도 안전하다.
const CACHE_CONTROL: &str = "public, max-age=86400";
const ALLOW: &str = "GET, HEAD, OPTIONS";

// ESPv2의 --cors_preset=basic --cors_allow_headers=* 와 동일
fn cors() -> Cors {
    Cors::new()
        .with_origins(["*"])
        .with_methods([Method::Get, Method::Post, Method::Put, Method::Patch, Method::Delete, Method::Options])
        .with_allowed_headers(["*"])
}

/// gRPC transcoding 호환: regcodePattern / regcode_pattern, isIgnoreZero / is_ignore_zero
fn parse_query(url: &Url) -> (String, bool) {
    let param = |camel: &str, snake: &str| {
        url.query_pairs()
            .find(|(k, _)| k == camel || k == snake)
            .map(|(_, v)| v.into_owned())
    };
    let pattern = param("regcodePattern", "regcode_pattern").unwrap_or_default();
    let ignore_zero = param("isIgnoreZero", "is_ignore_zero").as_deref() == Some("true");
    (pattern, ignore_zero)
}

/// 정규화된 (pattern, ignore_zero)와 데이터 버전으로 만든 캐시 키.
/// 파라미터 순서/이름 별칭/불필요한 파라미터와 상관없이 같은 질의는 같은 키가 된다.
fn cache_key(pattern: &str, ignore_zero: bool) -> String {
    let mut url = Url::parse("https://cache.juso.dev/v1/regcodes").expect("static url");
    url.query_pairs_mut()
        .append_pair("v", regcode::DATA_VERSION)
        .append_pair("p", pattern)
        .append_pair("z", if ignore_zero { "1" } else { "0" });
    url.into()
}

fn json_response(regcodes: Vec<&'static regcode::Regcode>, cors: &Cors) -> Result<Response> {
    let mut resp = Response::from_json(&ListRegcodesResponse { regcodes })?.with_cors(cors)?;
    resp.headers_mut().set("cache-control", CACHE_CONTROL)?;
    Ok(resp)
}

/// 캐시 응답은 헤더가 immutable이므로 복사해서 x-cache를 붙인다.
/// 상태코드(206 등)와 본문 스트림은 그대로 넘긴다.
fn tag_cache_hit(hit: Response) -> Result<Response> {
    let headers = Headers::new();
    for (name, value) in hit.headers().entries() {
        headers.set(&name, &value)?;
    }
    headers.set("x-cache", "HIT")?;

    let status = hit.status_code();
    let (builder, body) = hit.into_parts();
    Ok(builder.with_status(status).with_headers(headers).body(body))
}

#[event(fetch)]
async fn fetch(req: Request, _env: Env, ctx: Context) -> Result<Response> {
    let cors = cors();

    if req.method() == Method::Options {
        return Response::empty()?.with_cors(&cors);
    }

    let url = req.url()?;
    if url.path() != "/v1/regcodes" {
        return Response::error("Not Found", 404)?.with_cors(&cors);
    }
    if !matches!(req.method(), Method::Get | Method::Head) {
        let mut resp = Response::error("Method Not Allowed", 405)?.with_cors(&cors)?;
        resp.headers_mut().set("allow", ALLOW)?;
        return Ok(resp);
    }

    let (raw_pattern, ignore_zero) = parse_query(&url);

    // 어떤 코드와도 일치할 수 없는 패턴은 스캔/캐시 없이 바로 빈 결과를 돌려준다.
    let Some(pattern) = regcode::normalize_pattern(&raw_pattern) else {
        return json_response(Vec::new(), &cors);
    };

    // 캐시는 최적화일 뿐이므로 읽기/쓰기에 실패해도 계산해서 응답한다.
    let key = cache_key(&pattern, ignore_zero);
    match Cache::default().get(key.as_str(), false).await {
        Ok(Some(hit)) => return tag_cache_hit(hit),
        Ok(None) => {}
        Err(e) => console_error!("cache get failed: {e}"),
    }

    let mut resp = json_response(regcode::list_regcodes(&pattern, ignore_zero), &cors)?;

    match resp.cloned() {
        Ok(to_cache) => ctx.wait_until(async move {
            if let Err(e) = Cache::default().put(key.as_str(), to_cache).await {
                console_error!("cache put failed: {e}");
            }
        }),
        Err(e) => console_error!("response clone failed: {e}"),
    }

    resp.headers_mut().set("x-cache", "MISS")?;
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_of(query: &str) -> String {
        let url = Url::parse(&format!("https://api.juso.dev/v1/regcodes?{query}")).unwrap();
        let (raw, z) = parse_query(&url);
        cache_key(&regcode::normalize_pattern(&raw).unwrap(), z)
    }

    #[test]
    fn equivalent_queries_share_a_cache_key() {
        let base = key_of("regcodePattern=1111*");
        assert_eq!(base, key_of("regcode_pattern=1111*"));
        assert_eq!(base, key_of("regcodePattern=1111%2A"));
        assert_eq!(base, key_of("regcodePattern=1111**"));
        assert_eq!(base, key_of("_=123&regcodePattern=1111*&isIgnoreZero=false"));
        assert_eq!(base, key_of("isIgnoreZero=no&regcodePattern=1111*"));
    }

    #[test]
    fn different_queries_get_different_keys() {
        let base = key_of("regcodePattern=1111*");
        assert_ne!(base, key_of("regcodePattern=1111*&isIgnoreZero=true"));
        assert_ne!(base, key_of("regcodePattern=2111*"));
    }

    #[test]
    fn query_aliases() {
        let url = Url::parse("https://x/v1/regcodes?is_ignore_zero=true&regcode_pattern=11*").unwrap();
        assert_eq!(parse_query(&url), ("11*".to_string(), true));
        let url = Url::parse("https://x/v1/regcodes").unwrap();
        assert_eq!(parse_query(&url), (String::new(), false));
    }

    #[test]
    fn cache_key_contains_data_version() {
        assert!(key_of("regcodePattern=11*").contains(regcode::DATA_VERSION));
    }
}
