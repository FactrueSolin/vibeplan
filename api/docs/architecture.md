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

- HTTP 框架：`axum`
- 中间件：`tower-http`
- 异步运行时：`tokio`
- 数据访问：`sea-orm`
- 迁移：`sea-orm-migration`
- 序列化：`serde` + `serde_json`
- OpenAPI：`utoipa` + `utoipa-axum`
- 请求校验：`validator`
- 时间处理：`time`
- 唯一 ID：`uuid`，统一使用 `UUID v7`
- 错误处理：`thiserror`，启动与集成层可局部使用 `anyhow`
- 日志与追踪：`tracing` + `tracing-subscriber`
- 配置：`config` + `dotenvy`

推荐原因：

- `axum + tower-http` 足够轻量，同时能自然承接路由、中间件、请求 ID、CORS、trace 等通用能力。
- `SeaORM` 对 SQLite 友好，Entity、Relation、Migration 是同一套工具链，适合当前项目从骨架到可维护 V1 的演进路径。
- `utoipa` 适合和 `axum`、DTO、错误模型一起维护统一的 OpenAPI 契约，能避免接口代码和文档双份漂移。
- `validator` 适合承接 DTO 层校验，领域规则继续放在 `app service`，职责边界清晰。
- `time + UUID v7` 有利于统一时间格式与排序语义，避免 `chrono` 和随机 UUID 带来的风格分裂。
- `tracing` 能直接打通 HTTP 日志、SQL 慢查询、业务事件，后续迁移到多用户或远程部署也不需要推翻。

后端技术选项指导：

- Web 框架优先 `axum`。只有在团队已经深度熟悉 `actix-web`，或者明确需要其生态中的特定能力时，才考虑替换。
- 数据访问优先 `SeaORM`。如果后续读模型明显变复杂、出现大量手写 SQL 和复杂聚合，再局部引入 `SQLx` 辅助查询，而不是在 V1 直接切换整套方案。
- 后端接口必须采用 `OpenAPI 3.1`。推荐通过 `utoipa` 从路由、DTO、错误模型自动生成，不再接受只维护 Markdown 而不生成机器可读契约的方案。
- 后端必须同时提供运行时 `GET /api/v1/openapi.json` 和仓库产物 `api/docs/openapi.json`，两者必须来自同一份 `ApiDoc` 定义。
- 配置优先环境变量 + `.env` 文件，不引入额外配置中心。
- V1 不引入 GraphQL、gRPC、消息队列、事件总线、CQRS、缓存中间件。当前系统是单机场景，这些技术会增加心智负担，但不会实质提升交付速度。

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
├── bin/
├── config/
├── routes/
├── dto/
├── openapi/
├── app/
├── domain/
├── repository/
├── db/
└── error/
```

各层职责：

- `routes/`：HTTP 路由与请求响应映射。
- `dto/`：API 请求与响应结构体。
- `bin/`：放置 `export_openapi.rs` 等独立导出或维护型命令。
- `openapi/`：统一定义 `ApiDoc`、schema 组装和导出逻辑。
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
  FOREIGN KEY (project_id, status_id)
    REFERENCES task_statuses(project_id, id)
    ON DELETE RESTRICT
);
```

建议索引：

```sql
CREATE UNIQUE INDEX uq_tasks_project_id_id
ON tasks(project_id, id);

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
  project_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (task_id, tag_id),
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (project_id, task_id)
    REFERENCES tasks(project_id, id)
    ON DELETE CASCADE,
  FOREIGN KEY (project_id, tag_id)
    REFERENCES tags(project_id, id)
    ON DELETE CASCADE
);
```

### 8.6 task_relations

