// ==============================================================================
//  Duplicate File Finder v0.3
// ------------------------------------------------------------------------------
//  기능:
//  - 지정된 폴더와 모든 하위 폴더를 재귀적으로 탐색합니다.
//  - 파일 내용이 완전히 동일한 중복 파일들을 찾아냅니다.
//  - 텍스트, 이미지, 영상 등 모든 종류의 파일을 처리할 수 있습니다.
//  - 커맨드라인 인자를 통해 필수적으로 검색할 루트 폴더를 지정받습니다.
//  - 선택적으로 파일 이름이나 확장자 필터를 적용할 수 있습니다.
//
//  최적화 전략:
//  성능을 위해 2단계 필터링 방식을 사용합니다.
//  1. 파일 크기로 1차 그룹화: 파일 내용이 같다면 크기는 반드시 같다는 점을
//     이용하여, 비용이 큰 해시 계산 전에 비교 대상을 대폭 줄입니다.
//  2. SHA-256 해시로 2차 그룹화: 크기가 같은 파일 그룹에 대해서만 파일 내용의
//     고유한 서명(해시)을 계산하여 진짜 중복 파일을 찾아냅니다.
// ==============================================================================

// --- 외부 라이브러리 및 표준 라이브러리 모듈 가져오기 (use statements) ---

// std::collections::HashMap: 키-값 쌍을 저장하는 해시 맵 자료구조.
// 파일 크기 -> 파일 경로 리스트, 파일 해시 -> 파일 경로 리스트를 만드는 데 사용됩니다.
use std::collections::HashMap;

// std::env: 현재 환경에 대한 정보를 다루는 모듈.
// 여기서는 기본 폴더를 설정하기 위해 현재 작업 디렉터리를 가져오는 데 사용했었지만,
// 이제는 필수 인자로 변경되어 직접적인 사용은 없습니다. (미래 확장을 위해 남겨둘 수 있음)
// use std::env;

// std::ffi::OsStr: 운영체제 네이티브 문자열(OS String)을 다루기 위한 타입.
// 파일 이름이나 경로를 비교할 때, 유니코드(String)로 변환할 수 없는 문자가
// 포함될 수 있는 경우에도 안전하게 처리하기 위해 사용합니다.
use std::ffi::OsStr;

// std::fs::File: 파일 시스템의 파일을 다루기 위한 구조체.
use std::fs::File;

// std::io::{...}: 입출력(I/O) 작업을 위한 모듈.
// - io: Result<T, io::Error> 와 같은 공통 I/O 타입을 사용하기 위함.
// - BufReader: 파일을 효율적으로 읽기 위한 버퍼 리더.
// - Read: 데이터를 읽어오는 기능을 제공하는 트레이트(trait).
use std::io::{self, BufReader, Read};

// std::path::{Path, PathBuf}: 파일 시스템 경로를 다루기 위한 타입.
// - Path: 경로에 대한 빌려온(borrowed) 슬라이스. 변경 불가능.
// - PathBuf: 경로를 소유(owned)하며 변경 가능한 문자열 버퍼.
use std::path::{Path, PathBuf};

// 외부 라이브러리 `clap`: 커맨드라인 인자 파싱을 위한 강력한 도구.
// derive 기능을 통해 구조체 정의만으로 손쉽게 CLI를 만들 수 있습니다.
use clap::Parser;

// 외부 라이브러리 `sha2`: SHA-256 해시 알고리즘 구현체.
// - Digest: 모든 해시 함수가 구현해야 하는 공통 트레이트.
// - Sha256: SHA-256 해시 계산기.
use sha2::{Digest, Sha256};

// 외부 라이브러리 `walkdir`: 디렉터리를 재귀적으로 탐색하는 편리한 도구.
use walkdir::WalkDir;

