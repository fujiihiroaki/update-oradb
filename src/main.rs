//! # SQLファイルを読取り、一行ごとのSQL文をOracleに実行するプロジェクトです。
extern crate oracle;

use dotenv::dotenv;
use oracle::{Connection, Error};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use fern::Dispatch;

/// このプロジェクトのエントリーポイントです。
/// Oracleに接続し、SQL文を実行します。
///  //で始まる行はコメントとして無視されます。
fn main() -> Result<(), Error> {
    // .envファイルの読取
    dotenv().ok();

    // ロガーの初期化
    init_logger();

    // DB用環境変数の読取
    let db_host = env::var("DB_HOST").expect("DB_HOST not set");
    println!("HOST:{}", db_host);
    let db_port = env::var("DB_PORT").expect("DB_PORT not set");
    println!("PORT:{}", db_port);
    let db_user = env::var("DB_USER").expect("DB_USER not set");
    println!("USER:{}", db_user);
    let db_password = env::var("DB_PASSWORD").expect("DB_PASSWORD not set");
    println!("PASSWORD:{}", db_password);
    let db_name = env::var("DB_NAME").expect("DB_NAME not set");
    println!("DATABASE:{}", db_name);

    log::info!("host={} port={} user={} password={} dbname={}",
            db_host, db_port, db_user, db_password, db_name
        );

    // oracleに接続
    log::info!("try connecting to the database");
    let client = match Connection::connect(db_user, db_password,
                                           &format!("//{}:{}/{}", db_host, db_port, db_name).as_str(),
    ) {
        Ok(client) => client,
        Err(e) => {
            log::error!("{}", e);
            return Err(e);
        }
    };
    log::info!("Success connecting to the database");

    // SQL文の読取
    let file_name = env::var("FILE_NAME").expect("FILE_NAME must be set");
    let file = File::open(file_name).expect("file not found");

    // ファイルの読取
    let buf = BufReader::new(file);
    // SQL文の初期化
    let mut sql_string = String::from("");
    for line in buf.lines() {
        // ファイルから一行ずつ読取る
        let sql = &line.unwrap();
        // //で始まる行はコメントとして無視。空白行も無視
        if substring(&sql, 0, 2) != "//" && !trim(sql).is_empty() {
            if sql.contains(";") {
                sql_string.push_str(sql);
                println!("SQL:{}", sql_string.as_str());
                // SQL文の実行
                let stmt = match client.execute(&sql_string, &[]) {
                    Ok(stmt) => stmt,
                    Err(e) => {
                        log::error!("{}", e);
                        return Err(e);
                    }
                };
                if stmt.row_count().unwrap() > 0 {
                    log::info!("Affected rows:{}", stmt.row_count().unwrap());
                }
                log::debug!("Success SQL:{}", sql_string.as_str());

                // SQL文の初期化
                sql_string = String::from("");
            } else {
                // ;がない場合は改行を追加し、SQL文に追加
                let return_code = "\n";
                sql_string.push_str(sql);
                sql_string.push_str(&return_code);
            }
        }
    }

    Ok(())
}

/// 対象文字列の開始位置から長さ分の部分文字列を取得します
///
/// * `s` - 対象の文字列
/// * `start` - 開始位置
/// * `length` - 長さ
fn substring(s: &str, start: usize, length: usize) -> &str {
    if length == 0 {
        return "";
    }

    let mut ci = s.char_indices();
    let start_byte = match ci.nth(start) {
        Some(i) => i.0,
        None => return "",
    };

    match ci.nth(length - 1) {
        Some(j) => &s[start_byte..j.0],
        None => &s[start_byte..],
    }
}

/// 半角/全角の文字列をトリムします
///
/// * `s` - 対象の文字列
fn trim(s: &str) -> &str {
    let text = s.trim();
    text.trim_end_matches(|c: char| c.is_whitespace())
}

/// ロガーの初期化
///
/// ログは標準出力とlog.txtに出力されます
fn init_logger() {
    let logfile_name = env::var("LOG_FILE_NAME").expect("LOG_FILE_NAME must be set");

    let base_config = Dispatch::new();

    let file_config = create_log_config()
        .chain(fern::log_file(&logfile_name).unwrap());

    let stdout_config = create_log_config()
        .chain(std::io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()
        .unwrap();
}

/// ログの設定を作成します
///
fn create_log_config() -> Dispatch {
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} {}:[{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        }).level(log::LevelFilter::Info)
}