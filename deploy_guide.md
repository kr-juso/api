# 서버 배포 가이드

작성자: YeongCheon Kim

## generate gRPC code

```sh
protoc \
    --include_imports \
    --include_source_info \
    --proto_path=. \
    -I./api/protobuf \
    -I./api/protobuf/googleapis \
    --go_out=./internal/grpc \
    --go-grpc_out=./internal/grpc \
    --descriptor_set_out=api_descriptor.pb \
    api/protobuf/juso/juso.proto
```

## build command

```sh
go build -o juso_regcode cmd/main.go
docker build -t asia-docker.pkg.dev/kr-juso/juso/api-grpc .
docker push asia-docker.pkg.dev/kr-juso/juso/api-grpc
gcloud run deploy grpc-server --image asia-docker.pkg.dev/kr-juso/juso/api-grpc --platform managed --region asia-northeast3
```

## endpoint deploy command

[guide 참고](https://cloud.google.com/endpoints/docs/grpc/get-started-cloud-run#deploy_esp)

```sh
gcloud endpoints services deploy api_descriptor.pb api_config.yaml

./gcloud_build_image.sh \
    -s grpc-server-mkvo6j4wsq-du.a.run.app \
    -c 2021-11-06r0 \
    -p kr-juso

gcloud run deploy grpc-proxy-server \
  --image="gcr.io/kr-juso/endpoints-runtime-serverless:2.32.0-grpc-server-mkvo6j4wsq-du.a.run.app-2021-11-06r0" \
  --set-env-vars=ESPv2_ARGS=^++^--cors_preset=basic++--cors_allow_headers="*" \
  --allow-unauthenticated \
  --platform managed \
  --region asia-northeast3 \
  --project kr-juso
```

## Cloudflare Workers 배포 (GCP Cloud Run + ESPv2 대체)

Workers는 gRPC 서버를 직접 호스팅할 수 없어서, ESPv2가 하던 REST 변환(`GET /v1/regcodes`)을
`worker/` 의 Rust(workers-rs) Worker가 직접 처리한다. 데이터(`pkg/csv/internal/address.csv`)는
`build.rs` 가 폐지되지 않은 행만 골라 wasm에 포함하고, 첫 요청 때 한 번 파싱한다. 데이터 해시는
캐시 키에 들어가므로 데이터를 갱신해 배포하면 이전 캐시는 자동으로 무효화된다.

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --locked --version 0.8.7   # Cargo.lock의 worker 0.8.x와 맞춘다
cd worker
cargo test
npx wrangler login
npx wrangler dev     # curl 'localhost:8787/v1/regcodes?regcodePattern=1111*&isIgnoreZero=true'
npx wrangler deploy
```

### 도메인 전환 (`api.juso.dev`)

`wrangler.jsonc` 에 `api.juso.dev` 커스텀 도메인이 설정되어 있다. juso.dev 존이 같은 Cloudflare
계정에 있어야 한다(네임서버를 Cloudflare로 변경). 기존 `api.juso.dev` DNS 레코드(GCP를 가리키는
CNAME/A)가 있으면 커스텀 도메인 배포가 충돌하므로 아래 순서로 전환한다.

1. 전환 하루 전쯤 기존 레코드의 TTL을 60초로 낮추고, 레코드 값을 기록해 둔다(롤백용).
2. Cloud Run(`grpc-server`, `grpc-proxy-server`)은 **전환이 끝나고 안정화될 때까지 지우지 않는다.**
3. 기존 `api.juso.dev` 레코드를 삭제하고 **바로** `npx wrangler deploy` 를 실행한다. 이 사이가
   짧은 중단 구간이다(TTL을 낮춰 두면 최소화된다). 배포가 실패하면 4번 롤백을 한다.
4. 롤백: 기록해 둔 레코드를 다시 만들면 GCP로 돌아간다. 배포가 성공했어도 문제가 있으면 같은 방법으로
   되돌린 뒤 `npx wrangler delete` 로 Worker를 내린다.
5. 확인 (두 번째 요청부터 HIT):

```sh
curl -si 'https://api.juso.dev/v1/regcodes?regcodePattern=1111*' | grep -i x-cache
```

Cache API는 workers.dev 에서는 동작하지 않으므로 `workers_dev` 는 꺼 두었다. 확인이 끝나고 며칠
문제가 없으면 Cloud Run 서비스와 Endpoints 설정을 정리한다.
