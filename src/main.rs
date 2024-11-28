mod instruction;
use std::io::{self, Read};

use instruction::{parse_script_file, Script};

fn main() {
    let mut script_path: String = String::new();
    println!("-------------TalkingBot-------------");
    println!("-------------by Orangec-------------");
    println!("请输入你的脚本路径: ");
    let _ = io::stdin().read_line(&mut script_path);

    let script: Script = match parse_script_file(script_path.trim()) {
        Ok(script) => script,
        Err(err) => {
            panic!("脚本解析错误: {}", err);
        }
    };
}