```sql
CREATE TABLE task_relations (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  related_task_id TEXT NOT NULL,
  relation_type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (project_id, task_id)
    REFERENCES tasks(project_id, id)
    ON DELETE CASCADE,
  FOREIGN KEY (project_id, related_task_id)
    REFERENCES tasks(project_id, id)
    ON DELETE CASCADE
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

API 风格采用 `REST + JSON`，并在 V1 就统一版本、字段命名和错误模型。

OpenAPI 实施标准见 [openapi-standard.md](/var/tmp/vibe-kanban/worktrees/62de-/plan/api/docs/openapi-standard.md)。本章描述的是接口设计约束，最终机器可读契约以自动生成的 `api/docs/openapi.json` 为准。

统一前缀建议：

- 本地开发：`http://localhost:3001/api/v1`
- 生产部署：由反向代理统一暴露 `/api/v1/*`

### 9.1 总体规范

- URL 使用资源名复数，例如 `projects`、`statuses`、`tasks`。
- JSON 字段统一使用 `camelCase`，数据库字段继续使用 `snake_case`；Rust DTO 统一加 `#[serde(rename_all = "camelCase")]`。
- 所有公开接口都必须出现在 OpenAPI 中，并通过统一的 `ApiDoc` 自动导出。
- `id` 统一为字符串型 `UUID v7`。
- `createdAt`、`updatedAt`、`completedAt`、`archivedAt` 使用 UTC RFC 3339 时间字符串。
- `startDate`、`dueDate` 使用 `YYYY-MM-DD`，仅表达日期，不表达时区。
- 成功响应统一返回 `data`，列表接口可额外返回 `meta`。
- 失败响应统一返回 `error` 对象，包含 `code`、`message`、`details`、`requestId`。
- `PATCH` 采用部分更新语义，只提交变化字段；字段显式传 `null` 表示清空可空值。
- 列表接口统一支持 `page`、`pageSize`；筛选接口补充 `sortBy`、`sortOrder`。

成功响应示例：

```json
{
  "data": {
    "id": "018f5f24-5c6d-7b9e-a69b-5fe1e14715b2",
    "name": "Plan"
  },
  "meta": {
    "requestId": "req_01hsz3m8h7"
  }
}
```

错误响应示例：

```json
{
  "error": {
    "code": "validation_error",
    "message": "title must not be empty",
    "details": {
      "field": "title"
    },
    "requestId": "req_01hsz3m8h7"
  }
}
```

### 9.2 核心 DTO 约定

- `ProjectDto`：`id`、`name`、`slug`、`description`、`color`、`createdAt`、`updatedAt`
- `StatusDto`：`id`、`projectId`、`name`、`color`、`sortOrder`、`isDone`、`isHidden`、`createdAt`、`updatedAt`
- `TaskDto`：`id`、`projectId`、`statusId`、`title`、`description`、`priority`、`position`、`startDate`、`dueDate`、`completedAt`、`archivedAt`、`tagIds`、`createdAt`、`updatedAt`
- `CommentDto`：`id`、`taskId`、`authorName`、`content`、`createdAt`、`updatedAt`
- `TagDto`：`id`、`projectId`、`name`、`color`、`createdAt`、`updatedAt`

### 9.3 项目接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/v1/projects?page=1&pageSize=20` | 获取项目列表 |
| `POST` | `/api/v1/projects` | 创建项目，并初始化默认列 |
| `GET` | `/api/v1/projects/{projectId}` | 获取项目详情 |
| `PATCH` | `/api/v1/projects/{projectId}` | 更新项目 |
| `DELETE` | `/api/v1/projects/{projectId}` | 删除项目，级联清理下游数据 |

`POST /api/v1/projects` 请求体建议：

```json
{
  "name": "Plan",
  "slug": "plan",
  "description": "Local kanban",
  "color": "#2563eb"
}
```

### 9.4 看板快照接口

