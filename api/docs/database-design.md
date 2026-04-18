# Plan 数据库设计方案

## 1. 文档目标

本文基于 [architecture.md](/home/tianci/plan/api/docs/architecture.md) 的领域模型，给出一份可直接落地到 `SeaORM + SQLite` 的数据库设计方案。

目标：

- 明确 V1 需要落库的实体、字段、关系、约束与索引。
- 明确哪些一致性由数据库保证，哪些规则由应用服务保证。
- 明确 `SeaORM Entity`、`Migration`、SQLite 运行参数的推荐实现方式。

本文默认前提：

- 单机部署
- SQLite 文件数据库，例如 `data/plan.db`
- 后端是唯一写入口
- 使用 `SeaORM` 做实体映射和查询
- 使用 `sea-orm-migration` 管理 schema 迁移

## 2. 设计原则

### 2.1 主键与 ID

- 所有主键统一使用 `TEXT` 类型存储 `UUID v7`，如果当前库尚未接入 v7，也可先使用 `UUID v4`。
- 不使用 SQLite 自增整数主键，避免未来导入导出、离线创建记录、跨环境合并时受限。
- 所有外键字段与主键字段保持同一存储类型，统一为 `TEXT`。

### 2.2 时间字段

- 所有时间字段统一使用 `TEXT` 存储 UTC 时间，格式为 RFC 3339，例如 `2026-04-18T10:30:45Z`。
- `created_at`、`updated_at`、`completed_at`、`archived_at` 全部由服务端生成，不依赖数据库默认时间。
- 只要统一使用 UTC RFC 3339，字符串比较即可满足大多数时间排序和 `CHECK` 判断。

### 2.3 布尔与枚举

- SQLite 无原生布尔类型，统一使用 `INTEGER`，并通过 `CHECK (value IN (0, 1))` 约束。
- 枚举统一使用 `TEXT` 存储，并用 `CHECK` 约束合法值。
- `SeaORM` 中使用 `DeriveActiveEnum` 维护 Rust 枚举与数据库字符串的映射。

### 2.4 排序策略

- 列顺序使用 `task_statuses.sort_order`。
- 列内任务顺序使用 `tasks.position`。
- `sort_order` 和 `position` 都不加唯一约束，避免拖拽重排时产生瞬时唯一冲突。
- 创建新列或新任务时，建议采用稀疏步长，例如 `MAX(sort_order) + 1000`、`MAX(position) + 1000`。
- 当序号过于稠密时，由应用服务在事务内做一次规范化重排。

### 2.5 删除与归档策略

- 删除 `project` 时允许级联删除下游数据。
- 任务的业务删除优先使用归档，即设置 `archived_at`，而不是物理删除。
- 删除状态列时使用 `ON DELETE RESTRICT`，必须先迁移或清空该列下任务。
- 评论、标签关联、任务关系、活动日志可以随任务或项目级联删除。

### 2.6 一致性下沉原则

能用数据库约束保证的规则，尽量不要只留在应用层。

本方案中，以下一致性会直接下沉到数据库：

- 任务所属状态列必须属于同一项目
- 任务与标签关联必须属于同一项目
- 任务关系的两个任务必须属于同一项目

实现方式不是 trigger，而是组合外键加冗余 `project_id`。

## 3. V1 / V2 范围

### 3.1 V1 必落表

- `projects`
- `task_statuses`
- `tasks`
- `task_comments`
- `tags`
- `task_tags`
- `activity_logs`

### 3.2 V2 可选表

- `task_relations`
- `subtasks`
- `users`
- `task_assignees`
- `attachments`
- `saved_filters`
- `notifications`

说明：

- `task_relations` 在架构设计中已经定义了领域模型，因此本文给出完整结构。
- 如果当前目标是优先打通主链路，可以把 `task_relations` 放到 V2 migration 再落。

## 4. 实体关系

```text
projects 1 ── N task_statuses
projects 1 ── N tasks
projects 1 ── N tags
projects 1 ── N activity_logs

task_statuses 1 ── N tasks
tasks 1 ── N task_comments
tasks N ── N tags                (through task_tags)
tasks N ── N tasks               (through task_relations, V2)
tasks 1 ── N activity_logs
```

