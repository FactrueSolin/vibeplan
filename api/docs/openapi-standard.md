# Plan 后端 OpenAPI 接口标准

## 1. 目标

本规范用于约束 `plan` 后端接口的 OpenAPI 实现方式，确保以下目标同时成立：

- OpenAPI 是后端接口的正式契约，而不是事后补文档
- 后端代码变更后，可以自动生成最新的 `openapi.json`
- 前端、测试、未来 SDK 生成都基于同一份规范
- Markdown 设计文档和 OpenAPI 契约保持一致，但以 OpenAPI 为最终机器可读标准

本文适用于 `plan/api` Rust 后端。

## 2. 总体要求

### 2.1 必须使用 OpenAPI 3.1

- 后端接口规范统一采用 `OpenAPI 3.1.x`
- 输出格式统一为 JSON
- 文档版本号与后端 API 主版本保持一致，例如当前为 `v1`

### 2.2 OpenAPI 是接口单一事实来源

- 对外 HTTP 接口的路径、方法、参数、请求体、响应体、错误码，都必须在 OpenAPI 中有定义
- 新增、修改、删除接口时，必须同步更新 OpenAPI 注解和导出产物
- Markdown 文档用于解释设计原则和业务语义，不作为最终机器消费契约

### 2.3 必须自动生成 `openapi.json`

后端必须同时提供两种 OpenAPI 输出方式：

- 运行时暴露：`GET /api/v1/openapi.json`
- 构建或开发命令导出：生成仓库内文件 `api/docs/openapi.json`

要求：

- 两种输出内容必须来自同一份 `OpenApi` 定义
- 不允许手写或人工维护 `openapi.json`
- 不允许出现“运行时返回一份，仓库文件又是另一份”的双源问题

## 3. 技术方案

### 3.1 推荐实现

后端推荐使用以下组合：

- `utoipa`
- `utoipa-axum`
- `utoipa::OpenApi` derive

原因：

- 与 `axum` 配合自然
- 可以直接从 DTO、枚举、路径注解生成 schema
- 适合当前 Rust 技术栈，不需要额外维护 YAML 文件

### 3.2 不推荐方案

以下方案在当前项目阶段不推荐采用：

- 手写 `openapi.yaml` 再与代码分离维护
- 先写 Markdown，再人工转换 OpenAPI
- 只暴露 Swagger UI，不产出独立 JSON 文件
- 把 OpenAPI 生成放到外部脚本，脱离 Rust 类型系统

这些方案都会增加漂移风险。

## 4. 目录与文件约定

建议目录：

```text
api/
├── docs/
│   ├── architecture.md
│   ├── database-design.md
│   ├── openapi-standard.md
│   └── openapi.json
└── src/
    ├── dto/
    ├── routes/
    ├── openapi.rs
    └── bin/
        └── export_openapi.rs
```

约定：

- `src/openapi.rs`：统一定义 `ApiDoc`
- `src/routes/`：每个路由处理函数附带 `utoipa::path` 注解
- `src/dto/`：请求与响应 DTO 派生 OpenAPI schema
- `src/bin/export_openapi.rs`：导出 `api/docs/openapi.json`
- `api/docs/openapi.json`：自动生成产物，可提交到仓库

## 5. OpenAPI 内容标准

### 5.1 基本信息

OpenAPI 顶层信息至少包含：

- `title`: `Plan API`
- `version`: `v1`
- `description`: 简短说明本地任务管理系统 API

### 5.2 Servers

至少声明：

- `http://localhost:3001/api/v1`

如果后续有生产环境，可再追加生产地址；本地地址必须保留，方便调试与联调。

### 5.3 Tags

接口按领域分组，至少使用以下 tags：

- `Projects`
- `Statuses`
- `Tasks`
- `Comments`
- `Tags`
- `System`

说明：

- `System` 用于健康检查、OpenAPI 文档等系统接口
- 不要按技术层分 tag，例如 `CRUD`、`Internal`

### 5.4 OperationId

每个接口必须定义稳定的 `operationId`。

推荐命名：

- `listProjects`
- `createProject`
- `getProject`
- `updateProject`
- `deleteProject`
- `getBoardSnapshot`
- `listProjectStatuses`
- `reorderProjectStatuses`
- `listProjectTasks`
- `createProjectTask`
- `reorderProjectTasks`