主看板首屏统一使用快照接口，不拆多次请求。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/v1/projects/{projectId}/board?includeArchived=false` | 获取项目看板快照 |

返回建议：

```json
{
  "data": {
    "project": {},
    "statuses": [],
    "tasks": [],
    "tags": [],
    "taskTags": [],
    "summary": {
      "activeTaskCount": 0,
      "doneTaskCount": 0,
      "archivedTaskCount": 0
    }
  },
  "meta": {
    "requestId": "req_01hsz3m8h7"
  }
}
```

设计要求：

- 一次请求返回渲染看板所需的核心数据，避免首页 N+1 请求。
- 响应保持扁平化和规范化，前端根据 `statusId`、`tagIds` 自行分组，不在接口层嵌套整棵列树。
- `board` 接口服务于主视图；搜索、筛选、分页统一走任务列表接口，不再单独设计 `/search` 路径。

### 9.5 状态列接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/v1/projects/{projectId}/statuses` | 获取状态列列表 |
| `POST` | `/api/v1/projects/{projectId}/statuses` | 创建状态列 |
| `PATCH` | `/api/v1/statuses/{statusId}` | 更新状态列 |
| `DELETE` | `/api/v1/statuses/{statusId}` | 删除状态列，列下有未归档任务时返回 `409` |
| `POST` | `/api/v1/projects/{projectId}/statuses/reorder` | 重排列顺序 |

`POST /api/v1/projects/{projectId}/statuses/reorder` 请求体建议：

```json
{
  "orderedStatusIds": [
    "status_todo",
    "status_doing",
    "status_done"
  ]
}
```

### 9.6 任务接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/v1/projects/{projectId}/tasks` | 任务列表、搜索、筛选、分页 |
| `POST` | `/api/v1/projects/{projectId}/tasks` | 创建任务 |
| `GET` | `/api/v1/tasks/{taskId}` | 获取任务详情 |
| `PATCH` | `/api/v1/tasks/{taskId}` | 更新任务 |
| `DELETE` | `/api/v1/tasks/{taskId}` | 物理删除任务，仅建议用于已归档任务 |
| `POST` | `/api/v1/tasks/{taskId}/archive` | 归档任务 |
| `POST` | `/api/v1/tasks/{taskId}/restore` | 恢复任务 |

`GET /api/v1/projects/{projectId}/tasks` 查询参数建议：

- `q`：关键字，匹配标题和描述
- `statusId`
- `priority`
- `tagId`
- `archived`：`exclude | only | include`
- `page`
- `pageSize`
- `sortBy`：`updatedAt | dueDate | createdAt | position`
- `sortOrder`：`asc | desc`

`POST /api/v1/projects/{projectId}/tasks` 请求体建议：

```json
{
  "statusId": "status_todo",
  "title": "Define backend API",
  "description": "Finalize REST contracts for board and tasks",
  "priority": "high",
  "startDate": "2026-04-18",
  "dueDate": "2026-04-21",
  "tagIds": [
    "tag_backend"
  ]
}
```

`PATCH /api/v1/tasks/{taskId}` 请求体建议：

```json
{
  "title": "Define stable backend API",
  "description": "Updated description",
  "priority": "urgent",
  "statusId": "status_doing",
  "dueDate": null,
  "tagIds": [
    "tag_backend",
    "tag_api"
  ]
}
```

说明：

- `tagIds` 在创建和更新任务时都允许整组提交，后端在事务内做关联替换。
- 任务进入 `isDone = true` 的列时，后端自动写入 `completedAt`；离开完成列时自动清空。
- 常规用户流转优先使用 `archive/restore`；`DELETE` 更偏向维护接口。

### 9.7 任务拖拽排序接口

