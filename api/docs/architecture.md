# Plan 任务管理系统架构设计方案

## 1. 背景与目标

`plan` 是一个前后端分离的 Web 任务管理系统，产品形态参考 `vibe-kanban` 的看板体验，但目标不是复制其完整能力，而是构建一个更聚焦、更容易本地部署和维护的独立应用。

当前仓库现状：

- 前端为独立 `Next.js 16 + React 19 + TypeScript` 项目。
- 后端为独立 `Rust` 项目，当前仅为初始化骨架。
- 底层数据存储采用 `SQLite`，本地部署，不依赖云数据库。

本方案的目标是：

- 明确 `plan` 的技术分层和模块边界。
- 明确首版需要实现的核心任务管理能力。
- 给出可直接落地的数据库设计、接口设计、前端组织方式和迭代顺序。

## 2. 设计原则

### 2.1 产品原则

- 保留 `vibe-kanban` 最核心的交互价值：看板视图、拖拽改状态、右侧任务详情面板、快速创建与编辑。
- 删除与当前目标无关的复杂能力：远程工作区、PR 同步、多组织、Electric 实时同步、复杂权限模型。
- 优先做单机本地可用版本，再考虑多用户和同步能力。

### 2.2 技术原则

- 前后端严格分离，前端不直接访问 SQLite。
- 后端作为唯一数据入口，统一处理事务、校验、排序和业务规则。
- 数据模型以“项目 -> 列状态 -> 任务”为核心，先保证主链路简单稳定。
- 保持模块可维护，避免一开始做成大而全的单体泥球。

## 3. 总体架构

系统采用经典三层结构：

1. 表现层：Next.js 前端，负责页面渲染、交互、状态展示。
2. 应用层：Rust API，负责用例编排、参数校验、事务管理、业务规则。
3. 数据层：SQLite，负责持久化项目、列、任务、评论、标签等实体。

架构关系如下：

```text
+---------------------------+
| Next.js Web Frontend      |
| - Kanban Board            |
| - Task Detail Panel       |
| - Filters / Search        |
+------------+--------------+
             |
             | HTTP JSON API
             v
+---------------------------+
| Rust API (Axum)           |
| - Routes                  |
| - Application Services    |
| - Domain Rules            |
| - SeaORM Repositories     |
+------------+--------------+
             |
             v
+---------------------------+
| SQLite                    |
| - projects                |
| - task_statuses           |
| - tasks                   |
| - task_comments           |
| - tags / task_tags        |
| - task_relations          |
| - activity_logs           |
+---------------------------+
```

## 4. 技术选型

### 4.1 前端

- 框架：`Next.js 16`
- UI：`React 19`
- 语言：`TypeScript`
- 样式：`Tailwind CSS 4`
- 服务端状态管理：`@tanstack/react-query`
- UI 状态管理：`zustand`
- 表单校验：`zod`
- 拖拽：`@dnd-kit/core`、`@dnd-kit/sortable`

选型原因：

- Next.js 适合承载应用级路由和未来的 SEO/静态能力。
- React Query 适合管理看板快照、任务详情、评论等服务端数据。
- Zustand 适合管理右侧面板开关、筛选条件、当前选中任务等本地状态。
- `dnd-kit` 比传统方案更适合现代 React 应用，灵活性更高。

### 4.2 后端

- Web 框架：`axum`
- 异步运行时：`tokio`
- 数据访问：`sea-orm`
- 序列化：`serde`
- 时间处理：`chrono` 或 `time`
- 唯一 ID：`uuid`
- 错误处理：`thiserror` + 统一 API 错误映射
- 日志：`tracing` + `tracing-subscriber`

选型原因：

- `axum` 生态成熟，适合构建清晰的 REST API。
- `SeaORM` 对 SQLite 支持稳定，实体、关系和迁移工具链完整，适合当前项目从骨架阶段快速落地。
- 当前项目虽然规模不大，但仍需要清晰的模型定义、迁移管理和事务边界，`SeaORM + repository` 的组合更平衡。

