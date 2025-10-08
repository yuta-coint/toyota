#![allow(non_snake_case)]

use std::process::Stdio;
use tools::*;

fn main() {
    if std::env::args().len() < 2 {
        eprintln!("Usage: {} <command>", std::env::args().nth(0).unwrap());
        return;
    }

    let command = std::env::args().nth(1).unwrap();
    
    // ★ 修正点: `command`の所有権を渡さず、参照(&)を渡す
    let mut p = std::process::Command::new(&command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            // これでエラーメッセージ内でも`command`が使える
            eprintln!("failed to execute command `{}`: {}", command, e);
            std::process::exit(1)
        });

    match exec(&mut p) {
        Ok(score) => {
            println!("Score = {}", score);
        }
        Err(err) => {
            let _ = p.kill();
            eprintln!("Testerror: {}", err);
            println!("Score = 0");
        }
    }
}