要求：

- 同一版本内不能重复
- 不要把实现细节写进名字，例如 `handleGetProject`

## 6. Schema 标准

### 6.1 字段命名

- OpenAPI 中所有 JSON 字段统一使用 `camelCase`
- Rust struct 字段可用 `snake_case`，通过 `serde(rename_all = "camelCase")` 暴露

### 6.2 时间与日期

- `createdAt`、`updatedAt`、`completedAt`、`archivedAt` 使用 `type: string` + `format: date-time`
- `startDate`、`dueDate` 使用 `type: string` + `format: date`

### 6.3 ID

- 所有资源主键字段统一为 `type: string`
- description 中明确说明是 `UUID v7`

### 6.4 枚举

以下字段必须建成显式 enum，而不是普通 string：

- `priority`
- `archived` 查询参数
- `sortBy`
- `sortOrder`
- `error.code` 可选枚举值

### 6.5 可空字段

- 可空字段必须在 schema 中明确标记为 nullable
- 不允许仅靠示例暗示字段可能为 null

## 7. 路径与参数标准

### 7.1 路径风格

- 统一使用 `/api/v1`
- 资源名使用复数
- 路径参数使用 `{projectId}`、`{taskId}` 这类 camelCase 形式

### 7.2 查询参数

查询参数必须定义：

- 名称
- 类型
- 是否必填
- 合法枚举值
- 默认值
- 含义说明

例如任务列表接口的：

- `page`
- `pageSize`
- `q`
- `statusId`
- `priority`
- `tagId`
- `archived`
- `sortBy`
- `sortOrder`

### 7.3 请求体

要求：

- `POST`、`PATCH` 的 body 必须显式声明 schema
- 至少给出一个 example
- `PATCH` 只描述允许更新的字段，不要直接复用完整实体 schema

## 8. 响应标准

### 8.1 成功响应

统一使用：

```json
{
  "data": {},
  "meta": {
    "requestId": "req_01hsz3m8h7"
  }
}
```

列表响应可增加：

```json
{
  "data": [],
  "meta": {
    "page": 1,
    "pageSize": 20,
    "total": 100,
    "requestId": "req_01hsz3m8h7"
  }
}
```

### 8.2 错误响应

统一错误结构：

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

所有公开接口都必须在 OpenAPI 中声明常见错误响应：

- `400`
- `404`
- `409`
- `422`
- `500`

## 9. 自动生成要求

### 9.1 运行时导出

应用启动后必须注册：

- `GET /api/v1/openapi.json`

要求：

- 返回 `application/json`
- 内容直接来源于 `ApiDoc::openapi()`
- 不通过手写静态文件返回

### 9.2 文件导出

后端必须提供导出命令，例如：

```bash
cargo run --bin export_openapi
```

该命令负责：

- 构建 `ApiDoc`
- 序列化为 JSON
- 写入 `api/docs/openapi.json`

### 9.3 开发与 CI 约束

建议增加以下流程：

- 本地开发时，在接口变更后执行一次 OpenAPI 导出
- CI 中校验 `api/docs/openapi.json` 是否与当前代码生成结果一致

推荐校验思路：

1. 运行导出命令到临时文件
2. 与仓库内 `api/docs/openapi.json` 对比
3. 不一致则 CI 失败

这样可以防止接口代码变了但产物没更新。

## 10. 与架构文档的关系

- [architecture.md](/var/tmp/vibe-kanban/worktrees/62de-/plan/api/docs/architecture.md) 负责说明整体架构、后端选型、接口分层与 API 范围
- 本文负责说明 OpenAPI 的实现标准和产物要求
- 接口路径、请求体、响应体以 `OpenAPI JSON` 为最终机器可读契约

## 11. 最终要求

对于 `plan` 后端，OpenAPI 不是可选增强，而是默认交付物：

- 必须在代码中声明 OpenAPI
- 必须自动生成 `api/docs/openapi.json`
- 必须暴露 `/api/v1/openapi.json`
- 必须保持与实际后端路由一致

只有满足这四点，前后端接口规范才算真正落地。
