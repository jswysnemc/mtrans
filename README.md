# mtrans

使用大模型 API 进行翻译的 CLI 工具。

## 功能特性

- 🌍 **基础翻译** - 支持多语言翻译，自动检测源语言，流式输出
- 📖 **词典模式** - 提供词义解释、词根分析、例句等
- 💻 **变量命名模式** - 将文本转换为不同风格的变量名（snake_case, camelCase 等）
- 🔄 **交互式模式** - REPL 模式，连续翻译
- 📁 **文件支持** - 从文件读取，输出到文件
- 📋 **剪贴板支持** - 自动复制结果到剪贴板
- 📝 **自定义 Prompt** - 支持自定义翻译模板
- 🔐 **安全存储** - API Key 使用硬件指纹加密存储

## 安装

```bash
# 克隆项目
git clone <repository-url>
cd mtrans

# 编译安装
cargo build --release
cargo install --path .
```

## 配置

首次使用前需要配置 API 信息：

```bash
# 交互式初始化配置
mtrans config init

# 或单独设置配置项
mtrans config set --key <your-api-key>
mtrans config set --model gpt-3.5-turbo
mtrans config set --baseurl https://api.openai.com/v1

# 显示当前配置
mtrans config info

# 测试连接
mtrans config test

# 清除配置
mtrans config clear
```

> ⚠️ **安全说明**: API Key 使用 AES-256-GCM 加密，基于硬件指纹派生密钥。配置文件无法在其他机器上解密。

配置文件位置：
- Linux/macOS: `~/.config/mtrans/config.toml`
- Windows: `%APPDATA%\mtrans\config.toml`

Prompt 模板位置：
- `~/.config/mtrans/prompt.d/` (Linux/macOS)
- `%APPDATA%\mtrans\prompt.d\` (Windows)

## 使用方法

### 语法糖

mtrans 支持多种便捷的语法糖：

```bash
# :lang - 指定目标语言
mtrans :zh "Hello world"        # 翻译为中文
mtrans :jp "Hello world"        # 翻译为日语

# :from,to - 指定源语言和目标语言
mtrans :en,zh "Hello"           # 英文 → 中文
mtrans :zh,jp "你好"            # 中文 → 日语

# i:lang / o:lang - 分别指定输入/输出语言
mtrans i:en o:zh "Hello"        # 英文 → 中文
mtrans i:zh "你好世界"          # 指定源语言为中文
mtrans o:jp "Hello"             # 指定目标语言为日语
```

### 基础翻译

```bash
# 直接翻译文本（自动检测语言）
mtrans "Hello world"

# 使用标准参数
mtrans "Hello world" -t zh           # 指定目标语言
mtrans "你好世界" -F en              # 指定源语言（-F 大写）

# 从标准输入读取
echo "Hello" | mtrans :zh
cat README.md | mtrans :zh

# 翻译命令行帮助
ls --help | mtrans
git --help | mtrans :zh
```

### 词典模式

```bash
# 词典模式（解释、词根、例句）
mtrans -w serendipity
mtrans -w ephemeral
```

输出示例：
```
**serendipity** /ˌserənˈdipədē/

📖 **释义**
- [n.] 意外发现美好事物的能力；机缘巧合的幸运发现

🔤 **词源**
源自1754年霍勒斯·沃波尔创造，灵感来自波斯童话《三个塞兰迪普王子》

📝 **例句**
1. The discovery of penicillin was a classic case of serendipity.
   → 青霉素的发现是机缘巧合的典型例子。

🔗 **相关词汇**
- 同义词: chance, accident, fortuitous discovery
- 反义词: deliberate planning, intentionality
```

### 变量命名模式

```bash
# 变量命名模式（默认输出所有风格）
mtrans -c "获取用户信息"

# 指定命名风格
mtrans -c "user login status" -s snake_case      # user_login_status
mtrans -c "user login status" -s camelCase       # userLoginStatus
mtrans -c "user login status" -s PascalCase      # UserLoginStatus
mtrans -c "user login status" -s kebab-case      # user-login-status
mtrans -c "user login status" -s CONSTANT_CASE   # USER_LOGIN_STATUS
```

输出示例：
```
snake_case: get_user_info
camelCase: getUserInfo
PascalCase: GetUserInfo
kebab-case: get-user-info
CONSTANT_CASE: GET_USER_INFO
```

### 交互式模式

```bash
# 进入 REPL 交互模式
mtrans -i
```

交互模式命令：
```
mtrans> Hello world              # 直接翻译
mtrans> :zh Hello world          # 翻译为中文
mtrans> :en 你好                 # 翻译为英文
mtrans> help                     # 显示帮助
mtrans> exit                     # 退出
```

### 文件操作

```bash
# 从文件读取
mtrans -f input.txt

# 输出到文件
mtrans "Hello world" -o output.txt

# 组合使用
mtrans -f input.txt -o output.txt -t zh
```

### 剪贴板

```bash
# 自动复制结果到剪贴板
mtrans "Hello world" --clipboard
mtrans :zh "Hello" --clipboard
```

### 显示支持的语言

```bash
mtrans -l
```

## 支持的语言

| 代号 | 语言名称 |
|------|----------|
| auto | 自动检测 |
| en   | 英语 |
| zh   | 中文 |
| jp   | 日语 |
| ko   | 韩语 |
| fr   | 法语 |
| de   | 德语 |
| es   | 西班牙语 |
| ru   | 俄语 |
| pt   | 葡萄牙语 |
| it   | 意大利语 |

## 自定义 Prompt 模板

在 `prompt.d` 目录下可以自定义以下模板：

- `word.prompt` - 词典模式模板
- `code.prompt` - 变量命名模式模板
- `common.prompt` - 通用翻译模板

模板变量：
- `{{text}}` - 待翻译的文本
- `{{source_lang}}` - 源语言
- `{{target_lang}}` - 目标语言
- `{{style}}` - 代码命名风格

## 命令行参数

```
mtrans [OPTIONS] [TEXT] [COMMAND]

参数:
  [TEXT]  待翻译的文本（也支持 :lang 语法糖）

选项:
  -F, --from <LANG>         源语言
  -t, --to <LANG>           目标语言
  -l, --list-languages      显示支持的语言列表
  -w, --word                词典模式
  -c, --code                变量命名模式
  -s, --style <STYLE>       代码命名风格
  -i, --interactive         交互式 REPL 模式
  -f, --file <PATH>         从文件读取
  -o, --output <PATH>       输出到文件
      --clipboard           复制到剪贴板
  -h, --help                显示帮助信息

子命令:
  config                    配置管理
```

## 配置子命令

```
mtrans config <COMMAND>

命令:
  init      交互式初始化配置
  set       设置配置项 (--key, --model, --baseurl)
  info      显示当前配置
  test      测试连接
  clear     清除配置
```

## 安全性

mtrans 使用以下安全措施保护您的 API Key：

1. **硬件指纹加密** - API Key 使用 AES-256-GCM 加密存储
2. **密钥派生** - 加密密钥从机器唯一标识（machine-id）派生
3. **防复制** - 配置文件在其他机器上无法解密
4. **脱敏显示** - `config info` 只显示 API Key 的首尾字符

## 开发

```bash
# 运行
cargo run -- "Hello world"
cargo run -- :zh "Hello world"

# 测试
cargo test

# 构建 release 版本
cargo build --release
```

## 许可证

MIT License
