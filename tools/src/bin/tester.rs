#![allow(non_snake_case)]

use std::io::{Read, Write};
use std::process::Stdio;
use tools::*; // genやcompute_scoreなどを使うために必要

// ここに`exec`関数を定義する！
fn exec(p: &mut std::process::Child, _verbose: bool) -> Result<i64, String> {
    // 子プロセスの標準入出力へのハンドルを取得
    let mut stdin = p.stdin.take().ok_or_else(|| "failed to open stdin".to_string())?;
    let mut stdout = p.stdout.take().ok_or_else(|| "failed to open stdout".to_string())?;

    // テストケースを生成
    let input = gen(0, None, None);

    // テストケースを子プロセスに書き込む
    let input_str = format!("{}", input);
    stdin.write_all(input_str.as_bytes()).map_err(|e| e.to_string())?;
    drop(stdin);

    // 子プロセスからの出力を読み取る
    let mut output_str = String::new();
    stdout.read_to_string(&mut output_str).map_err(|e| e.to_string())?;

    // 出力をパースする
    let output = parse_output(&input, &output_str)?;

    // スコアを計算する
    let (score, err) = compute_score(&input, &output.actions);
    if !err.is_empty() {
        return Err(err);
    }

    // スコアを返す
    Ok(score)
}

// `main`関数は`exec`を呼び出すだけ
fn main() {
    if std::env::args().len() < 2 {
        eprintln!("Usage: {} <command> [<args>...]", std::env::args().nth(0).unwrap());
        return;
    }
    let (command, args) = (std::env::args().nth(1).unwrap(), std::env::args().skip(2).collect::<Vec<_>>());
    let mut p = std::process::Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("failed to execute the command");
            eprintln!("{}", e);
            std::process::exit(1)
        });
    match exec(&mut p, true) { // ここで上で定義したexecを呼び出す
        Ok(score) => {
            eprintln!("Score = {}", score);
        }
        Err(err) => {
            if let Ok(Some(status)) = p.try_wait() {
                if !status.success() {
                    std::process::exit(1);
                }
            }
            let _ = p.kill();
            eprintln!("{}", err);
            eprintln!("Score = 0");
        }
    }
}