额外约束：

- `tasks.project_id` 必须和所属 `task_statuses.project_id` 一致
- `task_tags.project_id` 必须和 `tasks.project_id`、`tags.project_id` 一致
- `task_relations.project_id` 必须和两个任务的 `project_id` 一致

## 5. 物理表设计

### 5.1 projects

用途：项目主表，对应一个看板空间。

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL CHECK (trim(name) <> ''),
  slug TEXT NOT NULL UNIQUE CHECK (trim(slug) <> ''),
  description TEXT,
  color TEXT NOT NULL DEFAULT '#2563eb',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

建议索引：

```sql
CREATE INDEX idx_projects_updated_at
ON projects(updated_at DESC);
```

说明：

- `slug` 是否统一小写、是否允许中划线之外字符，交给应用层规范化。
- `name` 不做全局唯一，允许重名项目。

### 5.2 task_statuses

用途：项目中的看板列。

```sql
CREATE TABLE task_statuses (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL CHECK (trim(name) <> ''),
  color TEXT NOT NULL,
  sort_order INTEGER NOT NULL,
  is_done INTEGER NOT NULL DEFAULT 0 CHECK (is_done IN (0, 1)),
  is_hidden INTEGER NOT NULL DEFAULT 0 CHECK (is_hidden IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

建议索引与约束：

```sql
CREATE UNIQUE INDEX uq_task_statuses_project_name
ON task_statuses(project_id, name);

CREATE UNIQUE INDEX uq_task_statuses_project_id_id
ON task_statuses(project_id, id);

CREATE INDEX idx_task_statuses_project_sort
ON task_statuses(project_id, sort_order ASC, created_at ASC);
```

说明：

- `uq_task_statuses_project_id_id` 看起来冗余，但它是后续 `(project_id, status_id)` 组合外键的基础。
- `sort_order` 不做唯一约束，避免拖拽改列顺序时增加写入复杂度。

### 5.3 tasks

用途：任务主表，是系统核心实体。

```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  status_id TEXT NOT NULL,
  title TEXT NOT NULL CHECK (trim(title) <> ''),
  description TEXT,
  priority TEXT CHECK (
    priority IS NULL OR priority IN ('low', 'medium', 'high', 'urgent')
  ),
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
    ON DELETE RESTRICT,
  CHECK (
    start_date IS NULL OR due_date IS NULL OR start_date <= due_date
  )
);
```

建议索引与约束：

```sql
CREATE UNIQUE INDEX uq_tasks_project_id_id
ON tasks(project_id, id);

CREATE INDEX idx_tasks_active_board
ON tasks(project_id, status_id, position ASC)
WHERE archived_at IS NULL;

CREATE INDEX idx_tasks_active_updated_at
ON tasks(project_id, updated_at DESC)
WHERE archived_at IS NULL;

CREATE INDEX idx_tasks_project_due_date
ON tasks(project_id, due_date ASC);

CREATE INDEX idx_tasks_project_archived_at
ON tasks(project_id, archived_at);
```

说明：

- 组合外键 `(project_id, status_id)` 直接保证任务只能挂在本项目的列下。
- `priority` 允许为空，表示未设置优先级。
- `completed_at` 由应用服务在任务进入或离开完成列时维护。
- 常用看板查询只关心未归档任务，因此主索引使用部分索引 `WHERE archived_at IS NULL`。

### 5.4 task_comments

用途：任务评论。

```sql
CREATE TABLE task_comments (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  author_name TEXT NOT NULL CHECK (trim(author_name) <> ''),
  content TEXT NOT NULL CHECK (trim(content) <> ''),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
```

建议索引：

```sql
CREATE INDEX idx_task_comments_task_created_at
ON task_comments(task_id, created_at ASC);
```

说明：

- V1 不引入完整用户表，因此只保留 `author_name`。
- 如果后续接入用户体系，可新增 `author_user_id`，保留 `author_name` 作为展示快照。

### 5.5 tags

用途：项目级标签定义。

```sql
CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  name TEXT NOT NULL CHECK (trim(name) <> ''),
  color TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);
```

建议索引与约束：

```sql
CREATE UNIQUE INDEX uq_tags_project_name
ON tags(project_id, name);

