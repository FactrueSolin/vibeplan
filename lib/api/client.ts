'use client';

import type {
  ApiErrorPayload,
  ApiSuccess,
  BoardSnapshotDto,
  CommentDto,
  CreateCommentInput,
  CreateProjectInput,
  CreateTaskInput,
  ProjectListItemDto,
  ReorderTasksInput,
  TagDto,
  TaskDto,
  UpdateTaskInput,
} from '@/lib/types';

export class ApiClientError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly requestId?: string,
  ) {
    super(message);
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<ApiSuccess<T>> {
  const response = await fetch(path, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
  });

  if (!response.ok) {
    const payload = (await response.json()) as ApiErrorPayload;

    throw new ApiClientError(
      payload.error.message,
      payload.error.code,
      payload.error.requestId,
    );
  }

  return (await response.json()) as ApiSuccess<T>;
}

export const apiClient = {
  listProjects() {
    return request<ProjectListItemDto[]>('/api/v1/projects');
  },
  createProject(input: CreateProjectInput) {
    return request<ProjectListItemDto>('/api/v1/projects', {
      method: 'POST',
      body: JSON.stringify(input),
    });
  },
  getBoard(projectId: string, includeArchived: boolean) {
    const params = new URLSearchParams({
      includeArchived: String(includeArchived),
    });

    return request<BoardSnapshotDto>(`/api/v1/projects/${projectId}/board?${params}`);
  },
  createTask(projectId: string, input: CreateTaskInput) {
    return request<TaskDto>(`/api/v1/projects/${projectId}/tasks`, {
      method: 'POST',
      body: JSON.stringify(input),
    });
  },
  reorderTasks(projectId: string, input: ReorderTasksInput) {
    return request<BoardSnapshotDto>(`/api/v1/projects/${projectId}/tasks/reorder`, {
      method: 'POST',
      body: JSON.stringify(input),
    });
  },
  getTask(taskId: string) {
    return request<TaskDto>(`/api/v1/tasks/${taskId}`);
  },
  updateTask(taskId: string, input: UpdateTaskInput) {
    return request<TaskDto>(`/api/v1/tasks/${taskId}`, {
      method: 'PATCH',
      body: JSON.stringify(input),
    });
  },
  archiveTask(taskId: string) {
    return request<TaskDto>(`/api/v1/tasks/${taskId}/archive`, {
      method: 'POST',
    });
  },
  restoreTask(taskId: string) {
    return request<TaskDto>(`/api/v1/tasks/${taskId}/restore`, {
      method: 'POST',
    });
  },
  listComments(taskId: string) {
    return request<CommentDto[]>(`/api/v1/tasks/${taskId}/comments`);
  },
  createComment(taskId: string, input: CreateCommentInput) {
    return request<CommentDto>(`/api/v1/tasks/${taskId}/comments`, {
      method: 'POST',
      body: JSON.stringify(input),
    });
  },
  listTags(projectId: string) {
    return request<TagDto[]>(`/api/v1/projects/${projectId}/tags`);
  },
};
