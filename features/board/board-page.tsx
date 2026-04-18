'use client';

import {
  closestCorners,
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from '@dnd-kit/core';
import {
  SortableContext,
  arrayMove,
  rectSortingStrategy,
  sortableKeyboardCoordinates,
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import Link from 'next/link';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { startTransition, useDeferredValue, useEffect, useState } from 'react';
import { z } from 'zod';

import { TaskDetailSheet } from '@/features/task/task-detail-sheet';
import { apiClient } from '@/lib/api/client';
import {
  applyTaskFilters,
  applyTaskReorder,
  computeBoardSummary,
  countOverdueTasks,
  sortStatuses,
  sortTasks,
} from '@/lib/board-utils';
import type {
  ApiSuccess,
  BoardSnapshotDto,
  CreateTaskInput,
  Priority,
  ReorderTasksInput,
  StatusDto,
  TaskDto,
} from '@/lib/types';
import { cn, formatDateLabel } from '@/lib/utils';
import { useBoardUiStore } from '@/stores/board-ui-store';

const createTaskSchema = z.object({
  title: z.string().trim().min(1, '请输入任务标题'),
  statusId: z.string().uuid(),
  priority: z.enum(['low', 'medium', 'high', 'urgent']),
  description: z.string().trim().max(3000, '描述不要超过 3000 字').optional(),
  startDate: z.string().optional(),
  dueDate: z.string().optional(),
});

type CreateTaskForm = z.infer<typeof createTaskSchema>;

const priorityTone: Record<Priority, string> = {
  low: 'bg-white text-[color:var(--ink-soft)]',
  medium: 'bg-[rgba(114,134,91,0.16)] text-[color:var(--olive)]',
  high: 'bg-[rgba(47,127,116,0.16)] text-[color:var(--teal-strong)]',
  urgent: 'bg-[color:var(--accent)] text-white',
};

export function BoardPage({
  projectId,
  taskId,
}: {
  projectId: string;
  taskId: string | null;
}) {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const queryClient = useQueryClient();
  const filters = useBoardUiStore((state) => state.filters);
  const isCreateTaskOpen = useBoardUiStore((state) => state.isCreateTaskOpen);
  const setSearch = useBoardUiStore((state) => state.setSearch);
  const setStatusId = useBoardUiStore((state) => state.setStatusId);
  const setPriority = useBoardUiStore((state) => state.setPriority);
  const setTagId = useBoardUiStore((state) => state.setTagId);
  const setArchived = useBoardUiStore((state) => state.setArchived);
  const clearFilters = useBoardUiStore((state) => state.clearFilters);
  const setCreateTaskOpen = useBoardUiStore((state) => state.setCreateTaskOpen);
  const resetForProject = useBoardUiStore((state) => state.resetForProject);
  const [activeTaskId, setActiveTaskId] = useState<string | null>(null);
  const [createTaskForm, setCreateTaskForm] = useState<CreateTaskForm>({
    title: '',
    statusId: '',
    priority: 'medium',
    description: '',
    startDate: '',
    dueDate: '',
  });
  const [createError, setCreateError] = useState<string | null>(null);

  useEffect(() => {
    resetForProject();
  }, [projectId, resetForProject]);

  const deferredSearch = useDeferredValue(filters.q);
  const boardQuery = useQuery({
    queryKey: ['board', projectId, filters.archived !== 'exclude'],
    queryFn: () => apiClient.getBoard(projectId, filters.archived !== 'exclude'),
  });

  const board = boardQuery.data?.data;
  const effectiveFilters = {
    ...filters,
    q: deferredSearch,
  };
  const filteredTasks = board ? applyTaskFilters(board, effectiveFilters) : [];
  const visibleStatuses = board ? sortStatuses(board.statuses) : [];
  const resolvedCreateTaskStatusId =
    createTaskForm.statusId || board?.statuses[0]?.id || '';
  const columnList = visibleStatuses.map((status) => ({
    status,
    tasks: sortTasks(filteredTasks.filter((task) => task.statusId === status.id)),
  }));

  const isFiltered =
    effectiveFilters.q.trim() !== '' ||
    effectiveFilters.priority !== 'all' ||
    effectiveFilters.statusId !== 'all' ||
    effectiveFilters.tagId !== 'all' ||
    effectiveFilters.archived !== 'exclude';
  const dragEnabled = !isFiltered;

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 6,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  );

  const reorderMutation = useMutation({
    mutationFn: (input: ReorderTasksInput) => apiClient.reorderTasks(projectId, input),
    onMutate: async (input) => {
      const key = ['board', projectId, false] as const;
      await queryClient.cancelQueries({ queryKey: ['board', projectId] });
      const previous =
        queryClient.getQueryData<ApiSuccess<BoardSnapshotDto>>(key) ?? null;

      if (previous) {
        queryClient.setQueryData<ApiSuccess<BoardSnapshotDto>>(key, {
          ...previous,
          data: applyTaskReorder(previous.data, input),
        });
      }

      return { previous };
    },
    onError: (_error, _variables, context) => {
      if (context?.previous) {
        queryClient.setQueryData(['board', projectId, false], context.previous);
      }
    },
    onSuccess: async ({ data }) => {
      queryClient.setQueryData<ApiSuccess<BoardSnapshotDto>>(
        ['board', projectId, false],
        {
          data,
          meta: {
            requestId: `req_${Date.now()}`,
          },
        },
      );
      await queryClient.invalidateQueries({ queryKey: ['board', projectId] });
    },
  });

  const createTaskMutation = useMutation({
    mutationFn: (input: CreateTaskInput) => apiClient.createTask(projectId, input),
    onSuccess: async () => {
      setCreateError(null);
      setCreateTaskOpen(false);
      setCreateTaskForm({
        title: '',
        statusId: resolvedCreateTaskStatusId,
        priority: 'medium',
        description: '',
        startDate: '',
        dueDate: '',
      });
      await queryClient.invalidateQueries({ queryKey: ['board', projectId] });
    },
  });

  function openTask(nextTaskId: string) {
    const params = new URLSearchParams(searchParams.toString());
    params.set('taskId', nextTaskId);

    startTransition(() => {
      router.replace(`${pathname}?${params.toString()}`, { scroll: false });
    });
  }

  function closeTask() {
    const params = new URLSearchParams(searchParams.toString());
    params.delete('taskId');
    const queryString = params.toString();

    startTransition(() => {
      router.replace(queryString ? `${pathname}?${queryString}` : pathname, {
        scroll: false,
      });
    });
  }

  function handleCreateTask() {
    const parsed = createTaskSchema.safeParse({
      ...createTaskForm,
      statusId: resolvedCreateTaskStatusId,
    });

    if (!parsed.success) {
      setCreateError(parsed.error.issues[0]?.message ?? '任务信息不完整');
      return;
    }

    setCreateError(null);
    createTaskMutation.mutate({
      title: parsed.data.title,
      statusId: parsed.data.statusId,
      priority: parsed.data.priority,
      description: parsed.data.description || null,
      startDate: parsed.data.startDate || null,
      dueDate: parsed.data.dueDate || null,
      tagIds: [],
    });
  }

  function handleDragStart(event: DragStartEvent) {
    setActiveTaskId(String(event.active.id));
  }

  function handleDragEnd(event: DragEndEvent) {
    setActiveTaskId(null);

    if (!board || !dragEnabled) {
      return;
    }

    const activeId = String(event.active.id);
    const overId = event.over ? String(event.over.id) : null;

    if (!overId || activeId === overId) {
      return;
    }

    const nextInput = buildReorderInput({
      activeId,
      board,
      overId,
    });

    if (!nextInput) {
      return;
    }

    reorderMutation.mutate(nextInput);
  }

  const summary = board
    ? computeBoardSummary({
        statuses: board.statuses,
        tasks: board.tasks,
      })
    : {
        activeTaskCount: 0,
        doneTaskCount: 0,
        archivedTaskCount: 0,
      };
  const activeDragTask =
    board?.tasks.find((task) => task.id === activeTaskId) ?? null;

  return (
    <main className="min-h-screen px-4 py-5 sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-[1600px] flex-col gap-5">
        <section className="rounded-[30px] border border-[color:var(--line-soft)] bg-[color:var(--panel)] px-5 py-5 shadow-[var(--shadow)] backdrop-blur sm:px-6 lg:px-7">
          <div className="flex flex-col gap-5 border-b border-[color:var(--line-soft)] pb-5 xl:flex-row xl:items-end xl:justify-between">
            <div className="max-w-2xl">
              <Link
                href="/"
                className="text-xs font-semibold uppercase tracking-[0.22em] text-[color:var(--ink-faint)]"
              >
                返回项目列表
              </Link>
              <h1 className="mt-3 text-3xl font-semibold tracking-[-0.06em] text-[color:var(--ink)] sm:text-4xl">
                {board?.project.name ?? '加载项目中...'}
              </h1>
              <p className="mt-3 max-w-xl text-sm leading-7 text-[color:var(--ink-soft)]">
                {board?.project.description ?? '正在读取项目描述。'}
              </p>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 xl:min-w-[34rem] xl:grid-cols-4">
              <SummaryCard label="进行中" value={String(summary.activeTaskCount)} />
              <SummaryCard label="已完成" value={String(summary.doneTaskCount)} />
              <SummaryCard
                label="逾期风险"
                value={String(countOverdueTasks(board?.tasks ?? []))}
              />
              <SummaryCard label="已归档" value={String(summary.archivedTaskCount)} />
            </div>
          </div>

          <div className="mt-5 flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-[minmax(18rem,24rem)_repeat(3,minmax(8rem,10rem))]">
              <label className="flex items-center rounded-full border border-[color:var(--line-soft)] bg-white/75 px-4 py-3">
                <input
                  value={filters.q}
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="搜索标题或描述"
                  className="w-full bg-transparent text-sm outline-none placeholder:text-[color:var(--ink-faint)]"
                />
              </label>

              <ToolbarSelect
                value={filters.statusId}
                onChange={setStatusId}
                options={[
                  { value: 'all', label: '全部状态' },
                  ...visibleStatuses.map((status) => ({
                    value: status.id,
                    label: status.name,
                  })),
                ]}
              />

              <ToolbarSelect
                value={filters.priority}
                onChange={(value) => setPriority(value as Priority | 'all')}
                options={[
                  { value: 'all', label: '全部优先级' },
                  { value: 'low', label: '低' },
                  { value: 'medium', label: '中' },
                  { value: 'high', label: '高' },
                  { value: 'urgent', label: '紧急' },
                ]}
              />

              <ToolbarSelect
                value={filters.tagId}
                onChange={setTagId}
                options={[
                  { value: 'all', label: '全部标签' },
                  ...(board?.tags ?? []).map((tag) => ({
                    value: tag.id,
                    label: tag.name,
                  })),
                ]}
              />
            </div>

            <div className="flex flex-wrap items-center gap-3">
              <ToolbarSelect
                value={filters.archived}
                onChange={(value) =>
                  setArchived(value as 'exclude' | 'include' | 'only')
                }
                options={[
                  { value: 'exclude', label: '排除归档' },
                  { value: 'include', label: '包含归档' },
                  { value: 'only', label: '仅归档' },
                ]}
              />
              <button
                type="button"
                onClick={clearFilters}
                className="rounded-full border border-[color:var(--line-soft)] px-4 py-2 text-sm font-medium text-[color:var(--ink-soft)]"
              >
                清空筛选
              </button>
              <button
                type="button"
                onClick={() => setCreateTaskOpen(true)}
                className="rounded-full bg-[color:var(--accent)] px-4 py-2 text-sm font-semibold text-white transition hover:bg-[color:var(--accent-strong)]"
              >
                新建任务
              </button>
            </div>
          </div>

          {isFiltered ? (
            <div className="mt-4 rounded-[20px] border border-[rgba(199,102,55,0.16)] bg-[rgba(199,102,55,0.08)] px-4 py-3 text-sm text-[color:var(--accent-strong)]">
              当前处于筛选结果视图，已关闭拖拽排序，避免在局部结果里误改列内顺序。
            </div>
          ) : null}
        </section>

        <section className="rounded-[30px] border border-[color:var(--line-soft)] bg-[color:var(--panel)] p-4 shadow-[var(--shadow)] backdrop-blur sm:p-5">
          {boardQuery.isLoading ? (
            <div className="grid gap-4 xl:grid-cols-3">
              {Array.from({ length: 3 }).map((_, index) => (
                <div
                  key={index}
                  className="h-[540px] animate-pulse rounded-[26px] border border-[color:var(--line-soft)] bg-white/45"
                />
              ))}
            </div>
          ) : null}

          {!boardQuery.isLoading && !board ? (
            <div className="flex min-h-[50vh] items-center justify-center rounded-[26px] border border-dashed border-[color:var(--line-soft)] px-6 text-sm text-[color:var(--danger)]">
              项目不存在，或者 mock 数据尚未初始化完成。
            </div>
          ) : null}

          {board ? (
            <DndContext
              sensors={sensors}
              collisionDetection={closestCorners}
              onDragStart={handleDragStart}
              onDragEnd={handleDragEnd}
            >
              <div className="flex gap-4 overflow-x-auto pb-2">
                {columnList.map(({ status, tasks }) => (
                  <TaskColumn
                    key={status.id}
                    dragEnabled={dragEnabled}
                    onSelectTask={openTask}
                    status={status}
                    tags={board.tags}
                    tasks={tasks}
                  />
                ))}
              </div>
              <DragOverlay>
                {activeDragTask ? (
                  <TaskCardInner dragging tags={board.tags} task={activeDragTask} />
                ) : null}
              </DragOverlay>
            </DndContext>
          ) : null}
        </section>
      </div>

      {board && taskId ? (
        <TaskDetailSheet
          board={board}
          onClose={closeTask}
          projectId={projectId}
          taskId={taskId}
        />
      ) : null}

      {isCreateTaskOpen && board ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[rgba(20,14,10,0.38)] px-4 py-8 backdrop-blur-sm">
          <div className="w-full max-w-xl rounded-[30px] border border-[color:var(--line-soft)] bg-[color:var(--paper)] p-6 shadow-[var(--shadow)]">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-sm font-medium text-[color:var(--ink-faint)]">
                  快速创建任务
                </p>
                <h2 className="mt-1 text-2xl font-semibold tracking-[-0.04em] text-[color:var(--ink)]">
                  先录入标题，再决定是否补充更多信息
                </h2>
              </div>
              <button
                type="button"
                onClick={() => setCreateTaskOpen(false)}
                className="rounded-full border border-[color:var(--line-soft)] px-3 py-1.5 text-sm text-[color:var(--ink-soft)]"
              >
                关闭
              </button>
            </div>

            <div className="mt-6 grid gap-4">
              <label className="grid gap-2">
                <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                  任务标题
                </span>
                <input
                  autoFocus
                  value={createTaskForm.title}
                  onChange={(event) =>
                    setCreateTaskForm((current) => ({
                      ...current,
                      title: event.target.value,
                    }))
                  }
                  placeholder="例如：完成任务详情面板交互"
                  className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                />
              </label>

              <div className="grid gap-4 sm:grid-cols-2">
                <ToolbarSelect
                  value={resolvedCreateTaskStatusId}
                  onChange={(value) =>
                    setCreateTaskForm((current) => ({
                      ...current,
                      statusId: value,
                    }))
                  }
                  options={board.statuses.map((status) => ({
                    value: status.id,
                    label: status.name,
                  }))}
                />
                <ToolbarSelect
                  value={createTaskForm.priority}
                  onChange={(value) =>
                    setCreateTaskForm((current) => ({
                      ...current,
                      priority: value as Priority,
                    }))
                  }
                  options={[
                    { value: 'low', label: '低优先级' },
                    { value: 'medium', label: '中优先级' },
                    { value: 'high', label: '高优先级' },
                    { value: 'urgent', label: '紧急' },
                  ]}
                />
              </div>

              <label className="grid gap-2">
                <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                  描述
                </span>
                <textarea
                  value={createTaskForm.description}
                  onChange={(event) =>
                    setCreateTaskForm((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }
                  rows={4}
                  placeholder="可选，后续也可以在详情面板补充。"
                  className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                />
              </label>

              <div className="grid gap-4 sm:grid-cols-2">
                <label className="grid gap-2">
                  <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                    开始日期
                  </span>
                  <input
                    type="date"
                    value={createTaskForm.startDate}
                    onChange={(event) =>
                      setCreateTaskForm((current) => ({
                        ...current,
                        startDate: event.target.value,
                      }))
                    }
                    className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                  />
                </label>

                <label className="grid gap-2">
                  <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                    截止日期
                  </span>
                  <input
                    type="date"
                    value={createTaskForm.dueDate}
                    onChange={(event) =>
                      setCreateTaskForm((current) => ({
                        ...current,
                        dueDate: event.target.value,
                      }))
                    }
                    className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                  />
                </label>
              </div>
            </div>

            {createError || createTaskMutation.error ? (
              <p className="mt-4 rounded-[18px] border border-[rgba(167,66,55,0.2)] bg-[rgba(167,66,55,0.08)] px-4 py-3 text-sm text-[color:var(--danger)]">
                {createError ?? createTaskMutation.error?.message}
              </p>
            ) : null}

            <div className="mt-6 flex items-center justify-between gap-4">
              <p className="text-sm leading-6 text-[color:var(--ink-soft)]">
                默认只要求标题，其他字段都可以稍后在右侧详情面板里继续编辑。
              </p>
              <button
                type="button"
                onClick={handleCreateTask}
                disabled={createTaskMutation.isPending}
                className="rounded-full bg-[color:var(--accent)] px-4 py-2 text-sm font-semibold text-white transition hover:bg-[color:var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-60"
              >
                {createTaskMutation.isPending ? '创建中...' : '创建任务'}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </main>
  );
}

function ToolbarSelect({
  onChange,
  options,
  value,
}: {
  onChange: (value: string) => void;
  options: Array<{ value: string; label: string }>;
  value: string;
}) {
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      className="rounded-full border border-[color:var(--line-soft)] bg-white/75 px-4 py-3 text-sm text-[color:var(--ink)] outline-none transition focus:border-[color:var(--accent)]"
    >
      {options.map((option) => (
        <option key={option.value} value={option.value}>
          {option.label}
        </option>
      ))}
    </select>
  );
}

function SummaryCard({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <article className="rounded-[22px] border border-[color:var(--line-soft)] bg-white/60 px-4 py-4">
      <p className="text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--ink-faint)]">
        {label}
      </p>
      <div className="mt-2 text-3xl font-semibold tracking-[-0.08em] text-[color:var(--ink)]">
        {value}
      </div>
    </article>
  );
}

function TaskColumn({
  dragEnabled,
  onSelectTask,
  status,
  tags,
  tasks,
}: {
  dragEnabled: boolean;
  onSelectTask: (taskId: string) => void;
  status: StatusDto;
  tags: BoardSnapshotDto['tags'];
  tasks: TaskDto[];
}) {
  const { isOver, setNodeRef } = useDroppable({
    id: status.id,
    data: {
      type: 'column',
      statusId: status.id,
    },
    disabled: !dragEnabled,
  });

  return (
    <section
      ref={setNodeRef}
      className={cn(
        'flex min-h-[68vh] w-[320px] min-w-[320px] flex-col rounded-[28px] border bg-[color:var(--panel-soft)] p-4 transition',
        isOver
          ? 'border-[color:var(--accent)] shadow-[0_16px_40px_rgba(50,34,18,0.14)]'
          : 'border-[color:var(--line-soft)]',
      )}
    >
      <div className="flex items-start justify-between gap-3 border-b border-[color:var(--line-soft)] pb-4">
        <div>
          <div className="inline-flex items-center gap-2 rounded-full border border-[color:var(--line-soft)] bg-white/80 px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-[color:var(--ink-faint)]">
            <span
              className="h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: status.color }}
            />
            {status.name}
          </div>
          <p className="mt-3 text-sm text-[color:var(--ink-soft)]">
            {status.isDone ? '只保留结果感，不堆叠操作噪音。' : '可拖拽流转，也可在详情面板调整。'}
          </p>
        </div>
        <strong className="rounded-full bg-white/85 px-3 py-2 text-sm text-[color:var(--ink)]">
          {tasks.length}
        </strong>
      </div>

      <SortableContext items={tasks.map((task) => task.id)} strategy={rectSortingStrategy}>
        <div className="mt-4 flex flex-1 flex-col gap-3">
          {tasks.map((task) => (
            <SortableTaskCard
              key={task.id}
              dragEnabled={dragEnabled}
              onClick={() => onSelectTask(task.id)}
              tags={tags}
              task={task}
            />
          ))}
        </div>
      </SortableContext>

      {!tasks.length ? (
        <div className="mt-4 rounded-[20px] border border-dashed border-[color:var(--line-soft)] px-4 py-8 text-center text-sm leading-6 text-[color:var(--ink-soft)]">
          这个状态下暂时没有任务。
        </div>
      ) : null}
    </section>
  );
}

function SortableTaskCard({
  dragEnabled,
  onClick,
  tags,
  task,
}: {
  dragEnabled: boolean;
  onClick: () => void;
  tags: BoardSnapshotDto['tags'];
  task: TaskDto;
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } =
    useSortable({
      id: task.id,
      data: {
        type: 'task',
        task,
      },
      disabled: !dragEnabled,
    });

  return (
    <div
      ref={setNodeRef}
      style={{
        transform: CSS.Transform.toString(transform),
        transition,
      }}
    >
      <TaskCardInner
        dragging={isDragging}
        handleProps={{
          ...attributes,
          ...listeners,
        }}
        onClick={onClick}
        tags={tags}
        task={task}
      />
    </div>
  );
}

function TaskCardInner({
  dragging = false,
  handleProps,
  onClick,
  tags,
  task,
}: {
  dragging?: boolean;
  handleProps?: Record<string, unknown>;
  onClick?: () => void;
  tags: BoardSnapshotDto['tags'];
  task: TaskDto;
}) {
  const taskTags = tags.filter((tag) => task.tagIds.includes(tag.id)).slice(0, 3);

  return (
    <article
      className={cn(
        'rounded-[22px] border border-[color:var(--line-soft)] bg-white/90 p-4 shadow-[0_14px_34px_rgba(50,34,18,0.08)] transition',
        dragging
          ? 'rotate-[1deg] shadow-[0_24px_60px_rgba(50,34,18,0.18)]'
          : 'hover:-translate-y-0.5',
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <button type="button" onClick={onClick} className="flex-1 text-left">
          <h3 className="text-base font-semibold leading-6 text-[color:var(--ink)]">
            {task.title}
          </h3>
          <p className="mt-2 text-sm leading-6 text-[color:var(--ink-soft)]">
            {task.description || '暂无描述，点击右侧面板补充。'}
          </p>
        </button>
        <button
          type="button"
          aria-label="drag task"
          className="rounded-full border border-[color:var(--line-soft)] px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-[color:var(--ink-faint)]"
          {...handleProps}
        >
          拖拽
        </button>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        {taskTags.map((tag) => (
          <span
            key={tag.id}
            className="rounded-full px-3 py-1 text-xs font-medium"
            style={{
              backgroundColor: `${tag.color}22`,
              color: tag.color,
            }}
          >
            {tag.name}
          </span>
        ))}
      </div>

      <div className="mt-4 flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.14em] text-[color:var(--ink-faint)]">
        <span className={cn('rounded-full px-3 py-1 font-semibold', priorityTone[task.priority])}>
          {task.priority}
        </span>
        <span className="rounded-full border border-[color:var(--line-soft)] px-3 py-1">
          截止 {formatDateLabel(task.dueDate)}
        </span>
      </div>
    </article>
  );
}

function buildReorderInput({
  activeId,
  board,
  overId,
}: {
  activeId: string;
  board: BoardSnapshotDto;
  overId: string;
}): ReorderTasksInput | null {
  const activeTask = board.tasks.find((task) => task.id === activeId);

  if (!activeTask) {
    return null;
  }

  const overTask = board.tasks.find((task) => task.id === overId);
  const destinationStatusId = overTask?.statusId ?? overId;
  const sourceStatusId = activeTask.statusId;

  if (destinationStatusId === sourceStatusId && !overTask) {
    return null;
  }

  if (destinationStatusId === sourceStatusId && overTask) {
    const statusTaskIds = sortTasks(
      board.tasks.filter((task) => task.statusId === sourceStatusId),
    ).map((task) => task.id);

    return {
      movedTaskId: activeId,
      sourceStatusId,
      destinationStatusId,
      orderedTaskIds: arrayMove(
        statusTaskIds,
        statusTaskIds.indexOf(activeId),
        statusTaskIds.indexOf(overTask.id),
      ),
    };
  }

  const destinationTaskIds = sortTasks(
    board.tasks.filter((task) => task.statusId === destinationStatusId),
  ).map((task) => task.id);
  const targetIndex = overTask
    ? destinationTaskIds.indexOf(overTask.id)
    : destinationTaskIds.length;
  const orderedTaskIds = [...destinationTaskIds];
  orderedTaskIds.splice(targetIndex, 0, activeId);

  return {
    movedTaskId: activeId,
    sourceStatusId,
    destinationStatusId,
    orderedTaskIds,
  };
}
