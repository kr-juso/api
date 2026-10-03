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
`include_str!` 로 wasm에 포함되어 첫 요청 때 한 번 파싱된다.

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build
cd worker
cargo test
npx wrangler login
npx wrangler dev     # curl 'localhost:8787/v1/regcodes?regcodePattern=1111*&isIgnoreZero=true'
npx wrangler deploy
```

`wrangler.jsonc` 에 `api.juso.dev` 커스텀 도메인이 설정되어 있다. 배포하려면 juso.dev 존이 같은
Cloudflare 계정에 있어야 하고(네임서버를 Cloudflare로 변경), 기존 `api.juso.dev` DNS 레코드(GCP를
가리키는 CNAME/A)가 있으면 먼저 삭제해야 한다. 배포 후 확인:

```sh
curl -si 'https://api.juso.dev/v1/regcodes?regcodePattern=1111*' | grep -i x-cache   # 두 번째 요청부터 HIT
```
