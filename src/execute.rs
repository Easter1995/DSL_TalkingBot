use std::collections::HashMap;
use crate::instruction::{Instruction, Script};
use regex::Regex;

// 脚本上下文
pub struct Context {
    pub variables: HashMap<String, String>, // 存储变量
    pub current_module: String,             // 当前模块名
    pub script: Script,                     // 脚本对象
}
impl Context {
    pub fn new(initial_module: &str, script: Script) -> Self {
        Self {
            variables: HashMap::new(),
            current_module: initial_module.to_string(), // 初始化为指定的脚本入口
            script,
        }
    }
}

pub fn execute_instruction(
    instruction: &Instruction,
    context: &mut Context,
) -> Result<Option<String>, String> {
    match instruction {
        Instruction::Output(text) => {
            // 替换变量并输出
            let output = replace_variables(text, &context.variables);
            println!("{}", output);
            Ok(None)
        }
        Instruction::Goto(target) => {
            // 跳转到目标模块
            if context.script.modules.contains_key(target) {
                Ok(Some(target.clone()))
            } else {
                Err(format!("无法跳转到不存在的模块: {}", target))
            }
        }
        Instruction::Input => {
            // 模拟用户输入
            println!("等待输入:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_string();

            if !input.is_empty() {
                context.variables.insert("last_input".to_string(), input);
                Ok(None)
            } else {
                Ok(None)
            }
        }
        Instruction::For(pattern, target) => {
            // 使用正则表达式匹配上下文中的输入
            if let Some(input) = context.variables.get("last_input") {
                if regex::Regex::new(pattern)
                    .map_err(|e| format!("正则表达式错误: {}", e))?
                    .is_match(input)
                {
                    Ok(Some(target.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        Instruction::DefaultGoto(target) => {
            // 默认跳转目标
            Ok(Some(target.clone()))
        }
        Instruction::Save(var_name) => {
            // 保存输入到指定变量
            if let Some(input) = context.variables.get("last_input") {
                context.variables.insert(var_name.clone(), input.clone());
                Ok(None)
            } else {
                Err("没有可保存的输入".to_string())
            }
        }
        Instruction::Eval(expression) => {
            // 简单计算表达式并更新变量
            if let Some((var, value)) = parse_assignment(expression, &context.variables)? {
                context.variables.insert(var, value);
                Ok(None)
            } else {
                Err(format!("无法解析表达式：{}", expression))
            }        
        }
        Instruction::Exit => {
            // 退出脚本
            println!("退出脚本执行。");
            std::process::exit(0);
        }
    }
}

pub fn execute_module(context: &mut Context) -> Result<(), String> {
    loop {
        let current_module_name = context.current_module.clone();
        let cur_script = context.script.clone();
        let module = cur_script
            .modules
            .get(&current_module_name)
            .ok_or_else(|| format!("模块 '{}' 不存在", context.current_module))?;
        
        for instruction in &module.instructions {
            if let Some(next_module) = execute_instruction(instruction, context)? {
                context.current_module = next_module;
                break;
            }
        }
    }
}

pub fn execute_script(script: &Script) {
    let mut context = Context::new("main", script.clone());
    loop {
        // 执行模块
        if let Err(e) = execute_module(&mut context) {
            eprintln!("模块执行错误: {}", e);
            break;  
        }
    }
}

fn replace_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    // 使用正则表达式匹配 `${key}` 格式的占位符
    let re = regex::Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();

    result = re.replace_all(&result, |caps: &regex::Captures| {
        let key = &caps[1];
        // 如果变量存在，替换为变量值；否则替换为 `{key}还没有初始化`
        variables.get(key).cloned().unwrap_or_else(|| format!("{{{}}}还没有初始化", key))
    })
    .to_string();

    result
}

fn parse_assignment(expression: &str, variables: &HashMap<String, String>) -> Result<Option<(String, String)>, String> {
    let parts: Vec<&str> = expression.split('=').collect();
    if parts.len() == 2 {
        let var = parts[0].trim().to_string();
        let value_expr = parts[1].trim();

        // 处理简单的加法表达式，例如 Number1 + Number2
        if value_expr.contains("Number(") {
            let re = Regex::new(r"Number\((\w+)\)").unwrap();
            let mut total = 0;
            for cap in re.captures_iter(value_expr) {
                let var_name = &cap[1];
                let val_str = variables.get(var_name).map(|s| s.as_str()).unwrap_or("0");
                let val = val_str.parse::<i32>().unwrap_or(0);
                total += val;
            }
            Ok(Some((var, total.to_string())))
        } else {
            // 直接赋值
            Ok(Some((var, value_expr.to_string())))
        }
    } else {
        Err("无效的赋值表达式".to_string())
    }
}