看板拖拽单独建接口，避免把排序语义塞进通用 `PATCH`。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/api/v1/projects/{projectId}/tasks/reorder` | 移动任务并重算列内顺序 |

请求体建议：

```json
{
  "movedTaskId": "task_xxx",
  "sourceStatusId": "status_todo",
  "destinationStatusId": "status_doing",
  "orderedTaskIds": [
    "task_2",
    "task_xxx",
    "task_9"
  ]
}
```

后端要求：

- 在单事务内校验任务、源列、目标列都属于同一项目。
- 重写目标列顺序；如果跨列移动，同时重写源列剩余任务顺序。
- 只更新真实发生变化的记录，避免整列无差别写入。
- 同步维护 `completedAt` 与 `activityLogs`。

### 9.8 评论接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/v1/tasks/{taskId}/comments` | 获取评论列表，按 `createdAt ASC` |
| `POST` | `/api/v1/tasks/{taskId}/comments` | 新增评论 |
| `PATCH` | `/api/v1/comments/{commentId}` | 编辑评论 |
| `DELETE` | `/api/v1/comments/{commentId}` | 删除评论 |

`POST /api/v1/tasks/{taskId}/comments` 请求体建议：

```json
{
  "authorName": "system",
  "content": "Initial note"
}
```

### 9.9 标签接口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/v1/projects/{projectId}/tags` | 获取标签列表 |
| `POST` | `/api/v1/projects/{projectId}/tags` | 创建标签 |
| `PATCH` | `/api/v1/tags/{tagId}` | 更新标签 |
| `DELETE` | `/api/v1/tags/{tagId}` | 删除标签 |
| `PUT` | `/api/v1/tasks/{taskId}/tags/{tagId}` | 绑定标签，要求幂等 |
| `DELETE` | `/api/v1/tasks/{taskId}/tags/{tagId}` | 解绑标签 |

说明：

- 任务标签绑定推荐使用 `PUT`，因为“绑定同一标签”天然是幂等动作。
- 如果前端已经拿到完整 `tagIds`，优先通过 `PATCH /tasks/{taskId}` 一次性提交；`PUT/DELETE` 适合细粒度交互。

### 9.10 状态码与错误码

建议统一使用：

- `200 OK`：普通读取、更新、归档、恢复、重排
- `201 Created`：创建成功
- `204 No Content`：删除成功
- `400 Bad Request`：JSON 结构错误、参数类型错误
- `404 Not Found`：资源不存在
- `409 Conflict`：唯一键冲突、删除列时仍有任务、非法状态迁移
- `422 Unprocessable Entity`：业务字段校验失败
- `500 Internal Server Error`：未处理错误

建议保留的错误码枚举：

- `validation_error`
- `not_found`
- `conflict`
- `invalid_operation`
- `internal_error`

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
2. 调用 `GET /api/v1/projects/:projectId/board`
3. 前端按 `statuses + tasks` 分组渲染列与卡片
4. 点击卡片后，更新 `taskId` 参数并加载任务详情

拖拽任务：

1. 前端本地乐观更新列内任务顺序
2. 调用 `POST /api/v1/projects/:projectId/tasks/reorder`
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
- 注册 `GET /api/v1/openapi.json`，返回运行时生成的 OpenAPI 文档

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
- `InvalidOperationError`
- `InternalError`

HTTP 映射：

- `400`：JSON 结构错误、参数类型错误
- `404`：资源不存在
- `409`：排序冲突、唯一性冲突、删除前置条件不满足
- `422`：业务字段校验失败
- `500`：服务内部错误

错误响应示例：

```json
{
  "error": {
    "code": "validation_error",
    "message": "title must not be empty",
    "details": {
      "field": "title"
    },
    "requestId": "req_01hsz3m8h7"
  }
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
- 前端通过 `next.config.ts` rewrites 代理 `/api/v1/*`
- 后端提供 `GET /api/v1/openapi.json` 供前端联调、Mock 和调试使用

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
- OpenAPI JSON 必须由代码自动导出，避免人工维护的契约漂移。

对于 SQLite 运行建议：

- 开启 WAL 模式。
- 统一通过连接池访问数据库。
- 启动时自动执行 migration。

对 OpenAPI 运行建议：

- 启动时注册 `GET /api/v1/openapi.json`
- 提供独立导出命令生成 `api/docs/openapi.json`
- 在 CI 中校验 `api/docs/openapi.json` 是否与当前代码生成结果一致

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
