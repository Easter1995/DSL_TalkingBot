use talking_bot::instruction::{parse_script_file, parse_str_to_instruction, Instruction};
use talking_bot::execute::execute_script;

#[test]
#[should_panic]
/// 测试模块定义错误的脚本
fn test_module_end_error() {
    let test_file = "scripts/module_end_error.txt";
    match parse_script_file(test_file.trim()) {
        Ok(script) => script,
        Err(err) => {
            panic!("脚本解析错误: {}", err);
        }
    };
}

#[test]
/// 测试规范的指令是否能被正确分析
fn test_good_instructions() {
    assert_eq!(parse_str_to_instruction("output \"output\""), Ok(Some(Instruction::Output("output".to_string()))));
    assert_eq!(parse_str_to_instruction("goto module1"), Ok(Some(Instruction::Goto("module1".to_string()))));
    assert_eq!(parse_str_to_instruction("input"), Ok(Some(Instruction::Input)));
    assert_eq!(parse_str_to_instruction("for /hello|hi/ goto module2"), Ok(Some(Instruction::For("hello|hi".to_string(), "module2".to_string()))));
    assert_eq!(parse_str_to_instruction("default goto default"), Ok(Some(Instruction::DefaultGoto("default".to_string()))));
    assert_eq!(parse_str_to_instruction("save var"), Ok(Some(Instruction::Save("var".to_string()))));
    assert_eq!(parse_str_to_instruction("eval balance=Number(balance)+Number(charge)"), Ok(Some(Instruction::Eval("balance=Number(balance)+Number(charge)".to_string()))));
    assert_eq!(parse_str_to_instruction("exit"), Ok(Some(Instruction::Exit)));
    assert_eq!(parse_str_to_instruction(""), Ok(None));
}

#[test]
/// 测试错误的指令是否能被识别
fn test_bad_instructions() {
    assert_eq!(parse_str_to_instruction("badins"), Err("无法解析指令：badins".to_string()));
    assert_eq!(parse_str_to_instruction("goto"), Err("无法解析指令：goto".to_string()));
    assert_eq!(parse_str_to_instruction("output out"), Err("无法解析指令：output out".to_string()));
}

#[test]
/// 测试script里面是否正确存储module
fn test_parse_script_file() {
    let test_file = "scripts/example1.txt";
    if let Ok(script) = parse_script_file(test_file) {
        if let Some(main_module) = script.modules.get("main") {
            if let Some(ins) = main_module.instructions.first() {
                assert_eq!(*ins, Instruction::Output("您好，您可以对我描述您的问题".to_string()));
            }
        }
        if let Some(menu_module) = script.modules.get("menu") {
            if let Some(ins) = menu_module.instructions.first() {
                assert_eq!(*ins, Instruction::Input);
            }
        }
    }
}


#[test]
/// 测试模块不存在的情况
fn test_module_not_exist_error() {
    let test_file = "scripts/module_not_exist_error.txt";
    let script = parse_script_file(test_file).expect("Failed to parse script file");
    // 捕获执行脚本时的 panic
    let result = std::panic::catch_unwind(|| {
        execute_script(&script);
    });
    // 确保捕获到了错误
    assert!(result.is_err(), "Expected an error, but the script executed successfully");
    // 验证错误信息是否正确
    if let Err(err) = result {
        if let Some(error_msg) = err.downcast_ref::<String>() {
            println!("Captured error: {}", error_msg);
            assert!(
                error_msg.contains("无法跳转到不存在的模块: foo"),
                "Expected error message to contain '无法跳转到不存在的模块: foo', but got: {}",
                error_msg
            );
        } else if let Some(error_msg) = err.downcast_ref::<&str>() {
            println!("Captured error: {}", error_msg);
            assert!(
                error_msg.contains("无法跳转到不存在的模块: foo"),
                "Expected error message to contain '无法跳转到不存在的模块: foo', but got: {}",
                error_msg
            );
        } else {
            panic!(
                "Expected a String or &str error message, but got an unknown type: {:?}",
                err
            );
        }
    }
}

#[test]
/// 验证main模块不存在的情况
fn test_no_main() {
    let test_file = "scripts/no_main.txt";
    let script = parse_script_file(test_file).expect("Failed to parse script file");
    // 捕获执行脚本时的 panic
    let result = std::panic::catch_unwind(|| {
        execute_script(&script);
    });
    // 确保捕获到了错误
    assert!(result.is_err(), "Expected an error, but the script executed successfully");
    // 验证错误信息是否正确
    if let Err(err) = result {
        if let Some(error_msg) = err.downcast_ref::<String>() {
            println!("Captured error: {}", error_msg);
            assert!(
                error_msg.contains("模块 'main' 不存在"),
                "Expected error message to contain '模块 'main' 不存在', but got: {}",
                error_msg
            );
        } else if let Some(error_msg) = err.downcast_ref::<&str>() {
            println!("Captured error: {}", error_msg);
            assert!(
                error_msg.contains("模块 'main' 不存在"),
                "Expected error message to contain '模块 'main' 不存在', but got: {}",
                error_msg
            );
        } else {
            panic!(
                "Expected a String or &str error message, but got an unknown type: {:?}",
                err
            );
        }
    }
}