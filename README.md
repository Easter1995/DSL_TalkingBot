[toc]



# 基于领域特定语言的客服机器人设计与实现

## 程序描述

### 程序功能

​		领域特定语言（Domain Specific Language，DSL）可以提供一种相对简单的文法，用于特定领域的业务流程定制。本作业要求定义一个领域特定脚本语言，这个语言能够描述在线客服机器人（机器人客服是目前提升客服效率的重要技术，在银行、通信和商务等领域的复杂信息系统中有广泛的应用）的自动应答逻辑，并设计实现一个解释器解释执行这个脚本，可以根据用户的不同输入，根据脚本的逻辑设计给出相应的应答。

### 项目结构

```
├── Cargo.lock          // 记录依赖包元数据，cargo自动维护
├── Cargo.toml          // cargo项目管理脚本，维护项目信息和依赖包
├── README.md  
├── scripts	            // 脚本样例		
│   ├── example1.txt
│   ├── module_end_error.txt
│   ├── module_not_exist_error.txt
│   └── no_main.txt
├── src		            // 项目源码
│    ├── execute.rs     // 将指令转换为可执行的程序
│    ├── instruction.rs // 将脚本翻译为指令
│    └── main.rs	    // 程序入口，读取文件
└── tests			    // 测试
	 └── tests.rs
```

## 程序使用方法

​		本项目采用rust编写，用户可在命令行输入如下命令运行项目：

```bash
$ cargo build	 #项目构建
$ cargo run		 #项目运行
$ cargo clean	 #清除缓存与编译文件
$ cargo test	 #项目测试
```

## DSL语法

### 语法定义

​		该语言是按模块书写的，用户必须定义模块及其模块名，然后在模块内写出要执行的语句。脚本中必须包含`main`模块，该模块是脚本执行的入口，如果没有main模块会报错。

​		模块定义方法如下：

```
main {
	语句
}
```

​		该语言支持如下语句：

1. **output  "输出字符串"**：在控制台打印出输出字符串，其中字符串必须用双引号括起来。
2. **goto 模块名**：跳转到模块名指定的模块。
3. **input**：等待用户输入。
4. **for /option1|option2/ goto 模块名**：模糊匹配`option1`和`option2`，匹配到任意一项都跳转到模块名指定的模块。
5. **default goto 模块名**：一般跟上一条`for`语句配合使用，表示没有匹配到的话就跳转到模块名指定的模块。
6. **save 变量名：**保存一个变量到上下文。
7. **eval 变量 = 表达式：**给变量赋值，后面可以跟任意字符或数字，也可以跟表达式。目前表达式只实现了加法，且表达式中出现了变量的话必须用`Number(变量名)`来表示它的类型。示例参考后面给出的正确编写的脚本。
8. **exit：**退出程序。

### 脚本示例

​		正确编写的脚本示例：

```bash
main {
    output "您好，您可以对我描述您的问题"
    goto menu
}
menu {
    input
    for /你好|您好/ goto greet
    for /余额/ goto balance
    for /激活/ goto active
    for /充值|存款/ goto recharge
    for /退出|再见/ goto exit
    default goto default
}
greet {
    output "您好，很高兴见到你。"
    goto menu
}
balance {
    output "您的银行卡余额是 ${balance} 元"
    goto menu
}
active {
    output "您已成功激活银行卡，送您 20 元"
    save balance
    eval balance = 20
    goto menu
}
recharge {
    output "请输入您的存款金额"
    input
    save amount
    eval balance = Number(balance) + Number(amount)
    output "您已成功存款，现在余额为 ${balance} 元"
    goto menu
}
exit {
    output "再见，祝您今天愉快"
    exit
}
default {
    output "对不起，我听不懂您在说什么"
    goto menu
}
```

​		该脚本的运行结果：