### 4.3 数据库

- 数据库：`SQLite`
- 部署方式：本地文件数据库，例如 `data/plan.db`
- ORM：`SeaORM`
- 迁移工具：`sea-orm-migration`

选型原因：

- 满足单机部署、轻量运维和快速启动需求。
- 对首版单用户/小团队场景足够。
- 后续如需升级到 Postgres，可在 repository 层做相对平滑迁移。

## 5. 仓库结构建议

### 5.1 顶层结构

```text
plan/
├── app/                      # Next.js App Router
├── components/               # 前端通用组件
├── features/                 # 前端按业务拆分
├── lib/                      # 前端基础工具、API client
├── stores/                   # Zustand stores
├── api/                      # Rust backend
│   ├── docs/
│   ├── src/
│   └── Cargo.toml
├── migration/                # SeaORM migrations
│   ├── src/
│   └── Cargo.toml
└── public/
```

### 5.2 Rust 后端结构

虽然当前 `api` 还是单 crate，但建议尽早按模块拆分。第一阶段可以先单 crate，目录内部按职责分层：

```text
api/src/
├── main.rs
├── lib.rs
├── config/
├── routes/
├── dto/
├── app/
├── domain/
├── repository/
├── db/
└── error/
```

各层职责：

- `routes/`：HTTP 路由与请求响应映射。
- `dto/`：API 请求与响应结构体。
- `app/`：应用服务，负责具体用例编排。
- `domain/`：领域实体、值对象、业务规则。
- `repository/`：数据库查询与持久化。
- `db/`：连接池、迁移、事务工具。
- `error/`：统一错误类型和 HTTP 错误输出。

后续如果后端变复杂，可升级为 Cargo workspace：

- `crates/server`
- `crates/db`
- `crates/domain`
- `crates/api-types`

## 6. 核心业务范围

### 6.1 V1 必做能力

- 项目管理
- 看板列管理
- 任务创建、编辑、删除、归档
- 任务拖拽改列和排序
- 右侧任务详情面板
- 标签管理
- 评论
- 搜索与基础筛选

### 6.2 V1 不做能力

- 多组织
- 实时协同编辑
- PR / Git 工作区联动
- 远程同步引擎
- 复杂权限系统
- 自动化工作流引擎

### 6.3 V2 候选能力

- 子任务
- 任务关系（阻塞/关联）
- 活动时间线
- 导入导出
- 多用户和成员角色
- 通知

## 7. 领域模型设计

### 7.1 核心实体

#### Project

表示一个任务看板项目。

关键字段：

- `id`
- `name`
- `slug`
- `description`
- `color`
- `created_at`
- `updated_at`

#### TaskStatus

表示项目中的看板列。

关键字段：

- `id`
- `project_id`
- `name`
- `color`
- `sort_order`
- `is_done`
- `is_hidden`

说明：

- `sort_order` 控制列顺序。
- `is_done` 用于统计和筛选时识别完成列。
- `is_hidden` 便于后续支持隐藏列。

#### Task

表示核心任务卡片。

关键字段：

- `id`
- `project_id`
- `status_id`
- `title`
- `description`
- `priority`
- `position`
- `start_date`
- `due_date`
- `completed_at`
- `archived_at`
- `created_at`
- `updated_at`

说明：

- `position` 用于列内排序。
- 不建议直接复用 `vibe-kanban` 的 `sort_order = 1000 * 列序 + 任务序` 方案。
- `position` 应单独表示列内顺序，跨列移动时只更新目标列和受影响列的顺序。

#### Tag

表示项目级标签。

字段：

- `id`
- `project_id`
- `name`
- `color`

#### TaskComment

表示任务评论。

字段：

- `id`
- `task_id`
- `author_name`
- `content`
- `created_at`
- `updated_at`

首版可先用 `author_name` 或固定系统用户，暂不引入完整用户表。

#### TaskRelation

表示任务之间的关系。

字段：

- `id`
- `task_id`
- `related_task_id`
- `relation_type`

