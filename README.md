# V2EX TUI

一个基于终端的 V2EX 查看器，使用 V2EX API 2.0 Beta。

## 功能特性

- 🖥️ 终端界面，无需浏览器
- 📋 浏览节点主题列表
- 📖 查看主题详情和回复
- 🔔 查看通知
- 👤 查看个人资料
- 🎨 彩色界面，易于阅读
- ⌨️ Vim 风格快捷键

## 安装

### 前置要求

- Rust 1.70+
- V2EX Personal Access Token

### 构建

```bash
git clone <repository>
cd v2ex-tui
cargo build --release
```

编译完成后，可执行文件位于 `target/release/v2ex-tui`。

## 配置

### 获取 Personal Access Token

1. 登录 [V2EX](https://www.v2ex.com)
2. 进入设置 → API 访问令牌
3. 创建新的令牌（有效期 30-180 天）
4. **重要**：创建后 10 分钟内可以完整查看令牌，请立即保存

### 保存令牌

```bash
mkdir -p ~/.config/v2ex
echo "YOUR_PERSONAL_ACCESS_TOKEN" > ~/.config/v2ex/token.txt
chmod 600 ~/.config/v2ex/token.txt
```

## 使用

```bash
./v2ex-tui
```

### 快捷键

#### 导航
- `j` / `↓` - 向下移动
- `k` / `↑` - 向上移动
- `h` / `←` - 返回上一级
- `l` / `→` / `Enter` - 打开选中项
- `g` - 跳到顶部
- `G` - 跳到底部
- `PageUp` - 上一页
- `PageDown` - 下一页

#### 功能
- `r` - 刷新当前视图
- `n` - 查看通知
- `p` - 查看个人资料
- `t` - 切换显示回复（在主题详情中）
- `?` - 显示帮助
- `q` / `Esc` - 退出/返回

#### 快速切换节点
- `1` - Python
- `2` - Programmer
- `3` - Share
- `4` - Create
- `5` - Jobs
- `6` - Go
- `7` - Rust
- `8` - JavaScript
- `9` - Linux

## API 限制

- 每个 IP 每小时限制 600 次请求
- API 2.0 Beta 暂不支持发帖和回复

## 许可证

MIT
