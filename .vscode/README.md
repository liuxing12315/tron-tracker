# VSCode 配置说明

## 🚀 快速启动

### 启动所有服务

1. 按 `Cmd+Shift+P` 打开命令面板
2. 输入 `Tasks: Run Task` 并选择
3. 选择 `start-all` 任务

或者使用调试模式：

1. 按 `Cmd+Shift+D` 打开调试面板
2. 选择 `Launch All` 配置
3. 按 `F5` 开始调试

### 单独启动服务

- **后端服务**: 运行 `start-backend` 任务
- **前端服务**: 运行 `start-frontend` 任务

## 📋 配置文件说明

### tasks.json

包含以下任务：

- `start-backend`: 启动 Rust 后端服务 (`cargo run`)
- `start-frontend`: 启动 React 前端开发服务器 (`pnpm dev`)
- `start-all`: 同时启动前后端服务

### launch.json

包含以下调试配置：

- `Launch Backend`: 调试 Rust 后端
- `Launch Frontend`: 调试 React 前端
- `Launch All`: 同时调试前后端

### settings.json

包含开发环境设置：

- Rust Analyzer 配置
- TypeScript/JavaScript 设置
- ESLint 配置
- 终端配置

## 🌐 访问地址

- **前端界面**: <http://localhost:5173>
- **后端API**: <http://localhost:8080>
- **WebSocket**: ws://localhost:8080/ws

## ⚙️ 环境要求

确保以下服务正在运行：

- **PostgreSQL**: 数据库服务
- **Redis**: 缓存服务

## 🔧 常见问题

### 端口冲突

如果端口被占用，可以在配置文件中修改端口：

- 前端: 修改 `admin-ui/vite.config.js` 中的 `server.port`
- 后端: 修改环境变量或配置文件中的端口设置

### 调试问题

如果调试器无法连接：

1. 确保服务已正确启动
2. 检查端口是否正确
3. 确认 Chrome 浏览器已安装（前端调试）
