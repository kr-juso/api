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

#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    if req.method() == Method::Options {
        return Response::empty()?.with_cors(&cors());
    }

    let url = req.url()?;
    if req.method() != Method::Get || url.path() != "/v1/regcodes" {
        return Response::error("Not Found", 404)?.with_cors(&cors());
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
    Response::from_json(&body)?.with_cors(&cors())
}
