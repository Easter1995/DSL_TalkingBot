use std::collections::HashMap;
use std::io::{self, BufRead, BufReader};
use std::fs::File;
use regex::Regex;

// 定义命令类型
#[derive(Clone, PartialEq, Debug)]
pub enum Instruction {
    Output(String),      // 输出一段文本
    Goto(String),        // 跳转到指定模块
    Input,               // 读取用户输入
    For(String, String), // 正则表达式，跳转模块名
    DefaultGoto(String), // 默认跳转模块
    Save(String),        // 保存输入
    Eval(String),        // 计算表达式
    Exit,                // 退出
}

// 定义模块
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

// 定义整个脚本，由模块组成
#[derive(Debug, Clone)]
pub struct Script {
    pub modules: HashMap<String, Module>,
}

// 转换一行->指令
pub fn parse_str_to_instruction(line: &str) -> Result<Option<Instruction>, String> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(None); // 跳过空行
    }

    // 匹配不同的指令
    if let Some(caps) = Regex::new(r#"^output\s+"(.*)"$"#).unwrap().captures(line) {
        let text = caps.get(1).unwrap().as_str().to_string();
        return Ok(Some(Instruction::Output(text)));
    }

    if let Some(caps) = Regex::new(r"^goto\s+(\w+)$").unwrap().captures(line) {
        let target = caps.get(1).unwrap().as_str().to_string();
        return Ok(Some(Instruction::Goto(target)));
    }

    if let Some(caps) = Regex::new(r#"^for\s+/(.*)/\s+goto\s+(\w+)$"#).unwrap().captures(line) {
        let pattern = caps.get(1).unwrap().as_str().to_string();
        let target = caps.get(2).unwrap().as_str().to_string();
        return Ok(Some(Instruction::For(pattern, target)));
    }

    if let Some(caps) = Regex::new(r"^default\s+goto\s+(\w+)$").unwrap().captures(line) {
        let target = caps.get(1).unwrap().as_str().to_string();
        return Ok(Some(Instruction::DefaultGoto(target)));
    }

    if let Some(caps) = Regex::new(r"^save\s+(\w+)$").unwrap().captures(line) {
        let var_name = caps.get(1).unwrap().as_str().to_string();
        return Ok(Some(Instruction::Save(var_name)));
    }

    if let Some(caps) = Regex::new(r#"^eval\s+(.*)$"#).unwrap().captures(line) {
        let expression = caps.get(1).unwrap().as_str().to_string();
        return Ok(Some(Instruction::Eval(expression)));
    }

    if line == "input" {
        return Ok(Some(Instruction::Input));
    }

    if line == "exit" {
        return Ok(Some(Instruction::Exit));
    }

    Err(format!("无法解析指令：{}", line))
}

pub fn parse_script_file(file_name: &str) -> Result<Script, io::Error> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);

    let mut modules: HashMap<String, Module> = HashMap::new();
    let mut current_module: Option<Module> = None;

    for line_result in reader.lines() {
        let line = line_result?;
        let line_trimmed = line.trim();

        // 匹配模块定义
        if let Some(caps) = Regex::new(r"^(\w+)\s*\{").unwrap().captures(line_trimmed) {
            // 如果有正在解析的模块，先保存
            if let Some(module) = current_module.take() {
                modules.insert(module.name.clone(), module);
            }

            // 开始新的模块
            let module_name = caps.get(1).unwrap().as_str().to_string();
            current_module = Some(Module {
                name: module_name,
                instructions: Vec::new(),
            });
            continue;
        }

        // 匹配模块结束
        if line_trimmed == "}" {
            if let Some(module) = current_module.take() {
                modules.insert(module.name.clone(), module);
            } else {
                return Err(io::Error::new(io::ErrorKind::Other, "未找到匹配的模块开始"));
            }
            continue;
        }

        // 解析指令
        if let Some(ref mut module) = current_module {
            match parse_str_to_instruction(line_trimmed) {
                Ok(Some(instruction)) => module.instructions.push(instruction),
                Ok(None) => {} // 空行或注释
                Err(e) => {
                    return Err(io::Error::new(io::ErrorKind::Other, format!("解析错误：{}", e)));
                }
            }
        } else {
            return Err(io::Error::new(io::ErrorKind::Other, "未找到模块定义"));
        }
    }

    // 检查未结束的模块
    if let Some(module) = current_module {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("模块 `{}` 未正确结束", module.name),
        ));
    }

    Ok(Script { modules })
}