`relation_type` 建议支持：

- `blocking`
- `related`

#### ActivityLog

记录关键操作，便于后续任务时间线展示。

字段：

- `id`
- `project_id`
- `task_id`
- `event_type`
- `payload_json`
- `created_at`

## 8. SQLite 表结构建议

更完整的数据库设计见 [database-design.md](/home/tianci/plan/api/docs/database-design.md)。

### 8.1 projects

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  description TEXT,
  color TEXT NOT NULL DEFAULT '#2563eb',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### 8.2 task_statuses

```sql
CREATE TABLE task_statuses (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  color TEXT NOT NULL,
  sort_order INTEGER NOT NULL,
  is_done INTEGER NOT NULL DEFAULT 0,
  is_hidden INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

### 8.3 tasks

```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  status_id TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  priority TEXT,
  position INTEGER NOT NULL,
  start_date TEXT,
  due_date TEXT,
  completed_at TEXT,
  archived_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (status_id) REFERENCES task_statuses(id) ON DELETE RESTRICT
);
```

建议索引：

```sql
CREATE INDEX idx_tasks_project_status_position
ON tasks(project_id, status_id, position);

CREATE INDEX idx_tasks_project_updated_at
ON tasks(project_id, updated_at DESC);
```

### 8.4 task_comments

```sql
CREATE TABLE task_comments (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  author_name TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

### 8.5 tags 与 task_tags

```sql
CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL,
  color TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE task_tags (
  task_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (task_id, tag_id),
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

### 8.6 task_relations

```sql
CREATE TABLE task_relations (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  related_task_id TEXT NOT NULL,
  relation_type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
  FOREIGN KEY (related_task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

### 8.7 activity_logs

```sql
CREATE TABLE activity_logs (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_id TEXT,
  event_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

## 9. API 设计

API 风格采用 REST + JSON。

统一前缀建议：

- 本地开发：`http://localhost:3001/api`
- 生产部署：由反向代理统一暴露

### 9.1 项目接口

- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/:projectId`
- `PATCH /api/projects/:projectId`
- `DELETE /api/projects/:projectId`

### 9.2 看板快照接口

这是前端主页面的关键接口。

- `GET /api/projects/:projectId/board`

返回建议：

```json
{
  "project": {},
  "statuses": [],
  "tasks": [],
  "tags": [],
  "taskTags": [],
  "summary": {
    "totalTasks": 0,
    "doneTasks": 0
  }
}
```

设计原因：

- 前端打开项目页时只需一次请求即可渲染主看板。
- 避免项目、列、任务、标签拆成多次首屏请求。

### 9.3 状态列接口

- `GET /api/projects/:projectId/statuses`
- `POST /api/projects/:projectId/statuses`
- `PATCH /api/statuses/:statusId`
- `DELETE /api/statuses/:statusId`
- `POST /api/projects/:projectId/statuses/reorder`

### 9.4 任务接口

- `POST /api/tasks`
- `GET /api/tasks/:taskId`
- `PATCH /api/tasks/:taskId`
- `DELETE /api/tasks/:taskId`
- `POST /api/tasks/:taskId/archive`
- `POST /api/tasks/:taskId/restore`

### 9.5 任务拖拽排序接口

- `POST /api/tasks/reorder`

请求体建议：

```json
{
  "movedTaskId": "task_xxx",
  "sourceStatusId": "status_todo",
  "destinationStatusId": "status_doing",
  "orderedTaskIds": [
    "task_1",
    "task_2",
    "task_3"
  ]
}
```

也可以采用批量更新方案：

```json
{
  "updates": [
    { "id": "task_1", "statusId": "status_doing", "position": 1000 },
    { "id": "task_2", "statusId": "status_doing", "position": 2000 }
  ]
}
```

建议后端统一在事务中完成：

- 校验任务与列归属同一项目。
- 更新跨列任务的 `status_id`。
- 按最新顺序批量更新 `position`。
- 写入 `activity_logs`。

### 9.6 评论接口

- `GET /api/tasks/:taskId/comments`
- `POST /api/tasks/:taskId/comments`
- `PATCH /api/comments/:commentId`
- `DELETE /api/comments/:commentId`

### 9.7 标签接口

- `GET /api/projects/:projectId/tags`
- `POST /api/projects/:projectId/tags`
- `PATCH /api/tags/:tagId`
- `DELETE /api/tags/:tagId`
- `POST /api/tasks/:taskId/tags/:tagId`
- `DELETE /api/tasks/:taskId/tags/:tagId`

### 9.8 搜索与筛选接口

- `GET /api/projects/:projectId/tasks/search?q=...`

支持的筛选条件建议包括：

- `status_id`
- `priority`
- `tag_id`
- `is_archived`
- `keyword`

## 10. 前端架构设计

### 10.1 页面结构

首版建议采用以下页面：

- `/`：项目列表或默认跳转页
- `/projects/[projectId]`：主看板页
- `/projects/[projectId]?taskId=xxx`：带右侧详情面板的主看板页

不建议首版直接使用复杂的并行路由和拦截路由来承载侧边面板。原因：

- 当前项目仍在初期，过早引入复杂路由会提升维护成本。
- 用 `searchParams` 管理选中任务即可满足体验要求。
- 后续若确实需要更复杂的 URL 状态，再逐步升级。

### 10.2 前端模块划分

```text
features/
├── project/
├── board/
├── task/
├── comment/
├── tag/
└── search/
```

示例职责：

- `project/`：项目列表、项目设置。
- `board/`：看板容器、列、拖拽、批量排序。
- `task/`：任务卡片、详情面板、创建编辑表单。
- `comment/`：评论列表和编辑。
- `tag/`：标签选择器、标签管理。
- `search/`：筛选栏和搜索结果。

### 10.3 状态管理策略

#### React Query

管理服务端数据：

- `project list`
- `board snapshot`
- `task detail`
- `comments`
- `tags`

#### Zustand

管理纯 UI 状态：

- 当前选中任务 ID
- 右侧面板开关
- 当前筛选条件
- 看板视图偏好
- 搜索输入临时值

### 10.4 前端数据流

主看板加载：

1. 页面进入 `/projects/[projectId]`
2. 调用 `GET /api/projects/:projectId/board`
3. 前端按 `statuses + tasks` 分组渲染列与卡片
4. 点击卡片后，更新 `taskId` 参数并加载任务详情

拖拽任务：

1. 前端本地乐观更新列内任务顺序
2. 调用 `POST /api/tasks/reorder`
3. 成功则保持本地状态
4. 失败则回滚并提示用户

### 10.5 UI 交互建议

借鉴 `vibe-kanban`，但做精简版：

- 中间区域为横向看板列。
- 右侧为任务详情面板。
- 顶部为项目名、搜索框、筛选栏、创建按钮。
- 卡片上显示标题、标签、优先级、日期。
- 详情面板编辑采用自动保存或显式保存二选一，首版建议显式保存，风险更低。

## 11. 后端架构设计

### 11.1 路由层

职责：

- 接收 HTTP 请求
- 解析路径参数、查询参数和 JSON body
- 调用应用服务
- 输出统一 JSON 响应

不负责：

- 拼接复杂业务逻辑
- 直接编写分散的 SQL

### 11.2 应用服务层

职责：

- 承载用例，例如：
  - 创建任务
  - 更新任务
  - 移动任务
  - 获取看板快照
- 组织事务
- 调用多个 repository
- 记录活动日志

### 11.3 Repository 层

职责：

- 负责 SQL 查询与持久化
- 保持接口清晰，例如：
  - `ProjectRepository`
  - `TaskRepository`
  - `TaskStatusRepository`
  - `CommentRepository`
  - `TagRepository`

### 11.4 错误处理

建议统一错误模型：

- `ValidationError`
- `NotFoundError`
- `ConflictError`
- `InternalError`

HTTP 映射：

- `400`：请求参数非法
- `404`：资源不存在
- `409`：排序冲突、唯一性冲突
- `500`：服务内部错误

错误响应示例：

```json
{
  "code": "validation_error",
  "message": "title must not be empty"
}
```

## 12. 排序与拖拽策略

排序是看板系统的高风险点，必须单独设计。

### 12.1 推荐方案

使用列内独立 `position` 字段。

规则：

- 同一列内任务按 `position ASC` 排序。
- 创建任务时插入列尾，默认 `max(position) + 1000`。
- 拖拽时后端根据前端提交的新顺序重排受影响列。

### 12.2 为什么不直接复用大一统排序值

虽然 `vibe-kanban` 使用把列序和任务序编码到一个 `sort_order` 的思路，但在 `plan` 中不建议直接沿用，原因如下：

- 列排序和任务排序是两个不同维度。
- 后续如果列顺序变化，会影响全部任务排序值。
- 使用独立 `position` 更直观，数据库查询和维护成本更低。

### 12.3 事务要求

拖拽更新必须放在同一个事务中：

- 锁定或读取当前目标列顺序
- 更新移动任务的目标列
- 重写目标列及受影响源列的 `position`
- 写活动日志

这样可以避免局部更新导致的错乱顺序。

## 13. 本地部署方案

### 13.1 开发环境

前端：

- `pnpm dev`

后端：

- `cargo run`

本地联调建议：

- Next.js 运行在 `3000`
- Rust API 运行在 `3001`
- 前端通过 `next.config.ts` rewrites 代理 `/api/*`

### 13.2 生产部署

推荐两种模式：

#### 模式 A：前后端双进程

- `Next.js` 独立进程
- `Rust API` 独立进程
- `SQLite` 文件存放在本地持久化目录
- Nginx 或 Caddy 统一反向代理

#### 模式 B：静态前端 + Rust API

如果后续前端主要是客户端渲染，也可将前端静态化后由独立 HTTP 服务托管，后端继续独立提供 API。

首版更推荐模式 A，演进成本更低。

## 14. 安全与可靠性

首版虽然是本地部署，也需要保留基本边界：

- 后端对所有写接口做参数校验。
- 所有更新时间使用服务端时间，不信任前端传入。
- 删除操作建议优先归档而不是物理删除。
- 数据库定期备份，后续可加导出能力。

对于 SQLite 运行建议：

- 开启 WAL 模式。
- 统一通过连接池访问数据库。
- 启动时自动执行 migration。

## 15. 开发阶段规划

### 阶段一：最小可用版本

目标：

- 项目
- 状态列
- 任务 CRUD
- 看板视图
- 拖拽排序
- 右侧详情面板

交付结果：

- 能像 `vibe-kanban` 一样完成基础任务流转。

### 阶段二：增强任务管理

目标：

- 标签
- 评论
- 搜索
- 筛选
- 归档
- 活动日志

### 阶段三：高级能力

目标：

- 子任务
- 任务关系
- 导入导出
- 多用户
- 通知

## 16. 与 vibe-kanban 的关系

`plan` 应该借鉴 `vibe-kanban` 的是交互模式和任务域抽象，而不是完整技术架构。

可以借鉴的部分：

- 看板主视图 + 详情面板布局
- 项目范围数据聚合思想
- 拖拽后批量更新顺序的处理方式
- 状态列与任务的清晰分层

不建议直接复用的部分：

- Electric shape 同步
- 远程组织模型
- 工作区、PR、Review、远程连接等复杂能力
- 过于庞大的实体关联网络

## 17. 最终建议

对 `plan` 来说，最合理的实现路径不是“做一个简化版 `vibe-kanban` monorepo”，而是：

- 前端保留 `vibe-kanban` 的核心交互体验；
- 后端采用清晰的 Rust REST API；
- 数据层使用 SQLite 本地持久化；
- V1 只围绕项目、列、任务、详情、拖拽这条主链路打透。

这样可以在较低复杂度下，最快得到一个真正可用、可维护、可迭代的任务管理 Web 应用。
