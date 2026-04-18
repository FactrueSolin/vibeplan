'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useState, useTransition } from 'react';
import { z } from 'zod';

import { apiClient } from '@/lib/api/client';
import type { BoardSnapshotDto, Priority, TagDto, TaskDto } from '@/lib/types';
import { formatDateTimeLabel } from '@/lib/utils';

const taskDetailSchema = z.object({
  title: z.string().trim().min(1, '标题不能为空'),
  description: z.string().trim().max(3000, '描述过长'),
  statusId: z.string().uuid(),
  priority: z.enum(['low', 'medium', 'high', 'urgent']),
  startDate: z.string().optional(),
  dueDate: z.string().optional(),
  tagIds: z.array(z.string().uuid()),
});

type TaskDetailForm = {
  title: string;
  description: string;
  statusId: string;
  priority: Priority;
  startDate: string;
  dueDate: string;
  tagIds: string[];
};

export function TaskDetailSheet({
  board,
  onClose,
  projectId,
  taskId,
}: {
  board: BoardSnapshotDto;
  onClose: () => void;
  projectId: string;
  taskId: string;
}) {
  const queryClient = useQueryClient();
  const [isClosing, startClosing] = useTransition();
  const [error, setError] = useState<string | null>(null);
  const [commentText, setCommentText] = useState('');

  const taskQuery = useQuery({
    queryKey: ['task', taskId],
    queryFn: () => apiClient.getTask(taskId),
    enabled: Boolean(taskId),
  });

  const commentsQuery = useQuery({
    queryKey: ['comments', taskId],
    queryFn: () => apiClient.listComments(taskId),
    enabled: Boolean(taskId),
  });

  const saveTaskMutation = useMutation({
    mutationFn: (nextForm: TaskDetailForm) =>
      apiClient.updateTask(taskId, {
        title: nextForm.title,
        description: nextForm.description || null,
        priority: nextForm.priority,
        statusId: nextForm.statusId,
        startDate: nextForm.startDate || null,
        dueDate: nextForm.dueDate || null,
        tagIds: nextForm.tagIds,
      }),
    onSuccess: async () => {
      setError(null);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['board', projectId] }),
        queryClient.invalidateQueries({ queryKey: ['task', taskId] }),
      ]);
    },
  });

  const archiveMutation = useMutation({
    mutationFn: () =>
      taskQuery.data?.data.archivedAt
        ? apiClient.restoreTask(taskId)
        : apiClient.archiveTask(taskId),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['board', projectId] }),
        queryClient.invalidateQueries({ queryKey: ['task', taskId] }),
      ]);
    },
  });

  const createCommentMutation = useMutation({
    mutationFn: (content: string) =>
      apiClient.createComment(taskId, {
        authorName: 'system',
        content,
      }),
    onSuccess: async () => {
      setCommentText('');
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: ['comments', taskId] }),
        queryClient.invalidateQueries({ queryKey: ['board', projectId] }),
      ]);
    },
  });

  const activeTask = taskQuery.data?.data;

  function closeSheet() {
    startClosing(() => {
      onClose();
    });
  }

  return (
    <aside className="fixed inset-x-0 bottom-0 top-0 z-40 flex justify-end bg-[rgba(24,18,12,0.22)] backdrop-blur-sm">
      <div className="ml-auto flex h-full w-full max-w-2xl flex-col border-l border-[color:var(--line-soft)] bg-[color:var(--paper)] shadow-[var(--shadow)]">
        <div className="flex items-start justify-between gap-4 border-b border-[color:var(--line-soft)] px-5 py-5 sm:px-6">
          <div>
            <p className="text-sm font-medium text-[color:var(--ink-faint)]">
              任务详情
            </p>
            <h2 className="mt-1 text-2xl font-semibold tracking-[-0.04em] text-[color:var(--ink)]">
              {activeTask?.title ?? '加载任务中...'}
            </h2>
          </div>
          <button
            type="button"
            onClick={closeSheet}
            disabled={isClosing}
            className="rounded-full border border-[color:var(--line-soft)] px-3 py-1.5 text-sm text-[color:var(--ink-soft)]"
          >
            关闭
          </button>
        </div>

        {taskQuery.isLoading ? (
          <div className="flex flex-1 items-center justify-center px-6 text-sm text-[color:var(--ink-soft)]">
            正在加载任务详情...
          </div>
        ) : null}

        {!taskQuery.isLoading && !activeTask ? (
          <div className="flex flex-1 items-center justify-center px-6 text-sm text-[color:var(--danger)]">
            任务不存在或已经被删除。
          </div>
        ) : null}

        {activeTask ? (
          <div className="grid flex-1 overflow-hidden lg:grid-cols-[1.1fr_0.9fr]">
            <TaskEditor
              archivePending={archiveMutation.isPending}
              board={board}
              error={error ?? saveTaskMutation.error?.message ?? null}
              key={`${activeTask.id}:${activeTask.updatedAt}`}
              onArchive={() => archiveMutation.mutate()}
              onClose={closeSheet}
              onError={setError}
              onSave={(nextForm) => saveTaskMutation.mutateAsync(nextForm)}
              savePending={saveTaskMutation.isPending}
              task={activeTask}
            />

            <div className="border-t border-[color:var(--line-soft)] bg-white/55 px-5 py-5 lg:border-l lg:border-t-0 sm:px-6">
              <div className="flex h-full flex-col">
                <div>
                  <p className="text-sm font-medium text-[color:var(--ink-faint)]">
                    评论区
                  </p>
                  <h3 className="mt-1 text-xl font-semibold tracking-[-0.04em] text-[color:var(--ink)]">
                    在任务上下文里补充说明
                  </h3>
                </div>

                <div className="mt-5 flex-1 overflow-y-auto rounded-[24px] border border-[color:var(--line-soft)] bg-[color:var(--panel-soft)] p-4">
                  <div className="grid gap-3">
                    {commentsQuery.data?.data.map((comment) => (
                      <article
                        key={comment.id}
                        className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/75 p-4"
                      >
                        <div className="flex items-center justify-between gap-3">
                          <strong className="text-sm text-[color:var(--ink)]">
                            {comment.authorName}
                          </strong>
                          <span className="text-xs uppercase tracking-[0.18em] text-[color:var(--ink-faint)]">
                            {formatDateTimeLabel(comment.createdAt)}
                          </span>
                        </div>
                        <p className="mt-3 text-sm leading-6 text-[color:var(--ink-soft)]">
                          {comment.content}
                        </p>
                      </article>
                    ))}

                    {!commentsQuery.data?.data.length ? (
                      <p className="rounded-[18px] border border-dashed border-[color:var(--line-soft)] px-4 py-6 text-sm leading-6 text-[color:var(--ink-soft)]">
                        还没有评论。可以在这里补充背景、依赖或交接说明。
                      </p>
                    ) : null}
                  </div>
                </div>

                <div className="mt-4 grid gap-3">
                  <textarea
                    value={commentText}
                    onChange={(event) => setCommentText(event.target.value)}
                    rows={4}
                    placeholder="输入评论，补充当前任务的进展或风险。"
                    className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                  />
                  <button
                    type="button"
                    onClick={() => {
                      const content = commentText.trim();

                      if (!content) {
                        return;
                      }

                      createCommentMutation.mutate(content);
                    }}
                    disabled={createCommentMutation.isPending}
                    className="inline-flex items-center justify-center rounded-full bg-[color:var(--teal)] px-4 py-2 text-sm font-semibold text-white transition hover:bg-[color:var(--teal-strong)] disabled:cursor-not-allowed disabled:opacity-60"
                  >
                    {createCommentMutation.isPending ? '提交中...' : '添加评论'}
                  </button>
                </div>
              </div>
            </div>
          </div>
        ) : null}
      </div>
    </aside>
  );
}

