# Rust 공식 이미지를 기반으로 사용
FROM rust:latest

# WebAssembly 도구 설치
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-pack

# 작업 디렉토리 설정
WORKDIR /app

# 프로젝트 파일 복사
COPY . .

# 기본 명령어
CMD ["bash"]