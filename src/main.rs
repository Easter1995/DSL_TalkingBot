mod instruction;
mod execute;
use std::io::{self};

use execute::execute_script;
use instruction::{parse_script_file, Script};

fn main() {
    let mut script_path: String = String::new();
    let ascii_art = r#"
  _______    _ _    _             ____        _   
 |__   __|  | | |  (_)           |  _ \      | |  
    | | __ _| | | ___ _ __   __ _| |_) | ___ | |_ 
    | |/ _` | | |/ / | '_ \ / _` |  _ < / _ \| __|
    | | (_| | |   <| | | | | (_| | |_) | (_) | |_ 
    |_|\__,_|_|_|\_\_|_| |_|\__, |____/ \___/ \__|
                             __/ |                
                            |___/                 
    "#;
    println!("{}", ascii_art);
    println!("请输入你的脚本路径: ");
    let _ = io::stdin().read_line(&mut script_path);

    let script: Script = match parse_script_file(script_path.trim()) {
        Ok(script) => script,
        Err(err) => {
            panic!("脚本解析错误: {}", err);
        }
    };

    execute_script(&script);
}