/// 파일 시스템에서 중복된 파일을 찾아 그룹화하여 출력하는 프로그램
// `#[derive(Parser, Debug)]`: clap의 derive 매크로를 사용하여 이 구조체를 CLI 파서로 만듭니다.
// Debug 트레이트는 `{:#?}` 등을 통해 구조체를 보기 좋게 출력하는 데 필요합니다.
#[derive(Parser, Debug)]
// `#[command(...)]`: 프로그램의 버전, 설명 등 메타데이터를 설정합니다. `--help` 시 출력됩니다.
#[command(version, about, long_about = None)]
struct Args {
    /// [필수] 검색을 시작할 루트 폴더 경로.
    // `#[arg(...)]`: 각 필드에 대한 CLI 옵션 설정을 정의합니다.
    // - short: 짧은 옵션 이름 (e.g., -r)
    // - long: 긴 옵션 이름 (e.g., --root-folder)
    // - value_name: 도움말에 표시될 값의 이름 (e.g., <FOLDER_PATH>)
    // 이 필드는 Option<T>가 아니므로, clap은 자동으로 필수(required) 인자로 처리합니다.
    #[arg(short, long, value_name = "FOLDER_PATH")]
    root_folder: PathBuf,

    /// 검색할 파일 이름을 지정합니다 (예: "report.txt", "*.log").
    // 이 필드는 Option<String> 이므로, clap은 자동으로 선택적(optional) 인자로 처리합니다.
    #[arg(short, long, value_name = "FILENAME_PATTERN")]
    file_filter: Option<String>,
}

/// 파일 이름 필터링의 다양한 모드를 정의하는 열거형(enum).
/// 문자열을 직접 사용하는 것보다 타입-세이프(type-safe)하고,
/// `match` 구문을 통해 코드를 명확하게 만들 수 있어 좋은 설계 패턴입니다.
enum FilterMode {
    /// 필터를 적용하지 않음 (모든 파일 대상).
    None,
    /// 정확한 파일 이름으로 필터링.
    ByExactName(String),
    /// 파일 확장자로 필터링.
    ByExtension(String),
}

/// 프로그램의 메인 진입점.
fn main() {
    // 1. 커맨드라인 인자 파싱
    // `Args::parse()`는 사용자가 입력한 인자를 분석하여 `Args` 구조체를 채웁니다.
    // 만약 사용자가 `--help`를 입력했거나 필수 인자(--root-folder)를 누락했다면,
    // clap이 자동으로 도움말/오류 메시지를 출력하고 프로그램을 종료시켜 줍니다.
    let args = Args::parse();

    // 2. 검색할 루트 폴더 설정
    // `root_folder`는 필수 인자이므로 이제 Option을 해제할 필요 없이 직접 사용합니다.
    // args.root_folder는 PathBuf 타입의 소유권을 가집니다.
    let root_path = args.root_folder;

    // 3. 파일 이름 필터 모드 결정
    // 사용자가 입력한 `--file-filter` 값을 분석하여 `FilterMode`를 결정합니다.
    let filter_mode = match args.file_filter {
        // 필터가 제공되지 않았다면 FilterMode::None
        None => FilterMode::None,
        // 필터 문자열이 제공되었다면
        Some(filter_str) => {
            // `strip_prefix("*.")`를 사용하여 문자열이 "*."로 시작하는지 확인합니다.
            // 맞다면, 확장자 필터 모드로 설정하고 "*. " 부분을 제외한 나머지(확장자)를 저장합니다.
            if let Some(ext) = filter_str.strip_prefix("*.") {
                FilterMode::ByExtension(ext.to_string())
            } else {
                // "*." 패턴이 아니라면, 정확한 이름 필터 모드로 설정합니다.
                FilterMode::ByExactName(filter_str)
            }
        }
    };
    
    // 4. 사용자에게 현재 검색 설정을 알려줌 (사용자 경험 개선)
    // `root_path`는 `main` 함수가 소유하고 있으므로, 다른 함수에는 빌려주어야 합니다(&).
    print_search_info(&root_path, &filter_mode);

    // 5. 중복 파일 찾기 핵심 로직 실행
    // `find_duplicates` 함수는 파일 I/O 작업을 수행하므로 실패할 수 있습니다. (io::Result)
    // 따라서 `match` 구문을 사용하여 성공(Ok)과 실패(Err) 케이스를 모두 처리합니다.
    match find_duplicates(&root_path, &filter_mode) {
        // 성공 시, 찾은 중복 파일 그룹(duplicates)을 처리합니다.
        Ok(duplicates) => {
            if duplicates.is_empty() {
                println!("✅ 중복된 파일을 찾지 못했습니다.");
            } else {
                println!("\n✨ {}개의 중복 파일 그룹을 찾았습니다:\n", duplicates.len());
                // 결과 출력 함수를 호출합니다.
                print_duplicates(duplicates);
            }
        }
        // 실패 시, 표준 에러(stderr)에 오류 메시지를 출력합니다.
        Err(e) => {
            eprintln!("오류 발생: {}", e);
        }
    }
}

