'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { z } from 'zod';

import { apiClient } from '@/lib/api/client';
import { formatDateTimeLabel } from '@/lib/utils';
import { useBoardUiStore } from '@/stores/board-ui-store';

const createProjectSchema = z.object({
  name: z.string().trim().min(1, '请输入项目名称'),
  slug: z.string().trim().min(1, '请输入 slug'),
  description: z.string().trim().max(240, '描述不要超过 240 字').optional(),
  color: z.string().trim().regex(/^#[0-9a-fA-F]{6}$/, '颜色格式应为 #RRGGBB'),
});

type ProjectFormState = z.infer<typeof createProjectSchema>;

const colorOptions = ['#c76637', '#2f7f74', '#72865b', '#5d6eb1', '#b26d7c'] as const;

export function ProjectListPage() {
  const router = useRouter();
  const queryClient = useQueryClient();
  const isCreateProjectOpen = useBoardUiStore((state) => state.isCreateProjectOpen);
  const setCreateProjectOpen = useBoardUiStore(
    (state) => state.setCreateProjectOpen,
  );
  const [form, setForm] = useState<ProjectFormState>({
    name: '',
    slug: '',
    description: '',
    color: colorOptions[0],
  });
  const [error, setError] = useState<string | null>(null);

  const projectsQuery = useQuery({
    queryKey: ['projects'],
    queryFn: () => apiClient.listProjects(),
  });

  const createProjectMutation = useMutation({
    mutationFn: apiClient.createProject,
    onSuccess: async ({ data }) => {
      await queryClient.invalidateQueries({ queryKey: ['projects'] });
      setCreateProjectOpen(false);
      setForm({
        name: '',
        slug: '',
        description: '',
        color: colorOptions[0],
      });
      router.push(`/projects/${data.id}`);
    },
  });

  function submitProject() {
    const parsed = createProjectSchema.safeParse(form);

    if (!parsed.success) {
      setError(parsed.error.issues[0]?.message ?? '表单校验失败');
      return;
    }

    setError(null);
    createProjectMutation.mutate({
      name: parsed.data.name,
      slug: parsed.data.slug,
      description: parsed.data.description || null,
      color: parsed.data.color,
    });
  }

  return (
    <main className="min-h-screen px-4 py-6 sm:px-6 lg:px-10">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-6">
        <section className="rounded-[32px] border border-[color:var(--line-soft)] bg-[color:var(--panel)] px-6 py-7 shadow-[var(--shadow)] backdrop-blur md:px-8">
          <div className="flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between">
            <div className="max-w-2xl">
              <div className="inline-flex rounded-full border border-[color:rgba(199,102,55,0.18)] bg-[rgba(199,102,55,0.12)] px-3 py-1 text-[11px] font-semibold tracking-[0.24em] uppercase text-[color:var(--accent-strong)]">
                Local Kanban Workspace
              </div>
              <h1 className="mt-4 text-4xl font-semibold tracking-[-0.06em] text-[color:var(--ink)] sm:text-5xl">
                用最短路径把任务放上看板，再在同一上下文里推进它。
              </h1>
              <p className="mt-4 max-w-xl text-sm leading-7 text-[color:var(--ink-soft)] sm:text-base">
                首版只保留项目列表、看板流转、右侧详情面板和轻量搜索筛选。
                不追求过度配置，优先保证任务创建、拖拽和编辑这条主链路顺畅。
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-3 lg:min-w-[26rem]">
              <MetricCard
                label="项目总数"
                value={String(projectsQuery.data?.data.length ?? 0).padStart(2, '0')}
                hint="支持创建多个独立项目，但首页保持轻量平铺。"
              />
              <MetricCard
                label="核心能力"
                value="05"
                hint="项目、看板、筛选、详情、评论。"
              />
              <MetricCard
                label="部署方式"
                value="Local"
                hint="本地可跑、结构清晰、便于后续接真实后端。"
              />
            </div>
          </div>
        </section>

        <section className="rounded-[30px] border border-[color:var(--line-soft)] bg-[color:var(--panel)] px-6 py-6 shadow-[var(--shadow)] backdrop-blur">
          <div className="flex flex-col gap-4 border-b border-[color:var(--line-soft)] pb-5 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <p className="text-sm font-medium text-[color:var(--ink-faint)]">
                项目列表
              </p>
              <h2 className="mt-1 text-2xl font-semibold tracking-[-0.04em]">
                当前可进入的工作区
              </h2>
            </div>
            <button
              type="button"
              onClick={() => setCreateProjectOpen(true)}
              className="inline-flex items-center justify-center rounded-full bg-[color:var(--accent)] px-5 py-3 text-sm font-semibold text-white transition hover:bg-[color:var(--accent-strong)]"
            >
              创建项目
            </button>
          </div>

          {projectsQuery.isLoading ? (
            <div className="grid gap-4 pt-5 md:grid-cols-2 xl:grid-cols-3">
              {Array.from({ length: 3 }).map((_, index) => (
                <div
                  key={index}
                  className="h-56 animate-pulse rounded-[28px] border border-[color:var(--line-soft)] bg-white/45"
                />
              ))}
            </div>
          ) : null}

          {projectsQuery.data?.data.length ? (
            <div className="grid gap-4 pt-5 md:grid-cols-2 xl:grid-cols-3">
              {projectsQuery.data.data.map((project) => (
                <Link
                  key={project.id}
                  href={`/projects/${project.id}`}
                  className="group rounded-[28px] border border-[color:var(--line-soft)] bg-[color:var(--panel-soft)] p-5 transition hover:-translate-y-0.5 hover:border-[color:var(--line-strong)] hover:shadow-[0_22px_50px_rgba(50,34,18,0.12)]"
                >
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <div
                        className="h-3 w-14 rounded-full"
                        style={{ backgroundColor: project.color }}
                      />
                      <h3 className="mt-4 text-2xl font-semibold tracking-[-0.04em] text-[color:var(--ink)]">
                        {project.name}
                      </h3>
                    </div>
                    <span className="rounded-full border border-[color:var(--line-soft)] px-3 py-1 font-mono text-[11px] uppercase tracking-[0.18em] text-[color:var(--ink-faint)]">
                      {project.slug}
                    </span>
                  </div>
                  <p className="mt-4 min-h-14 text-sm leading-7 text-[color:var(--ink-soft)]">
                    {project.description || '这个项目还没有描述，适合作为一个轻量工作区开始使用。'}
                  </p>
                  <dl className="mt-5 grid grid-cols-3 gap-3 rounded-[22px] bg-white/65 p-4 text-sm">
                    <div>
                      <dt className="text-[color:var(--ink-faint)]">总任务</dt>
                      <dd className="mt-2 text-xl font-semibold text-[color:var(--ink)]">
                        {project.summary.taskCount}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-[color:var(--ink-faint)]">已完成</dt>
                      <dd className="mt-2 text-xl font-semibold text-[color:var(--teal-strong)]">
                        {project.summary.completedTaskCount}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-[color:var(--ink-faint)]">逾期</dt>
                      <dd className="mt-2 text-xl font-semibold text-[color:var(--danger)]">
                        {project.summary.overdueTaskCount}
                      </dd>
                    </div>
                  </dl>
                  <p className="mt-4 text-xs uppercase tracking-[0.2em] text-[color:var(--ink-faint)]">
                    更新于 {formatDateTimeLabel(project.updatedAt)}
                  </p>
                </Link>
              ))}
            </div>
          ) : null}

          {!projectsQuery.isLoading && !projectsQuery.data?.data.length ? (
            <div className="flex flex-col items-center justify-center gap-4 px-4 py-16 text-center">
              <div className="rounded-full border border-[color:var(--line-soft)] bg-white/75 px-4 py-2 text-xs font-semibold uppercase tracking-[0.2em] text-[color:var(--ink-faint)]">
                First Run
              </div>
              <h3 className="text-3xl font-semibold tracking-[-0.05em] text-[color:var(--ink)]">
                还没有项目，先创建一个工作区。
              </h3>
              <p className="max-w-md text-sm leading-7 text-[color:var(--ink-soft)]">
                系统会自动生成「待处理 / 进行中 / 已完成」三列，你可以立即开始创建第一个任务。
              </p>
              <button
                type="button"
                onClick={() => setCreateProjectOpen(true)}
                className="inline-flex items-center justify-center rounded-full bg-[color:var(--accent)] px-5 py-3 text-sm font-semibold text-white transition hover:bg-[color:var(--accent-strong)]"
              >
                创建第一个项目
              </button>
            </div>
          ) : null}
        </section>
      </div>

      {isCreateProjectOpen ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[rgba(20,14,10,0.42)] px-4 py-8 backdrop-blur-sm">
          <div className="w-full max-w-xl rounded-[30px] border border-[color:var(--line-soft)] bg-[color:var(--paper)] p-6 shadow-[var(--shadow)]">
            <div className="flex items-start justify-between gap-4">
              <div>
                <p className="text-sm font-medium text-[color:var(--ink-faint)]">
                  创建项目
                </p>
                <h3 className="mt-1 text-2xl font-semibold tracking-[-0.04em]">
                  先定义一个明确的工作区
                </h3>
              </div>
              <button
                type="button"
                onClick={() => setCreateProjectOpen(false)}
                className="rounded-full border border-[color:var(--line-soft)] px-3 py-1.5 text-sm text-[color:var(--ink-soft)]"
              >
                关闭
              </button>
            </div>

            <div className="mt-6 grid gap-4">
              <label className="grid gap-2">
                <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                  项目名称
                </span>
                <input
                  value={form.name}
                  onChange={(event) =>
                    setForm((current) => ({ ...current, name: event.target.value }))
                  }
                  placeholder="例如：Plan"
                  className="rounded-[20px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                />
              </label>

              <div className="grid gap-4 md:grid-cols-[1.4fr_1fr]">
                <label className="grid gap-2">
                  <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                    Slug
                  </span>
                  <input
                    value={form.slug}
                    onChange={(event) =>
                      setForm((current) => ({ ...current, slug: event.target.value }))
                    }
                    placeholder="plan"
                    className="rounded-[20px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 font-mono outline-none transition focus:border-[color:var(--accent)]"
                  />
                </label>

                <div className="grid gap-2">
                  <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                    主色
                  </span>
                  <div className="flex flex-wrap gap-2">
                    {colorOptions.map((color) => (
                      <button
                        key={color}
                        type="button"
                        onClick={() => setForm((current) => ({ ...current, color }))}
                        className="h-11 w-11 rounded-full border-2 transition"
                        style={{
                          backgroundColor: color,
                          borderColor:
                            form.color === color ? 'rgba(33,23,15,0.7)' : 'rgba(0,0,0,0)',
                        }}
                      />
                    ))}
                  </div>
                </div>
              </div>

              <label className="grid gap-2">
                <span className="text-sm font-medium text-[color:var(--ink-soft)]">
                  简短描述
                </span>
                <textarea
                  value={form.description}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }
                  rows={4}
                  placeholder="说明这个项目解决什么问题。"
                  className="rounded-[20px] border border-[color:var(--line-soft)] bg-white/80 px-4 py-3 outline-none transition focus:border-[color:var(--accent)]"
                />
              </label>
            </div>

            {error || createProjectMutation.error ? (
              <p className="mt-4 rounded-[18px] border border-[rgba(167,66,55,0.2)] bg-[rgba(167,66,55,0.08)] px-4 py-3 text-sm text-[color:var(--danger)]">
                {error ?? createProjectMutation.error?.message}
              </p>
            ) : null}

            <div className="mt-6 flex items-center justify-between gap-4">
              <p className="text-sm leading-6 text-[color:var(--ink-soft)]">
                创建后会自动生成默认三列，并跳转到项目看板。
              </p>
              <button
                type="button"
                onClick={submitProject}
                disabled={createProjectMutation.isPending}
                className="inline-flex items-center justify-center rounded-full bg-[color:var(--accent)] px-5 py-3 text-sm font-semibold text-white transition hover:bg-[color:var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-60"
              >
                {createProjectMutation.isPending ? '创建中...' : '创建项目'}
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </main>
  );
}

function MetricCard({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint: string;
}) {
  return (
    <article className="rounded-[24px] border border-[color:var(--line-soft)] bg-white/62 p-4">
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-[color:var(--ink-faint)]">
        {label}
      </p>
      <div className="mt-3 text-4xl font-semibold tracking-[-0.08em] text-[color:var(--ink)]">
        {value}
      </div>
      <p className="mt-3 text-sm leading-6 text-[color:var(--ink-soft)]">{hint}</p>
    </article>
  );
}
