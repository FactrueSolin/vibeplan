export type Id = string;

export type Priority = 'low' | 'medium' | 'high' | 'urgent';
export type ArchivedFilter = 'exclude' | 'include' | 'only';
export type SortBy = 'updatedAt' | 'dueDate' | 'createdAt' | 'position';
export type SortOrder = 'asc' | 'desc';

export type ApiMeta = {
  requestId: string;
  page?: number;
  pageSize?: number;
  total?: number;
};

export type ApiSuccess<T> = {
  data: T;
  meta: ApiMeta;
};

export type ApiErrorPayload = {
  error: {
    code:
      | 'validation_error'
      | 'not_found'
      | 'conflict'
      | 'invalid_operation'
      | 'internal_error';
    message: string;
    details?: Record<string, string | number | boolean | null>;
    requestId: string;
  };
};

export type ProjectDto = {
  id: Id;
  name: string;
  slug: string;
  description: string | null;
  color: string;
  createdAt: string;
  updatedAt: string;
};

export type ProjectListItemDto = ProjectDto & {
  summary: {
    taskCount: number;
    completedTaskCount: number;
    overdueTaskCount: number;
  };
};

export type StatusDto = {
  id: Id;
  projectId: Id;
  name: string;
  color: string;
  sortOrder: number;
  isDone: boolean;
  isHidden: boolean;
  createdAt: string;
  updatedAt: string;
};

export type TaskDto = {
  id: Id;
  projectId: Id;
  statusId: Id;
  title: string;
  description: string | null;
  priority: Priority;
  position: number;
  startDate: string | null;
  dueDate: string | null;
  completedAt: string | null;
  archivedAt: string | null;
  tagIds: Id[];
  createdAt: string;
  updatedAt: string;
};

export type CommentDto = {
  id: Id;
  taskId: Id;
  authorName: string;
  content: string;
  createdAt: string;
  updatedAt: string;
};

export type TagDto = {
  id: Id;
  projectId: Id;
  name: string;
  color: string;
  createdAt: string;
  updatedAt: string;
};

export type TaskTagDto = {
  projectId: Id;
  taskId: Id;
  tagId: Id;
};

export type BoardSnapshotDto = {
  project: ProjectDto;
  statuses: StatusDto[];
  tasks: TaskDto[];
  tags: TagDto[];
  taskTags: TaskTagDto[];
  summary: {
    activeTaskCount: number;
    doneTaskCount: number;
    archivedTaskCount: number;
  };
};

export type TaskListFilters = {
  q: string;
  statusId: Id | 'all';
  priority: Priority | 'all';
  tagId: Id | 'all';
  archived: ArchivedFilter;
  sortBy: SortBy;
  sortOrder: SortOrder;
};

export type CreateProjectInput = {
  name: string;
  slug: string;
  description?: string | null;
  color?: string;
};

export type CreateTaskInput = {
  statusId: Id;
  title: string;
  description?: string | null;
  priority?: Priority;
  startDate?: string | null;
  dueDate?: string | null;
  tagIds?: Id[];
};

export type UpdateTaskInput = Partial<{
  statusId: Id;
  title: string;
  description: string | null;
  priority: Priority;
  startDate: string | null;
  dueDate: string | null;
  tagIds: Id[];
}>;

export type ReorderTasksInput = {
  movedTaskId: Id;
  sourceStatusId: Id;
  destinationStatusId: Id;
  orderedTaskIds: Id[];
};

export type CreateCommentInput = {
  authorName: string;
  content: string;
};
