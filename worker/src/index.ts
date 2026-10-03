import { listRegcodes } from "./regcode.ts";

// ESPv2의 --cors_preset=basic --cors_allow_headers=* 와 동일
const CORS = {
  "access-control-allow-origin": "*",
  "access-control-allow-methods": "GET, POST, PUT, PATCH, DELETE, OPTIONS",
  "access-control-allow-headers": "*",
};

const json = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json", ...CORS },
  });

export default {
  async fetch(request: Request): Promise<Response> {
    if (request.method === "OPTIONS") return new Response(null, { status: 204, headers: CORS });

    const url = new URL(request.url);
    if (request.method !== "GET" || url.pathname !== "/v1/regcodes") {
      return json({ code: 5, message: "Not Found" }, 404);
    }

    // gRPC transcoding 호환: regcodePattern / regcode_pattern, isIgnoreZero / is_ignore_zero
    const q = url.searchParams;
    const pattern = q.get("regcodePattern") ?? q.get("regcode_pattern") ?? "";
    const ignoreZero = (q.get("isIgnoreZero") ?? q.get("is_ignore_zero")) === "true";

    return json({ regcodes: listRegcodes(pattern, ignoreZero) });
  },
};
