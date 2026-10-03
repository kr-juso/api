mod regcode;

use serde::Serialize;
use worker::*;

#[derive(Serialize)]
struct ListRegcodesResponse {
    regcodes: Vec<&'static regcode::Regcode>,
}

// ESPv2의 --cors_preset=basic --cors_allow_headers=* 와 동일
fn cors() -> Cors {
    Cors::new()
        .with_origins(["*"])
        .with_methods([Method::Get, Method::Post, Method::Put, Method::Patch, Method::Delete, Method::Options])
        .with_allowed_headers(["*"])
}

// 법정동 데이터는 배포 때만 바뀌므로 하루 캐시한다.
const CACHE_CONTROL: &str = "public, max-age=86400";

#[event(fetch)]
async fn fetch(req: Request, _env: Env, ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return Response::empty()?.with_cors(&cors());
    }

    let url = req.url()?;
    if req.method() != Method::Get || url.path() != "/v1/regcodes" {
        return Response::error("Not Found", 404)?.with_cors(&cors());
    }

    let cache = Cache::default();
    if let Some(mut hit) = cache.get(&req, false).await? {
        // 캐시에서 꺼낸 응답의 헤더는 immutable이라 복사해서 x-cache를 붙인다.
        let headers = hit.headers().clone();
        let mut resp = Response::from_bytes(hit.bytes().await?)?.with_headers(headers);
        resp.headers_mut().set("x-cache", "HIT")?;
        return Ok(resp);
    }

    // gRPC transcoding 호환: regcodePattern / regcode_pattern, isIgnoreZero / is_ignore_zero
    let param = |camel: &str, snake: &str| {
        url.query_pairs()
            .find(|(k, _)| k == camel || k == snake)
            .map(|(_, v)| v.into_owned())
    };
    let pattern = param("regcodePattern", "regcode_pattern").unwrap_or_default();
    let ignore_zero = param("isIgnoreZero", "is_ignore_zero").as_deref() == Some("true");

    let body = ListRegcodesResponse {
        regcodes: regcode::list_regcodes(&pattern, ignore_zero),
    };
    let mut resp = Response::from_json(&body)?.with_cors(&cors())?;
    resp.headers_mut().set("cache-control", CACHE_CONTROL)?;

    let to_cache = resp.cloned()?;
    ctx.wait_until(async move {
        let _ = Cache::default().put(&req, to_cache).await;
    });

    resp.headers_mut().set("x-cache", "MISS")?;
    Ok(resp)
}
