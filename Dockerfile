FROM ubuntu:20.04

COPY juso_regcode juso_regcode

ENTRYPOINT ["./juso_regcode"]
