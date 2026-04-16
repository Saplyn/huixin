# huixin

`huixin` 是一个基于 Rust + egui 的音乐创作工作区，当前由两个桌面程序组成：

- **huixin**：编排与演奏控制（节拍、片段、轨道、编辑工具）
- **huisheng**：音图（Patch）编辑与音频处理

它们共享同一套项目目录与序列化格式，适合在本地进行实验性音乐制作与交互控制。

## 仓库结构

```text
crates/
  huixin/    # 编排与控制端
  huisheng/  # 音图与音频处理端
  util/      # 共享工具与类型（项目结构、持久化、字体、通信格式）
Cargo.toml   # workspace 配置
```

## 环境要求

- Rust（建议使用最新稳定版，需支持 Edition 2024）
- Cargo
- Linux 下构建音频相关依赖时需要系统 `alsa` 开发库（如 `libasound2-dev`）

## 运行

在仓库根目录执行：

```bash
# 启动编排端
cargo run -p huixin

# 启动音频/音图端（建议在另一个终端）
cargo run -p huisheng
```

首次启动会提示选择“工作目录”（项目目录）。

## 项目目录（工作目录）约定

项目目录需要包含 `project.toml`，基础示例：

```toml
[info]
name = "my-project"
edition = 1

[dirs]
sheet = "sheet"    # 对应 sheet.ron（huixin）
patch = "patches"  # 对应 patches/*.ron（huisheng）
states = "states"  # 对应 states/{app_id}.ron
```

`[dirs]` 可选；未配置时默认使用 `sheet` / `patches` / `states`。

## 开发与校验

```bash
# 代码格式检查
cargo fmt --all --check

# 工作区构建检查
cargo check --workspace

# 工作区测试
cargo test --workspace
```

> 注意：在未安装系统 `alsa` 开发库的 Linux 环境中，`cargo check/test` 可能在 `alsa-sys` 阶段失败。