/// 현재 검색 설정을 요약하여 화면에 출력하는 헬퍼 함수.
fn print_search_info(root: &Path, filter: &FilterMode) {
    let filter_desc = match filter {
        FilterMode::None => "모든 파일".to_string(),
        FilterMode::ByExactName(name) => format!("이름이 '{}'인 파일", name),
        FilterMode::ByExtension(ext) => format!("확장자가 '.{}'인 파일", ext),
    };
    // `.display()` 메소드는 Path/PathBuf를 운영체제에 맞는 방식으로 출력 가능하게 만들어줍니다.
    println!(
        "🔍 '{}' 폴더에서 {}을(를) 대상으로 중복 파일을 검색합니다...",
        root.display(),
        filter_desc
    );
}

/// 지정된 경로에서 필터 조건에 맞는 중복 파일 그룹을 찾아 반환합니다.
fn find_duplicates(root: &Path, filter_mode: &FilterMode) -> io::Result<Vec<Vec<PathBuf>>> {
    // --- 1단계: 파일 크기로 그룹화 (빠른 1차 필터링) ---
    // `u64` (파일 크기)를 키로, `Vec<PathBuf>` (파일 경로 리스트)를 값으로 가집니다.
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    // `WalkDir::new(root)`는 지정된 폴더를 재귀적으로 탐색하는 이터레이터(iterator)를 생성합니다.
    for entry in WalkDir::new(root)
        .into_iter()
        // `.filter_map(|e| e.ok())`: 탐색 중 권한 오류 등으로 발생할 수 있는 에러(Err)는
        // 무시하고 성공적인 결과(Ok)만 다음 단계로 넘깁니다.
        .filter_map(|e| e.ok())
        // `.filter(|e| e.file_type().is_file())`: 디렉터리가 아닌 파일만 필터링합니다.
        .filter(|e| e.file_type().is_file())
        // `.filter(|e| ...)`: 사용자가 지정한 이름/확장자 필터를 적용합니다.
        .filter(|e| passes_filter(e.path(), filter_mode))
    {
        // 파일의 메타데이터(크기, 수정 시간 등)를 가져옵니다. `?` 연산자는 에러 발생 시
        // 함수에서 즉시 에러를 반환하게 해주는 문법적 설탕(syntactic sugar)입니다.
        let metadata = entry.metadata()?;
        // 크기가 0인 파일은 내용이 없으므로 중복으로 간주하지 않습니다.
        if metadata.len() > 0 {
            // `entry(key).or_default()`: 해시 맵에서 `metadata.len()` 키를 찾습니다.
            // - 키가 존재하면: 해당 키의 값(파일 경로 리스트)에 접근합니다.
            // - 키가 없으면: 새로운 빈 벡터 `Vec::new()`를 생성하여 삽입하고 접근합니다.
            // 이어서 `.push(...)`로 현재 파일 경로를 리스트에 추가합니다.
            files_by_size
                .entry(metadata.len())
                .or_default()
                .push(entry.into_path());
        }
    }
    
    // --- 2단계: 파일 내용의 해시로 그룹화 (정밀 2차 필터링) ---
    // 최종 중복 그룹들을 담을 벡터입니다.
    let mut final_duplicates = Vec::new();
    // 1단계에서 만들어진 `files_by_size` 맵에서, 값이 2개 이상인 (즉, 중복 가능성이 있는)
    // 그룹에 대해서만 반복문을 실행합니다.
    for (_size, paths) in files_by_size.into_iter().filter(|(_, p)| p.len() > 1) {
        let mut files_by_hash: HashMap<String, Vec<PathBuf>> = HashMap::new();
        // 크기가 같은 파일 리스트(paths) 내에서 각 파일의 해시를 계산합니다.
        for path in paths {
            match calculate_hash(&path) {
                Ok(hash) => {
                    files_by_hash.entry(hash).or_default().push(path);
                }
                // 해시 계산 중 오류 발생 시 경고 메시지만 출력하고 계속 진행합니다.
                Err(e) => {
                    eprintln!("경고: '{}' 파일의 해시를 계산할 수 없습니다: {}", path.display(), e);
                }
            }
        }
        
        // 해시 맵에서도 해시 값이 같은 파일이 2개 이상인 그룹만 찾아
        // 최종 중복 리스트 `final_duplicates`에 추가합니다.
        for (_hash, duplicate_paths) in files_by_hash.into_iter().filter(|(_, p)| p.len() > 1) {
            final_duplicates.push(duplicate_paths);
        }
    }

    // 모든 작업이 성공적으로 끝났으므로, 최종 결과를 `Ok`로 감싸서 반환합니다.
    Ok(final_duplicates)
}