![image-20241129165351688](https://cdn.jsdelivr.net/gh/Easter1995/blog-image/202411291653173.png)

​		错误编写的脚本示例：

```
main {
    goto bar
}
bar {
    goto foo
}
```

​		该脚本的运行结果：可以看到如果脚本编写错误会有比较详细的错误信息

![image-20241130142703282](https://cdn.jsdelivr.net/gh/Easter1995/blog-image/202411301444122.png)

## 基于DSL的客服机器人实现概述

### instruction模块：将脚本翻译为Instruction类型

​		对应阶段：词法分析和语法分析。

#### 数据结构

1. **Instruction的定义**

   对应脚本中一条语句：

   ```rust
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
   ```

   - `Output(String)`：`String`表示要打印的内容
   - `Goto(String)`：`String`表示要跳转的模块名
   - `For(String, String)`：第一个`String`表示模糊匹配到的字符串，第二个`String`表示匹配到后要跳转的模块
   - `DefaultGoto(String)`：`String`表示要跳转的模块名
   - `Save(String)`：`String`表示要保存的变量名
   - `Eval(String)`：`String`保存整个表达式

2. **Module的定义**

   对应脚本中的一个模块，每个模块包含一些语句：

   ```rust
   pub struct Module {
       pub name: String,                   // 模块名
       pub instructions: Vec<Instruction>, // 模块中包含的指令
   }
   ```

3. **Script的定义**

   对应整个脚本，每个脚本包含一些模块：

   ```rust
   pub struct Script {
       pub modules: HashMap<String, Module>, // 组成脚本的模块
   }
   ```

#### 函数

1. **将脚本中的一行转换为Instruction类型存储**

   ```rust
   /// 对脚本文件中的一行进行词法分析和语法分析
   /// # Arguments
   ///   - line: 脚本文件中的一行字符串 
   pub fn parse_str_to_instruction(line: &str) -> Result<Option<Instruction>, String>
   ```

   - 使用正则表达式匹配脚本文件中的语句
   - 将识别出来的语句封装在`Result<Option>`类型里面返回
     - `Ok`表示语句正确识别
     - `Err`表示语句编写错误，并且返回错误信息，指出哪条指令错了

2. **将脚本中的一个模块转换为Module类型存储，将脚本转换为Script类型存储**

   ```rust
   /// 对文件进行词法分析和语法分析
   /// # Arguments
   ///   - file_name: 文件名
   pub fn parse_script_file(file_name: &str) -> Result<Script, io::Error>
   ```

   - 使用正则表达式识别出`module {`的结构，表示一个模块的开始，将模块名存入`current_module`
   - 使用正则表达式识别出`}`的结构，表示一个模块的结束，将`current_module`对应的模块名存入`modules`
   - 在这过程中随时检查错误，如果没有错误则将`modules`封装进`Script`结构体，将`Script`结构体封装进`Result`类型，返回`Ok`

### execute模块：将每一条Instruction转换为可执行的代码

​		对应阶段：语义分析和运行时执行，检查语法树中的语义规则是否正确，并且逐步解释并执行语法树中的节点（模块和指令）。

#### 数据结构

1. **上下文的定义**

   ```rust
   pub struct Context {
       pub variables: HashMap<String, String>, // 存储变量
       pub current_module: String,             // 当前模块名
       pub script: Script,                     // 脚本对象
   }
   impl Context {
       pub fn new(initial_module: &str, script: Script) -> Self {
           Self {
               variables: HashMap::new(),
               // 初始化为指定的脚本入口
               current_module: initial_module.to_string(), 
               script,
           }
       }
   }
   ```

   - 保存脚本中用`save`语句和`eval`语句保存的变量名和变量值
   - 保存当前执行的模块信息
   - 保存当前执行的脚本信息

#### 函数

1. **执行整个模块**

   ```rust
   /// 执行脚本
   /// Arguments
   ///   - script: 要执行的脚本
   pub fn execute_script(script: &Script) {
       let mut context = Context::new("main", script.clone());
       loop {
           // 执行模块
           if let Err(e) = execute_module(&mut context) {
               panic!("模块执行错误: {}", e);
           }
       }
   }
   ```

   - 执行整个脚本
   - 初始化上下文，将当前模块初始化为`main`模块（定义的脚本入口）
   - 在循环里面执行脚本中的模块
   - 捕获错误，如果模块执行函数返回了`Err`，就将程序`panic`

2. **执行当前模块**

   ```rust
   /// 执行上下文指定的当前模块
   /// Arguments:
   ///   - context: 上下文
   pub fn execute_module(context: &mut Context) -> Result<(), String>
   ```

   - 获取上下文里面的当前模块信息
   - 如果模块存在，则执行模块中的每一条指令
   - 如果模块不存在就返回一个`Err`类型，错误信息为模块 '{}' 不存在

3. **执行某一条指令**

   ```rust
   /// 执行指令
   /// Arguments:
   ///   - instruction: 当前要执行的指令
   ///   - context: 上下文，用于实现模块跳转和变量初始化/赋值
   pub fn execute_instruction(
       instruction: &Instruction,
       context: &mut Context,
   ) -> Result<Option<String>, String>
   ```

   - 根据`instruction`和`context`，为每一种指令编写了可执行的代码
   - 将返回值包装为`Result`类型
     - `Ok`：将信息包装成`Option`类型，如果该语句有模块跳转相关的内容（`goto`、`for goto`、`default goto`），则在`Ok`里封装`Some(target.clone())`；否则封装`None`
     - `Err`：封装相应的错误信息
   - 如果有`Save`和`Eval`类型的语句，更新上下文

4. **辅助函数：将output语句中用${var}表示的变量替换为变量的实际值**

   ```rust
   /// 将${var}表示的变量替换为var的实际值
   /// Arguments:
   ///   - text: 原始语句
   ///   - variables: key为变量名，value为变量的实际值
   fn replace_variables(text: &str, variables: &HashMap<String, String>) -> String
   ```

   - 返回处理好的字符串
   - 如果变量没有在`context`里面，则返回{{{}}}还没有初始化

5. **辅助函数：处理eval语句中的表达式**

   ```rust
   /// 处理eval语句中的表达式，将表达式中出现的变量替换为实际值
   /// 目前只实现整数加法和赋值语句
   /// Arguments:
   ///   - expression: 原始的表达式
   ///   - variables: key为变量名，value为实际值
   fn parse_assignment(expression: &str, variables: &HashMap<String, String>) -> Result<Option<(String, String)>, String>
   ```

   - 如果表达式中包含`Number(var)`这样的写法，就在`variables`里面提取`var`变量的实际值，如果`variables`中不存在该变量，则将其初始化为0
   - 否则直接给变量赋值
   - 返回值被包装在`Result`类型里面
     - `Ok`：用来包装`Some(var, value)`，表示变量名对应的新值
     - `Err`：返回相应的错误信息

### 语法树

1. **根节点：`Script`**
   - 包含所有的模块，模块存储为 `HashMap`。

2. **中间节点：`Module`**

   - 每个模块对应一个中间节点，表示一个逻辑块。

   - 包含模块名称和模块内的指令序列。

3. **叶子节点：`Instruction`**
   - 指令是语法树的叶子节点，表示具体的操作（如输出文本、跳转等）。

​		假设脚本如下：

```
main {
    output "您好，您可以对我描述您的问题。"
    goto menu
}
menu {
    input
    goto processInput
}
processInput {
    for /您好|你好/ goto hello
    for /余额/ goto balance
    default goto default
}
```

​		对应的语法树可以表示为：

```
Script {
    modules: {
        "main": Module {
            name: "main",
            instructions: vec![
                Instruction::Output("您好，您可以对我描述您的问题。".to_string()),
                Instruction::Goto("menu".to_string()),
            ],
        },
        "menu": Module {
            name: "menu",
            instructions: vec![
                Instruction::Input,
                Instruction::Goto("processInput".to_string()),
            ],
        },
        "processInput": Module {
            name: "processInput",
            instructions: vec![
                Instruction::For("/您好|你好/".to_string(), "hello".to_string()),
                Instruction::For("/余额/".to_string(), "balance".to_string()),
                Instruction::DefaultGoto("default".to_string()),
            ],
        },
    },
}
```

### 程序流程

1. **阶段1：词法分析**

   任务：将文本分解为词法单元。

   **实现代码：** `parse_str_to_instruction` 的早期正则匹配部分。

2. **阶段2：语法分析**

   任务：将词法单元转换为语法树（`Script`、`Module`、`Instruction`）。

   **实现代码：** `parse_script_file` 的模块组织逻辑。

3. **阶段3：语义分析**

   任务：检查语法树的合法性。

   **实现代码：** `execute_module` 和 `execute_instruction` 的逻辑中，隐式完成了大部分语义检查。

4. **阶段4：运行时执行**

   任务：根据语法树逐步执行指令。

   **实现代码：** `execute_module` 和 `execute_instruction` 的核心逻辑。

## 测试桩与单元测试

​		通过cargo test进行自动测试：

![image-20241130144403542](https://cdn.jsdelivr.net/gh/Easter1995/blog-image/202411301444791.png)

​		rust提供了很好的测试系统，可以将测试模块集成到源码中，一共编写了6个测试：

1. **测试模块未正确结束的脚本：**

   `main.rs`下：

   ```rust
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
   ```

2. **测试正确编写的语句是否能被正确翻译为Instruction类型：**

   `instruction.rs`下：语句较多因此省略一些

   ```rust
   #[test]
   /// 测试规范的指令是否能被正确分析
   fn test_good_instructions() {
       assert_eq!(parse_str_to_instruction("output \"output\""), Ok(Some(Instruction::Output("output".to_string()))));
       assert_eq!(parse_str_to_instruction("goto module1"), Ok(Some(Instruction::Goto("module1".to_string()))));
       ...
   }
   ```

3. **测试语法错误的语句是否能被识别：**

   `instruction.rs`模块下：

   ```rust
   #[test]
   /// 测试错误的指令是否能被识别
   fn test_bad_instructions() {
       assert_eq!(parse_str_to_instruction("badins"), Err("无法解析指令：badins".to_string()));
       assert_eq!(parse_str_to_instruction("goto"), Err("无法解析指令：goto".to_string()));
       assert_eq!(parse_str_to_instruction("output out"), Err("无法解析指令：output out".to_string()));
   }
   ```

4. **测试正确编写的脚本是否能被正确存储进Instruction：**

   `instruction.rs`下

   ```rust
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
   ```

5. **脚本语法正确，测试是否能识别出缺少了main模块且正常报错：**

   `execute.rs`下：

   ```rust
   #[test]
   /// 验证main模块不存在的情况
   fn test_no_main()
   ```

6. **脚本语法正确，测试要跳转的模块不存在的情况能被正确识别且正常报错：**

   `execute.rs`下：

   ```rust
   #[test]
   /// 测试模块不存在的情况
   fn test_module_not_exist_error()
   ```

   