CREATE UNIQUE INDEX uq_tags_project_id_id
ON tags(project_id, id);

CREATE INDEX idx_tags_project_updated_at
ON tags(project_id, updated_at DESC);
```

说明：

- 标签名在同一项目内唯一，避免同名不同色。
- `uq_tags_project_id_id` 用于给 `task_tags` 提供组合外键目标。

### 5.6 task_tags

用途：任务与标签的多对多关系表。

```sql
CREATE TABLE task_tags (
  project_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
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

建议索引：

```sql
CREATE INDEX idx_task_tags_project_tag
ON task_tags(project_id, tag_id);
```

说明：

- 这里显式保存 `project_id`，是为了让“任务与标签必须属于同一项目”变成数据库可校验规则，而不是只靠应用层。
- `PRIMARY KEY (task_id, tag_id)` 足以保证一张任务不会重复绑定同一标签。

### 5.7 task_relations

用途：任务间关系，例如阻塞和关联。该表建议作为 V2 能力落地。

```sql
CREATE TABLE task_relations (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  related_task_id TEXT NOT NULL,
  relation_type TEXT NOT NULL CHECK (
    relation_type IN ('blocking', 'related')
  ),
  created_at TEXT NOT NULL,
  FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
  FOREIGN KEY (project_id, task_id)
    REFERENCES tasks(project_id, id)
    ON DELETE CASCADE,
  FOREIGN KEY (project_id, related_task_id)
    REFERENCES tasks(project_id, id)
    ON DELETE CASCADE,
  CHECK (task_id <> related_task_id),
  CHECK (
    relation_type <> 'related' OR task_id < related_task_id
  )
);
```

建议索引与约束：

```sql
CREATE UNIQUE INDEX uq_task_relations_pair_type
ON task_relations(task_id, related_task_id, relation_type);

CREATE INDEX idx_task_relations_related_task
ON task_relations(project_id, related_task_id);
```

说明：

- `project_id` 的设计原因和 `task_tags` 一样，是为了把“同项目关系”下沉到数据库。
- `blocking` 是有方向的。
- `related` 在业务上是对称关系，因此约束 `task_id < related_task_id`，避免同时出现 A-B 与 B-A 两条重复记录。

### 5.8 activity_logs

用途：记录关键领域事件，为后续时间线和审计提供基础。

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

建议索引：

```sql
CREATE INDEX idx_activity_logs_project_created_at
ON activity_logs(project_id, created_at DESC);

CREATE INDEX idx_activity_logs_task_created_at
ON activity_logs(task_id, created_at DESC);
```

说明：

- `event_type` 建议先用字符串，不做强枚举，避免每次新增事件都需要 migration。
- `payload_json` 用于存储字段变更前后值、排序变更详情、评论摘要等上下文。

## 6. 由数据库保证的规则

以下规则可以直接依赖 schema 保证：

- 项目删除后，状态列、任务、标签、评论、活动日志会自动级联删除
- 任务只能引用本项目下的状态列
- `task_tags` 只能关联同一项目下的任务和标签
- `task_relations` 只能关联同一项目下的两个任务
- 布尔字段只能取 `0 / 1`
- `priority`、`relation_type` 只能取约定枚举值
- 任务标题、项目名、列名、标签名、评论内容不能为空字符串
- `due_date` 不能早于 `start_date`

## 7. 由应用服务保证的规则

以下规则不建议依赖 SQLite trigger，而应由 `app service + transaction` 保证：

### 7.1 slug 规范化

- 创建和更新项目时，统一规范 `slug` 的大小写、分隔符和保留词。

### 7.2 完成时间维护

- 任务移动到 `is_done = 1` 的列时，自动写入 `completed_at`
- 任务从完成列移出时，清空 `completed_at`
- 只修改标题、描述、标签、评论时，不应影响 `completed_at`

### 7.3 拖拽重排

- 校验提交的任务或列集合是否合法
- 更新移动目标的 `status_id`
- 重写受影响记录的 `position` 或 `sort_order`
- 写入 `activity_logs`

### 7.4 删除列前的业务校验

- 如果列下仍有未归档任务，不允许直接删除
- 如果未来支持“删除列并转移到其他列”，应在单事务内批量迁移任务后再删除

### 7.5 活动事件写入

- `activity_logs` 是业务审计数据，何时写入、写入什么 payload，交给应用服务显式控制

## 8. SeaORM 映射建议

### 8.1 推荐目录

建议后端结构如下：

```text
api/
├── migration/
│   ├── Cargo.toml
│   └── src/
├── src/
│   ├── entity/
│   ├── repository/
│   ├── app/
│   ├── routes/
│   └── ...
└── Cargo.toml
```

说明：

- `migration` 单独成 crate，职责更清晰。
- `src/entity/` 只放 `SeaORM Entity` 和关系定义，不承载业务逻辑。
- 事务、拖拽重排、跨表聚合放在 `repository/` 和 `app/`。

### 8.2 Entity 文件建议

- `src/entity/project.rs`
- `src/entity/task_status.rs`
- `src/entity/task.rs`
- `src/entity/task_comment.rs`
- `src/entity/tag.rs`
- `src/entity/task_tag.rs`
- `src/entity/task_relation.rs`
- `src/entity/activity_log.rs`
- `src/entity/prelude.rs`
- `src/entity/sea_orm_active_enums.rs`

### 8.3 ActiveEnum 建议

建议至少定义：

- `TaskPriority`: `low | medium | high | urgent`
- `TaskRelationType`: `blocking | related`

不建议 V1 就把 `activity_logs.event_type` 也做成强枚举，因为活动类型扩展频率较高。

### 8.4 关系建模建议

- `Project` has many `TaskStatus`
- `Project` has many `Task`
- `Project` has many `Tag`
- `Project` has many `ActivityLog`
- `TaskStatus` has many `Task`
- `Task` has many `TaskComment`
- `Task` has many `TaskTag`
- `Tag` has many `TaskTag`
- `TaskRelation` 对 `Task` 需要两条 `belongs_to`，分别区分 `Task` 和 `RelatedTask`

说明：

- `task_tags` 和 `task_relations` 带有冗余 `project_id`，主要是为了数据库一致性，不是为了 ORM 便利。
- 对这类表的复杂读写，建议在 repository 层写显式查询，不要强依赖深层预加载。

### 8.5 Migration 实现建议

- 普通表、普通索引、外键可以直接使用 `sea_orm_migration::prelude::*`
- SQLite 部分索引如 `WHERE archived_at IS NULL`，建议在 migration 中执行原生 SQL
- 创建表顺序必须先父表后子表，否则外键无法建立

## 9. Migration 顺序建议

建议按以下顺序创建 migration：

1. `m20260418_000001_create_projects`
2. `m20260418_000002_create_task_statuses`
3. `m20260418_000003_create_tasks`
4. `m20260418_000004_create_tags`
5. `m20260418_000005_create_task_tags`
6. `m20260418_000006_create_task_comments`
7. `m20260418_000007_create_activity_logs`
8. `m20260418_000008_create_task_relations`

说明：

- `task_relations` 如果暂不实现，可直接延后到 V2。
- 默认列 `Todo / In Progress / Done` 不建议通过 migration 初始化，而应在“创建项目”用例中按业务插入。

## 10. SQLite 运行参数

应用启动后建议执行：

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA temp_store = MEMORY;
PRAGMA busy_timeout = 5000;
```

说明：

- `foreign_keys = ON` 是必须项，否则外键约束不会生效
- `WAL` 更适合本地应用的读写并发
- `busy_timeout` 可以降低短时写锁导致的失败概率

## 11. 最终结论

对于 `plan` 的 V1，最稳妥的数据库方案是：

- 以 `projects / task_statuses / tasks` 作为主链路
- 以 `task_comments / tags / task_tags / activity_logs` 作为增强能力
- 用组合外键把“任务属于本项目列”“任务与标签同项目”这类约束尽量下沉到数据库
- 用应用服务事务处理拖拽重排、完成时间维护、活动日志写入
- 用 `SeaORM Entity + Migration` 管理 schema，用 SQLite 的 `TEXT + CHECK + INDEX` 承载枚举、时间和排序

这样可以在保持模型简单的前提下，把关键完整性约束尽量前移到数据层，降低后续业务扩展时的数据污染风险。
