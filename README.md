# Duplicate File Finder

![Rust](https://img.shields.io/badge/rust-1.73+-orange.svg)
![Crates.io](https://img.shields.io/crates/v/clap.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

**Duplicate File Finder**는 지정된 디렉터리와 그 하위 모든 폴더를 스캔하여 **내용이 완전히 동일한** 중복 파일을 찾아내는 강력하고 효율적인 커맨드라인 유틸리티입니다. Rust로 작성되어 빠른 속도와 메모리 안정성을 보장합니다.

사진, 문서, 코드, 동영상 등 어떤 종류의 파일이든 상관없이 정확하게 중복을 찾아내어 디스크 공간을 정리하고 파일을 체계적으로 관리하는 데 도움을 줍니다.

---

## 🎯 주요 기능 (Key Features)

-   **내용 기반 탐색**: 파일 이름이 다르더라도, 파일의 내용을 바이트 단위로 비교하여 진짜 중복 파일을 찾아냅니다.
-   **재귀적 스캔**: 지정된 폴더뿐만 아니라, 그 안에 있는 모든 하위 폴더까지 샅샅이 검색합니다.
-   **강력한 필터링**: 특정 파일 이름(`--file-filter report.txt`)이나 확장자(`--file-filter '*.log'`)를 지정하여 검색 대상을 좁힐 수 있습니다.
-   **성능 최적화**: 대용량 파일과 수많은 파일을 효율적으로 처리하기 위해 2단계 탐색 전략을 사용합니다.
    1.  **빠른 크기 비교**: 내용이 같은 파일은 크기도 반드시 같다는 점을 이용해, 먼저 파일 크기별로 그룹화하여 비교 대상을 대폭 줄입니다.
    2.  **정확한 해시 비교**: 크기가 같은 파일 그룹에 대해서만 SHA-256 해시를 계산하여 내용이 100% 동일한지 최종 확인합니다.
-   **메모리 효율성**: 대용량 파일을 처리할 때도 파일을 통째로 메모리에 올리지 않고, 스트림 방식으로 조금씩 읽어 처리하므로 메모리 사용량이 매우 낮습니다.
-   **사용하기 쉬운 CLI**: `clap`을 기반으로 한 명확하고 직관적인 커맨드라인 인터페이스를 제공합니다.

---

## 🛠️ 설치 (Installation)

이 프로젝트를 사용하기 위해서는 Rust 컴파일러와 Cargo 패키지 매니저가 필요합니다. [rust-lang.org](https://www.rust-lang.org/tools/install)에서 `rustup`을 설치하세요.

1.  **Git 리포지토리 클론:**
    ```bash
    git clone https://github.com/<YourUsername>/duplicate-finder.git
    cd duplicate-finder
    ```

2.  **릴리즈 모드로 빌드:**
    `--release` 플래그는 최종 실행 파일에 대한 최적화를 활성화하여 최고의 성능을 보장합니다.
    ```bash
    cargo build --release
    ```

    빌드가 완료되면, 실행 파일은 `./target/release/duplicate_finder`에 생성됩니다.

---

## 🚀 사용 방법 (Usage)

실행 파일은 필수적으로 `--root-folder` 옵션을 요구하며, 선택적으로 `--file-filter` 옵션을 사용할 수 있습니다.

```bash
./target/release/duplicate_finder --root-folder <검색할_폴더_경로> [OPTIONS]
```

### 옵션 (Options)

| 짧은 이름 | 긴 이름         | 설명                                                                 | 필수 여부 |
| :-------- | :-------------- | :------------------------------------------------------------------- | :-------- |
| `-r`      | `--root-folder` | 중복 파일 검색을 시작할 최상위 폴더 경로입니다.                        | **필수**  |
| `-f`      | `--file-filter` | 검색 대상을 특정 파일로 한정합니다. 와일드카드 확장자(`'*.ext'`)를 지원합니다. | 선택      |
| `-h`      | `--help`        | 도움말 메시지를 출력합니다.                                          | -         |
| `-V`      | `--version`     | 프로그램 버전을 출력합니다.                                            | -         |

### 사용 예시

1.  **현재 폴더(`.`)와 모든 하위 폴더에서 중복 파일 찾기:**
    ```bash
    ./target/release/duplicate_finder -r .
    ```

2.  **사용자의 `Downloads` 폴더에서 중복 파일 찾기:**
    ```bash
    ./target/release/duplicate_finder --root-folder ~/Downloads
    ```

3.  **`Projects` 폴더에서 이름이 `main.rs`인 중복 파일만 찾기:**
    ```bash
    ./target/release/duplicate_finder -r ~/Projects -f main.rs
    ```

4.  **`Photos` 폴더에서 확장자가 `jpg`인 중복 사진 파일만 찾기:**
    > ⚠️  쉘(Shell)이 `*` 문자를 자체적으로 해석하는 것을 방지하기 위해, 와일드카드 패턴은 작은 따옴표(`' '`)로 감싸는 것이 안전합니다.
    ```bash
    ./target/release/duplicate_finder --root-folder /mnt/Photos --file-filter '*.jpg'
    ```

### 출력 결과 예시

```
🔍 '/path/to/your/Projects' 폴더에서 확장자가 '.rs'인 파일을 대상으로 중복 파일을 검색합니다...

✨ 2개의 중복 파일 그룹을 찾았습니다:

--- 그룹 1 (총 3개 파일) ---
  - /path/to/your/Projects/project-alpha/src/main.rs
  - /path/to/your/Projects/project-beta/src/main.rs
  - /path/to/your/Projects/backup/main_v1.rs

--- 그룹 2 (총 2개 파일) ---
  - /path/to/your/Projects/project-gamma/src/utils.rs
  - /path/to/your/Projects/common/lib/helpers.rs

```

---

## 📜 라이선스 (License)

이 프로젝트는 [MIT 라이선스](LICENSE)에 따라 배포됩니다. 자유롭게 사용하고 수정할 수 있습니다.