function TaskEditor({
  archivePending,
  board,
  error,
  onArchive,
  onClose,
  onError,
  onSave,
  savePending,
  task,
}: {
  archivePending: boolean;
  board: BoardSnapshotDto;
  error: string | null;
  onArchive: () => void;
  onClose: () => void;
  onError: (value: string | null) => void;
  onSave: (value: TaskDetailForm) => Promise<unknown>;
  savePending: boolean;
  task: TaskDto;
}) {
  const [form, setForm] = useState<TaskDetailForm>(() => mapTaskToForm(task));
  const baseline = mapTaskToForm(task);
  const isDirty = JSON.stringify(form) !== JSON.stringify(baseline);
  const selectedTags = board.tags.filter((tag) => form.tagIds.includes(tag.id));

  async function saveChanges() {
    const parsed = taskDetailSchema.safeParse(form);

    if (!parsed.success) {
      onError(parsed.error.issues[0]?.message ?? '任务校验失败');
      return;
    }

    onError(null);
    await onSave({
      ...parsed.data,
      startDate: parsed.data.startDate ?? '',
      dueDate: parsed.data.dueDate ?? '',
    });
  }

  function closeEditor() {
    if (isDirty && !window.confirm('当前任务还有未保存更改，确认关闭吗？')) {
      return;
    }

    onClose();
  }

  function toggleTag(tag: TagDto) {
    setForm((current) => ({
      ...current,
      tagIds: current.tagIds.includes(tag.id)
        ? current.tagIds.filter((item) => item !== tag.id)
        : [...current.tagIds, tag.id],
    }));
  }

  return (
    <div className="overflow-y-auto px-5 py-5 sm:px-6">
      <div className="grid gap-4">
        <label className="grid gap-2">
          <span className="text-sm font-medium text-[color:var(--ink-soft)]">
            标题
          </span>
          <input
            value={form.title}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                title: event.target.value,
              }))
            }
            className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
          />
        </label>

        <label className="grid gap-2">
          <span className="text-sm font-medium text-[color:var(--ink-soft)]">
            描述
          </span>
          <textarea
            value={form.description}
            onChange={(event) =>
              setForm((current) => ({
                ...current,
                description: event.target.value,
              }))
            }
            rows={7}
            className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
          />
        </label>

        <div className="grid gap-4 sm:grid-cols-2">
          <SelectField
            label="状态"
            value={form.statusId}
            onChange={(value) =>
              setForm((current) => ({ ...current, statusId: value }))
            }
            options={board.statuses.map((status) => ({
              value: status.id,
              label: status.name,
            }))}
          />
          <SelectField
            label="优先级"
            value={form.priority}
            onChange={(value) =>
              setForm((current) => ({
                ...current,
                priority: value as Priority,
              }))
            }
            options={[
              { value: 'low', label: '低' },
              { value: 'medium', label: '中' },
              { value: 'high', label: '高' },
              { value: 'urgent', label: '紧急' },
            ]}
          />
        </div>

        <div className="grid gap-4 sm:grid-cols-2">
          <DateField
            label="开始日期"
            value={form.startDate}
            onChange={(value) =>
              setForm((current) => ({ ...current, startDate: value }))
            }
          />
          <DateField
            label="截止日期"
            value={form.dueDate}
            onChange={(value) =>
              setForm((current) => ({ ...current, dueDate: value }))
            }
          />
        </div>

        <div className="grid gap-2">
          <span className="text-sm font-medium text-[color:var(--ink-soft)]">
            标签
          </span>
          <div className="flex flex-wrap gap-2">
            {board.tags.map((tag) => {
              const active = form.tagIds.includes(tag.id);

              return (
                <button
                  key={tag.id}
                  type="button"
                  onClick={() => toggleTag(tag)}
                  className="rounded-full border px-3 py-2 text-sm transition"
                  style={{
                    borderColor: active ? tag.color : 'var(--line-soft)',
                    backgroundColor: active ? `${tag.color}22` : 'rgba(255,255,255,0.7)',
                    color: active ? 'var(--ink)' : 'var(--ink-soft)',
                  }}
                >
                  {tag.name}
                </button>
              );
            })}
          </div>
          {selectedTags.length ? (
            <p className="text-xs uppercase tracking-[0.18em] text-[color:var(--ink-faint)]">
              已选择 {selectedTags.map((tag) => tag.name).join(' / ')}
            </p>
          ) : null}
        </div>

        {error ? (
          <p className="rounded-[18px] border border-[rgba(167,66,55,0.2)] bg-[rgba(167,66,55,0.08)] px-4 py-3 text-sm text-[color:var(--danger)]">
            {error}
          </p>
        ) : null}

        <div className="flex flex-wrap items-center justify-between gap-3 rounded-[24px] border border-[color:var(--line-soft)] bg-white/70 px-4 py-4">
          <div>
            <p className="text-sm font-medium text-[color:var(--ink)]">
              {isDirty ? '有未保存更改' : '当前内容已同步'}
            </p>
            <p className="mt-1 text-xs uppercase tracking-[0.18em] text-[color:var(--ink-faint)]">
              更新于 {formatDateTimeLabel(task.updatedAt)}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={onArchive}
              disabled={archivePending}
              className="rounded-full border border-[color:var(--line-soft)] px-4 py-2 text-sm font-medium text-[color:var(--ink-soft)] disabled:cursor-not-allowed disabled:opacity-60"
            >
              {task.archivedAt ? '恢复任务' : '归档任务'}
            </button>
            <button
              type="button"
              onClick={saveChanges}
              disabled={savePending}
              className="rounded-full bg-[color:var(--accent)] px-4 py-2 text-sm font-semibold text-white transition hover:bg-[color:var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-60"
            >
              {savePending ? '保存中...' : '保存更改'}
            </button>
            <button
              type="button"
              onClick={closeEditor}
              className="rounded-full border border-[color:var(--line-soft)] px-4 py-2 text-sm font-medium text-[color:var(--ink-soft)]"
            >
              取消
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function mapTaskToForm(task: TaskDto): TaskDetailForm {
  return {
    title: task.title,
    description: task.description ?? '',
    statusId: task.statusId,
    priority: task.priority,
    startDate: task.startDate ?? '',
    dueDate: task.dueDate ?? '',
    tagIds: task.tagIds,
  };
}

function SelectField({
  label,
  onChange,
  options,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  options: Array<{
    value: string;
    label: string;
  }>;
  value: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="text-sm font-medium text-[color:var(--ink-soft)]">{label}</span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
      >
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function DateField({
  label,
  onChange,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className="grid gap-2">
      <span className="text-sm font-medium text-[color:var(--ink-soft)]">{label}</span>
      <input
        type="date"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="rounded-[18px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
      />
    </label>
  );
}
