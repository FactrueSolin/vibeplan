import path from 'node:path';
import { promises as fs } from 'node:fs';

import { v7 as uuidv7 } from 'uuid';
import { z } from 'zod';

import {
  applyTaskFilters,
  applyTaskReorder,
  computeBoardSummary,
  countOverdueTasks,
  sortFilteredTasks,
  sortStatuses,
  sortTasks,
} from '@/lib/board-utils';
import type {
  BoardSnapshotDto,
  CommentDto,
  Priority,
  ProjectDto,
  ProjectListItemDto,
  StatusDto,
  TagDto,
  TaskDto,
  TaskListFilters,
  TaskTagDto,
} from '@/lib/types';

type MockDatabase = {
  projects: ProjectDto[];
  statuses: StatusDto[];
  tasks: TaskDto[];
  tags: TagDto[];
  taskTags: TaskTagDto[];
  comments: CommentDto[];
};

const databasePath = path.join(process.cwd(), 'data', 'plan-mock-db.json');

const dateSchema = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/)
  .or(z.literal(''));

const projectSchema = z.object({
  name: z.string().trim().min(1).max(64),
  slug: z.string().trim().min(1).max(64),
  description: z.string().trim().max(240).nullish(),
  color: z.string().trim().regex(/^#[0-9a-fA-F]{6}$/).optional(),
});

const taskSchema = z.object({
  statusId: z.string().uuid(),
  title: z.string().trim().min(1).max(120),
  description: z.string().trim().max(3000).nullish(),
  priority: z.enum(['low', 'medium', 'high', 'urgent']).optional(),
  startDate: dateSchema.nullish(),
  dueDate: dateSchema.nullish(),
  tagIds: z.array(z.string().uuid()).optional(),
});

const taskPatchSchema = z.object({
  statusId: z.string().uuid().optional(),
  title: z.string().trim().min(1).max(120).optional(),
  description: z.string().trim().max(3000).nullable().optional(),
  priority: z.enum(['low', 'medium', 'high', 'urgent']).optional(),
  startDate: dateSchema.nullable().optional(),
  dueDate: dateSchema.nullable().optional(),
  tagIds: z.array(z.string().uuid()).optional(),
});

const reorderSchema = z.object({
  movedTaskId: z.string().uuid(),
  sourceStatusId: z.string().uuid(),
  destinationStatusId: z.string().uuid(),
  orderedTaskIds: z.array(z.string().uuid()).min(1),
});

const commentSchema = z.object({
  authorName: z.string().trim().min(1).max(32),
  content: z.string().trim().min(1).max(1000),
});

let writeQueue = Promise.resolve();

function now() {
  return new Date().toISOString();
}

function seedDatabase(): MockDatabase {
  const createdAt = '2026-04-18T08:30:00.000Z';

  const projectAlphaId = uuidv7();
  const projectBetaId = uuidv7();
  const projectGammaId = uuidv7();

  const todoAlphaId = uuidv7();
  const doingAlphaId = uuidv7();
  const doneAlphaId = uuidv7();

  const todoBetaId = uuidv7();
  const doingBetaId = uuidv7();
  const doneBetaId = uuidv7();

  const todoGammaId = uuidv7();
  const doingGammaId = uuidv7();
  const doneGammaId = uuidv7();

  const tagUxId = uuidv7();
  const tagApiId = uuidv7();
  const tagInfraId = uuidv7();
  const tagDocsId = uuidv7();

  const taskAId = uuidv7();
  const taskBId = uuidv7();
  const taskCId = uuidv7();
  const taskDId = uuidv7();
  const taskEId = uuidv7();
  const taskFId = uuidv7();

  return {
    projects: [
      {
        id: projectAlphaId,
        name: 'Plan',
        slug: 'plan',
        description: '本地部署的项目任务系统，聚焦核心流转。',
        color: '#c76637',
        createdAt,
        updatedAt: '2026-04-18T10:20:00.000Z',
      },
      {
        id: projectBetaId,
        name: 'Remote Deploy',
        slug: 'remote-deploy',
        description: '发布链路与环境接入整理。',
        color: '#2f7f74',
        createdAt,
        updatedAt: '2026-04-17T14:10:00.000Z',
      },
      {
        id: projectGammaId,
        name: 'Docs Sprint',
        slug: 'docs-sprint',
        description: '梳理 API 契约、用户引导与空状态文案。',
        color: '#72865b',
        createdAt,
        updatedAt: '2026-04-16T09:00:00.000Z',
      },
    ],
    statuses: [
      createStatus(todoAlphaId, projectAlphaId, '待处理', '#d88c61', 0, false, createdAt),
      createStatus(doingAlphaId, projectAlphaId, '进行中', '#3d8f82', 1, false, createdAt),
      createStatus(doneAlphaId, projectAlphaId, '已完成', '#80906d', 2, true, createdAt),
      createStatus(todoBetaId, projectBetaId, '待处理', '#d88c61', 0, false, createdAt),
      createStatus(doingBetaId, projectBetaId, '进行中', '#3d8f82', 1, false, createdAt),
      createStatus(doneBetaId, projectBetaId, '已完成', '#80906d', 2, true, createdAt),
      createStatus(todoGammaId, projectGammaId, '待处理', '#d88c61', 0, false, createdAt),
      createStatus(doingGammaId, projectGammaId, '进行中', '#3d8f82', 1, false, createdAt),
      createStatus(doneGammaId, projectGammaId, '已完成', '#80906d', 2, true, createdAt),
    ],
    tags: [
      createTag(tagUxId, projectAlphaId, 'UX', '#f19a61', createdAt),
      createTag(tagApiId, projectAlphaId, 'API', '#70b7ad', createdAt),
      createTag(tagInfraId, projectAlphaId, 'Infra', '#9ca989', createdAt),
      createTag(tagDocsId, projectAlphaId, 'Docs', '#c6a86d', createdAt),
    ],
    tasks: [
      createTaskRecord({
        id: taskAId,
        projectId: projectAlphaId,
        statusId: todoAlphaId,
        title: '补齐看板主页面布局',
        description: '完成顶部工具栏、指标区和列布局，保证首屏结构稳定。',
        priority: 'urgent',
        position: 0,
        startDate: '2026-04-18',
        dueDate: '2026-04-19',
        completedAt: null,
        archivedAt: null,
        createdAt: '2026-04-18T09:00:00.000Z',
        updatedAt: '2026-04-18T09:40:00.000Z',
        tagIds: [tagUxId, tagApiId],
      }),
      createTaskRecord({
        id: taskBId,
        projectId: projectAlphaId,
        statusId: todoAlphaId,
        title: '实现空状态与首次引导',
        description: '项目为空时给出明确的创建路径和系统说明。',
        priority: 'medium',
        position: 1,
        startDate: null,
        dueDate: '2026-04-22',
        completedAt: null,
        archivedAt: null,
        createdAt: '2026-04-18T09:10:00.000Z',
        updatedAt: '2026-04-18T09:20:00.000Z',
        tagIds: [tagUxId, tagDocsId],
      }),
      createTaskRecord({
        id: taskCId,
        projectId: projectAlphaId,
        statusId: doingAlphaId,
        title: '任务详情侧栏编辑态',
        description: '显式保存、未保存变更提醒和评论输入要保持在同一上下文。',
        priority: 'high',
        position: 0,
        startDate: '2026-04-18',
        dueDate: '2026-04-20',
        completedAt: null,
        archivedAt: null,
        createdAt: '2026-04-18T08:40:00.000Z',
        updatedAt: '2026-04-18T10:15:00.000Z',
        tagIds: [tagUxId, tagInfraId],
      }),
      createTaskRecord({
        id: taskDId,
        projectId: projectAlphaId,
        statusId: doingAlphaId,
        title: '接入 mock API 契约',
        description: '项目列表、看板快照、任务详情和评论接口统一走同一响应格式。',
        priority: 'high',
        position: 1,
        startDate: '2026-04-18',
        dueDate: '2026-04-21',
        completedAt: null,
        archivedAt: null,
        createdAt: '2026-04-18T08:50:00.000Z',
        updatedAt: '2026-04-18T10:10:00.000Z',
        tagIds: [tagApiId, tagInfraId],
      }),
      createTaskRecord({
        id: taskEId,
        projectId: projectAlphaId,
        statusId: doneAlphaId,
        title: '默认状态列定义',
        description: '统一为待处理、进行中、已完成，降低首次使用门槛。',
        priority: 'low',
        position: 0,
        startDate: '2026-04-16',
        dueDate: '2026-04-17',
        completedAt: '2026-04-17T08:00:00.000Z',
        archivedAt: null,
        createdAt: '2026-04-16T06:00:00.000Z',
        updatedAt: '2026-04-17T08:00:00.000Z',
        tagIds: [tagDocsId],
      }),
      createTaskRecord({
        id: taskFId,
        projectId: projectAlphaId,
        statusId: doneAlphaId,
        title: '整理 UX 文档要点',
        description: '抽取首版页面结构、主链路和交互反馈要求。',
        priority: 'medium',
        position: 1,
        startDate: '2026-04-16',
        dueDate: '2026-04-17',
        completedAt: '2026-04-17T10:00:00.000Z',
        archivedAt: null,
        createdAt: '2026-04-16T08:00:00.000Z',
        updatedAt: '2026-04-17T10:00:00.000Z',
        tagIds: [tagDocsId, tagApiId],
      }),
    ],
    taskTags: [
      createTaskTag(projectAlphaId, taskAId, tagUxId),
      createTaskTag(projectAlphaId, taskAId, tagApiId),
      createTaskTag(projectAlphaId, taskBId, tagUxId),
      createTaskTag(projectAlphaId, taskBId, tagDocsId),
      createTaskTag(projectAlphaId, taskCId, tagUxId),
      createTaskTag(projectAlphaId, taskCId, tagInfraId),
      createTaskTag(projectAlphaId, taskDId, tagApiId),
      createTaskTag(projectAlphaId, taskDId, tagInfraId),
      createTaskTag(projectAlphaId, taskEId, tagDocsId),
      createTaskTag(projectAlphaId, taskFId, tagDocsId),
      createTaskTag(projectAlphaId, taskFId, tagApiId),
    ],
    comments: [
      createCommentRecord(taskCId, 'system', '保存策略先走显式保存，避免误改。'),
      createCommentRecord(taskCId, 'tianci', '右侧面板要在移动端切成全屏抽屉。'),
      createCommentRecord(taskDId, 'system', '统一成功响应包裹 data / meta。'),
    ],
  };
}

function createStatus(
  id: string,
  projectId: string,
  name: string,
  color: string,
  sortOrder: number,
  isDone: boolean,
  createdAt: string,
): StatusDto {
  return {
    id,
    projectId,
    name,
    color,
    sortOrder,
    isDone,
    isHidden: false,
    createdAt,
    updatedAt: createdAt,
  };
}

function createTag(
  id: string,
  projectId: string,
  name: string,
  color: string,
  createdAt: string,
): TagDto {
  return {
    id,
    projectId,
    name,
    color,
    createdAt,
    updatedAt: createdAt,
  };
}

function createTaskRecord(task: TaskDto): TaskDto {
  return task;
}

function createTaskTag(projectId: string, taskId: string, tagId: string): TaskTagDto {
  return {
    projectId,
    taskId,
    tagId,
  };
}

function createCommentRecord(taskId: string, authorName: string, content: string): CommentDto {
  const createdAt = now();

  return {
    id: uuidv7(),
    taskId,
    authorName,
    content,
    createdAt,
    updatedAt: createdAt,
  };
}

async function ensureDatabaseFile() {
  try {
    await fs.access(databasePath);
  } catch {
    await fs.mkdir(path.dirname(databasePath), { recursive: true });
    await fs.writeFile(databasePath, JSON.stringify(seedDatabase(), null, 2));
  }
}

async function readDatabase() {
  await ensureDatabaseFile();

  const file = await fs.readFile(databasePath, 'utf8');

  return JSON.parse(file) as MockDatabase;
}

async function writeDatabase(database: MockDatabase) {
  writeQueue = writeQueue.then(async () => {
    await fs.writeFile(databasePath, JSON.stringify(database, null, 2));
  });

  await writeQueue;

  return database;
}

export async function listProjects() {
  const database = await readDatabase();

  const projects = [...database.projects]
    .sort(
      (left, right) =>
        new Date(right.updatedAt).getTime() - new Date(left.updatedAt).getTime(),
    )
    .map((project): ProjectListItemDto => {
      const tasks = database.tasks.filter((task) => task.projectId === project.id);
      const statuses = database.statuses.filter((status) => status.projectId === project.id);
      const doneStatusIds = new Set(
        statuses.filter((status) => status.isDone).map((status) => status.id),
      );

      return {
        ...project,
        summary: {
          taskCount: tasks.filter((task) => !task.archivedAt).length,
          completedTaskCount: tasks.filter((task) => doneStatusIds.has(task.statusId)).length,
          overdueTaskCount: countOverdueTasks(tasks),
        },
      };
    });

  return projects;
}

export async function createProject(input: unknown) {
  const payload = projectSchema.parse(input);
  const database = await readDatabase();
  const createdAt = now();
  const projectId = uuidv7();

  const project: ProjectDto = {
    id: projectId,
    name: payload.name,
    slug: payload.slug,
    description: payload.description ?? null,
    color: payload.color ?? '#c76637',
    createdAt,
    updatedAt: createdAt,
  };

  database.projects.unshift(project);
  database.statuses.push(
    createStatus(uuidv7(), projectId, '待处理', '#d88c61', 0, false, createdAt),
    createStatus(uuidv7(), projectId, '进行中', '#3d8f82', 1, false, createdAt),
    createStatus(uuidv7(), projectId, '已完成', '#80906d', 2, true, createdAt),
  );

  await writeDatabase(database);

  const [createdProject] = (await listProjects()).filter((item) => item.id === projectId);

  return createdProject;
}

export async function getBoard(projectId: string, includeArchived: boolean) {
  const database = await readDatabase();
  const project = database.projects.find((item) => item.id === projectId);

  if (!project) {
    return null;
  }

  const statuses = sortStatuses(
    database.statuses.filter((status) => status.projectId === projectId && !status.isHidden),
  );
  const tasks = sortTasks(
    database.tasks.filter((task) =>
      includeArchived ? task.projectId === projectId : task.projectId === projectId && !task.archivedAt,
    ),
  );
  const tags = database.tags.filter((tag) => tag.projectId === projectId);
  const taskTags = database.taskTags.filter((link) => link.projectId === projectId);

  const snapshot: BoardSnapshotDto = {
    project,
    statuses,
    tasks,
    tags,
    taskTags,
    summary: computeBoardSummary({
      statuses,
      tasks,
    }),
  };

  return snapshot;
}

export async function listTasks(projectId: string, filters: TaskListFilters) {
  const snapshot = await getBoard(projectId, filters.archived !== 'exclude');

  if (!snapshot) {
    return null;
  }

  const tasks = sortFilteredTasks(applyTaskFilters(snapshot, filters), filters);

  return tasks;
}

export async function createTask(projectId: string, input: unknown) {
  const payload = taskSchema.parse(input);
  const database = await readDatabase();
  const project = database.projects.find((item) => item.id === projectId);
  const status = database.statuses.find(
    (item) => item.id === payload.statusId && item.projectId === projectId,
  );

  if (!project || !status) {
    return null;
  }

  const createdAt = now();
  const position = database.tasks.filter((task) => task.statusId === payload.statusId).length;
  const taskId = uuidv7();

  const task: TaskDto = {
    id: taskId,
    projectId,
    statusId: payload.statusId,
    title: payload.title,
    description: payload.description ?? null,
    priority: payload.priority ?? 'medium',
    position,
    startDate: payload.startDate ?? null,
    dueDate: payload.dueDate ?? null,
    completedAt: status.isDone ? createdAt : null,
    archivedAt: null,
    tagIds: payload.tagIds ?? [],
    createdAt,
    updatedAt: createdAt,
  };

  database.tasks.push(task);
  database.projects = database.projects.map((item) =>
    item.id === projectId ? { ...item, updatedAt: createdAt } : item,
  );
  database.taskTags = database.taskTags.filter((link) => link.taskId !== taskId);
  for (const tagId of task.tagIds) {
    database.taskTags.push(createTaskTag(projectId, taskId, tagId));
  }

  await writeDatabase(database);

  return task;
}

export async function reorderTasks(projectId: string, input: unknown) {
  const payload = reorderSchema.parse(input);
  const snapshot = await getBoard(projectId, true);

  if (!snapshot) {
    return null;
  }

  const nextSnapshot = applyTaskReorder(snapshot, payload);
  const database = await readDatabase();
  const taskMap = new Map(nextSnapshot.tasks.map((task) => [task.id, task]));

  database.tasks = database.tasks.map((task) => {
    const updatedTask = taskMap.get(task.id);

    return updatedTask ?? task;
  });
  database.projects = database.projects.map((project) =>
    project.id === projectId ? { ...project, updatedAt: now() } : project,
  );

  await writeDatabase(database);

  return getBoard(projectId, true);
}

export async function getTask(taskId: string) {
  const database = await readDatabase();

  return database.tasks.find((task) => task.id === taskId) ?? null;
}

export async function updateTask(taskId: string, input: unknown) {
  const payload = taskPatchSchema.parse(input);
  const database = await readDatabase();
  const task = database.tasks.find((item) => item.id === taskId);

  if (!task) {
    return null;
  }

  const nextStatusId = payload.statusId ?? task.statusId;
  const status = database.statuses.find((item) => item.id === nextStatusId);
  const updatedAt = now();

  if (!status) {
    return null;
  }

  const nextTask: TaskDto = {
    ...task,
    statusId: nextStatusId,
    title: payload.title ?? task.title,
    description:
      payload.description === undefined ? task.description : payload.description,
    priority: payload.priority ?? task.priority,
    startDate: payload.startDate === undefined ? task.startDate : payload.startDate,
    dueDate: payload.dueDate === undefined ? task.dueDate : payload.dueDate,
    tagIds: payload.tagIds ?? task.tagIds,
    completedAt: status.isDone ? task.completedAt ?? updatedAt : null,
    updatedAt,
  };

  database.tasks = database.tasks.map((item) => (item.id === taskId ? nextTask : item));
  if (payload.tagIds) {
    database.taskTags = database.taskTags.filter((link) => link.taskId !== taskId);
    for (const tagId of payload.tagIds) {
      database.taskTags.push(createTaskTag(task.projectId, taskId, tagId));
    }
  }
  database.projects = database.projects.map((project) =>
    project.id === task.projectId ? { ...project, updatedAt } : project,
  );

  await writeDatabase(database);

  return nextTask;
}

export async function archiveTask(taskId: string) {
  return mutateArchivedTask(taskId, true);
}

export async function restoreTask(taskId: string) {
  return mutateArchivedTask(taskId, false);
}

async function mutateArchivedTask(taskId: string, archived: boolean) {
  const database = await readDatabase();
  const updatedAt = now();
  const taskIndex = database.tasks.findIndex((task) => task.id === taskId);

  if (taskIndex === -1) {
    return null;
  }

  const persistedTask: TaskDto = {
    ...database.tasks[taskIndex],
    archivedAt: archived ? updatedAt : null,
    updatedAt,
  };
  database.tasks[taskIndex] = persistedTask;

  database.projects = database.projects.map((project) =>
    project.id === persistedTask.projectId ? { ...project, updatedAt } : project,
  );
  await writeDatabase(database);

  return persistedTask;
}

export async function listComments(taskId: string) {
  const database = await readDatabase();

  return [...database.comments]
    .filter((comment) => comment.taskId === taskId)
    .sort(
      (left, right) =>
        new Date(left.createdAt).getTime() - new Date(right.createdAt).getTime(),
    );
}

export async function createComment(taskId: string, input: unknown) {
  const payload = commentSchema.parse(input);
  const database = await readDatabase();
  const task = database.tasks.find((item) => item.id === taskId);

  if (!task) {
    return null;
  }

  const comment = createCommentRecord(taskId, payload.authorName, payload.content);

  database.comments.push(comment);
  database.tasks = database.tasks.map((item) =>
    item.id === taskId ? { ...item, updatedAt: comment.createdAt } : item,
  );
  await writeDatabase(database);

  return comment;
}

export async function listTags(projectId: string) {
  const database = await readDatabase();

  return database.tags.filter((tag) => tag.projectId === projectId);
}

export function parseTaskFilters(searchParams: URLSearchParams): TaskListFilters {
  const priority = searchParams.get('priority');
  const archived = searchParams.get('archived');
  const sortBy = searchParams.get('sortBy');
  const sortOrder = searchParams.get('sortOrder');

  return {
    q: searchParams.get('q') ?? '',
    statusId: searchParams.get('statusId') ?? 'all',
    priority: isPriority(priority) ? priority : 'all',
    tagId: searchParams.get('tagId') ?? 'all',
    archived:
      archived === 'only' || archived === 'include' || archived === 'exclude'
        ? archived
        : 'exclude',
    sortBy:
      sortBy === 'updatedAt' ||
      sortBy === 'dueDate' ||
      sortBy === 'createdAt' ||
      sortBy === 'position'
        ? sortBy
        : 'position',
    sortOrder: sortOrder === 'desc' ? 'desc' : 'asc',
  };
}

function isPriority(value: string | null): value is Priority {
  return value === 'low' || value === 'medium' || value === 'high' || value === 'urgent';
}