/// 주어진 파일 경로가 필터 조건을 만족하는지 여부를 반환하는 헬퍼 함수.
fn passes_filter(path: &Path, filter_mode: &FilterMode) -> bool {
    // `filter_mode`의 각 경우에 따라 다른 로직을 수행합니다.
    match filter_mode {
        FilterMode::None => true, // 필터가 없으면 무조건 true.
        FilterMode::ByExactName(name) => {
            // `path.file_name()`은 파일 이름을 `Option<&OsStr>`으로 반환합니다.
            // 파일 이름이 존재하고, 그 값이 주어진 이름과 같을 때만 true.
            path.file_name() == Some(OsStr::new(name))
        },
        FilterMode::ByExtension(ext) => {
            // `path.extension()`은 확장자를 `Option<&OsStr>`으로 반환합니다.
            // 확장자가 존재하고, 그 값이 주어진 확장자와 같을 때만 true.
            path.extension() == Some(OsStr::new(ext))
        },
    }
}

/// 파일의 SHA-256 해시 값을 계산하여 16진수 문자열로 반환합니다.
/// 파일 내용을 바이트 단위로 읽으므로 텍스트, 바이너리 구분 없이 모든 파일에 적용 가능합니다.
fn calculate_hash(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    // `BufReader`는 파일을 읽을 때 시스템 호출 횟수를 줄여 성능을 향상시킵니다.
    // 특히 대용량 파일을 처리할 때 효과적입니다.
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096]; // 4KB (4096 bytes) 크기의 버퍼.

    // `loop`를 사용하여 파일을 버퍼 크기만큼씩 반복해서 읽습니다.
    loop {
        // `reader.read`는 버퍼에 데이터를 채우고 읽은 바이트 수를 반환합니다.
        let bytes_read = reader.read(&mut buffer)?;
        // 읽은 바이트 수가 0이면 파일의 끝에 도달했다는 의미이므로 루프를 탈출합니다.
        if bytes_read == 0 {
            break;
        }
        // `hasher.update`로 읽은 데이터 조각을 해시 계산기에 주입합니다.
        // 슬라이스 `&buffer[..bytes_read]`를 사용하여 버퍼에서 실제로 읽은 만큼만 전달합니다.
        hasher.update(&buffer[..bytes_read]);
    }

    // `hasher.finalize()`로 최종 해시 결과를 얻고,
    // `format!("{:x}", ...)`를 통해 16진수(hexadecimal) 문자열로 변환하여 반환합니다.
    Ok(format!("{:x}", hasher.finalize()))
}

/// 찾은 중복 파일 그룹들을 형식에 맞게 화면에 출력하는 헬퍼 함수.
fn print_duplicates(duplicates: Vec<Vec<PathBuf>>) {
    // `iter().enumerate()`를 사용하면 인덱스(i)와 값(group)을 동시에 얻을 수 있습니다.
    for (i, group) in duplicates.iter().enumerate() {
        // 그룹 번호는 1부터 시작하도록 i + 1을 사용합니다.
        println!("--- 그룹 {} (총 {}개 파일) ---", i + 1, group.len());
        for path in group {
            println!("  - {}", path.display());
        }
        // 그룹 간 구분을 위해 빈 줄을 하나 추가합니다.
        println!();
    }
}