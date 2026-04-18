type Metric = {
  label: string;
  value: string;
  hint: string;
  accent: 'ember' | 'teal' | 'ink';
};

type Filter = {
  label: string;
  active?: boolean;
};

type TaskCard = {
  title: string;
  summary: string;
  priority: 'P0' | 'P1' | 'P2';
  dueDate: string;
  owner: string;
  tags: readonly string[];
};

type BoardColumn = {
  name: string;
  count: number;
  note: string;
  accent: 'sand' | 'teal' | 'olive';
  tasks: readonly TaskCard[];
};

const metrics: readonly Metric[] = [
  {
    label: '本周完成率',
    value: '68%',
    hint: '11 / 16 个任务已推进到完成态',
    accent: 'ember',
  },
  {
    label: '进行中',
    value: '07',
    hint: '2 个任务接近截止，需要优先跟进',
    accent: 'teal',
  },
  {
    label: '平均响应',
    value: '1.8h',
    hint: '评论和状态更新保持在同一上下文',
    accent: 'ink',
  },
];

const filters: readonly Filter[] = [
  { label: '全部任务', active: true },
  { label: '高优先级' },
  { label: '本周截止' },
  { label: '有评论' },
  { label: '仅我负责' },
];

const columns: readonly BoardColumn[] = [
  {
    name: '待处理',
    count: 6,
    note: '聚合待决策事项，优先展示任务价值与截止时间。',
    accent: 'sand',
    tasks: [
      {
        title: '重构任务创建弹层',
        summary: '把“仅标题必填”作为默认路径，减少首次录入成本。',
        priority: 'P1',
        dueDate: '4月20日',
        owner: 'YL',
        tags: ['交互', '录入', 'V1'],
      },
      {
        title: '项目空状态插画方向',
        summary: '弱化教程感，改成引导式信息块与首个动作按钮。',
        priority: 'P2',
        dueDate: '4月22日',
        owner: 'TC',
        tags: ['视觉', '空状态'],
      },
    ],
  },
  {
    name: '进行中',
    count: 4,
    note: '强化执行态感知，让拖拽和详情编辑自然衔接。',
    accent: 'teal',
    tasks: [
      {
        title: '任务详情侧栏高保真设计',
        summary: '右侧面板同时承载属性编辑、评论和活动记录。',
        priority: 'P0',
        dueDate: '今天',
        owner: 'TC',
        tags: ['详情', '核心链路', '面板'],
      },
      {
        title: '看板列头信息密度校准',
        summary: '列名、数量、说明与操作按钮控制在首屏一眼可扫。',
        priority: 'P1',
        dueDate: '4月19日',
        owner: 'SQ',
        tags: ['信息架构', '看板'],
      },
    ],
  },
  {
    name: '已完成',
    count: 9,
    note: '完成列不堆叠噪音，只保留结果感与节奏反馈。',
    accent: 'olive',
    tasks: [
      {
        title: '默认状态列定义',
        summary: '首版统一为待处理、进行中、已完成，降低首次使用门槛。',
        priority: 'P2',
        dueDate: '已完成',
        owner: 'MN',
        tags: ['架构', 'V1'],
      },
      {
        title: '搜索与筛选入口收敛',
        summary: '顶部工具栏合并搜索、筛选与新建任务，减少来回跳转。',
        priority: 'P1',
        dueDate: '已完成',
        owner: 'YL',
        tags: ['搜索', '工具栏'],
      },
    ],
  },
];

const detailChecklist = [
  { label: '标题与摘要支持快速提交', done: true },
  { label: '面板保持看板上下文可见', done: true },
  { label: '评论区支持轻量补充说明', done: false },
] as const;

const activityItems = [
  {
    time: '18:20',
    author: 'TC',
    text: '把优先级标签移到标题区右上角，减少卡片底部拥挤感。',
  },
  {
    time: '16:40',
    author: 'YL',
    text: '确认详情面板采用固定宽度，在移动端改为纵向堆叠。',
  },
  {
    time: '14:05',
    author: 'SQ',
    text: '补充筛选态说明，避免用户误判列表为空。',
  },
] as const;

const accentStyles = {
  ember: 'bg-[color:var(--accent-ember)]/12 text-[color:var(--accent-ember-strong)]',
  teal: 'bg-[color:var(--accent-teal)]/14 text-[color:var(--accent-teal-strong)]',
  ink: 'bg-[color:var(--ink)]/8 text-[color:var(--ink)]',
} as const;

const columnStyles = {
  sand: {
    badge:
      'border-[color:var(--line-soft)] bg-[color:var(--panel-strong)] text-[color:var(--ink)]',
    column:
      'border-[color:var(--line-soft)] bg-[linear-gradient(180deg,rgba(255,250,242,0.96),rgba(255,246,235,0.9))]',
  },
  teal: {
    badge:
      'border-[color:rgba(45,126,113,0.18)] bg-[color:rgba(226,244,240,0.85)] text-[color:var(--accent-teal-strong)]',
    column:
      'border-[color:rgba(45,126,113,0.18)] bg-[linear-gradient(180deg,rgba(240,251,247,0.96),rgba(228,244,239,0.92))]',
  },
  olive: {
    badge:
      'border-[color:rgba(109,128,92,0.16)] bg-[color:rgba(238,242,232,0.95)] text-[color:var(--accent-olive-strong)]',
    column:
      'border-[color:rgba(109,128,92,0.16)] bg-[linear-gradient(180deg,rgba(248,250,244,0.98),rgba(237,241,230,0.9))]',
  },
} as const;

const priorityStyles = {
  P0: 'bg-[color:var(--accent-ember)] text-white',
  P1: 'bg-[color:var(--ink)] text-white',
  P2: 'bg-[color:var(--panel-strong)] text-[color:var(--ink-soft)]',
} as const;

function MetricCard({ accent, hint, label, value }: Metric) {
  return (
    <article className="rounded-[24px] border border-[color:var(--line-soft)] bg-[color:var(--panel)]/90 p-4 shadow-[0_14px_40px_rgba(60,42,23,0.08)]">
      <div
        className={`inline-flex rounded-full px-3 py-1 text-[11px] font-semibold tracking-[0.24em] uppercase ${accentStyles[accent]}`}
      >
        {label}
      </div>
      <div className="mt-4 text-4xl font-semibold tracking-[-0.06em] text-[color:var(--ink)]">
        {value}
      </div>
      <p className="mt-3 max-w-[20rem] text-sm leading-6 text-[color:var(--ink-soft)]">
        {hint}
      </p>
    </article>
  );
}

function PriorityBadge({ priority }: { priority: TaskCard['priority'] }) {
  return (
    <span
      className={`inline-flex h-8 min-w-8 items-center justify-center rounded-full px-2.5 text-[11px] font-semibold tracking-[0.18em] uppercase ${priorityStyles[priority]}`}
    >
      {priority}
    </span>
  );
}

function TaskCardView({
  dueDate,
  owner,
  priority,
  summary,
  tags,
  title,
}: TaskCard) {
  return (
    <article className="rounded-[22px] border border-white/70 bg-white/90 p-4 shadow-[0_18px_40px_rgba(42,28,18,0.08)] backdrop-blur-sm transition-transform duration-200 hover:-translate-y-0.5">
      <div className="flex items-start justify-between gap-3">
        <div className="space-y-2">
          <h3 className="text-base font-semibold leading-6 text-[color:var(--ink)]">
            {title}
          </h3>
          <p className="text-sm leading-6 text-[color:var(--ink-soft)]">
            {summary}
          </p>
        </div>
        <PriorityBadge priority={priority} />
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        {tags.map((tag) => (
          <span
            key={tag}
            className="rounded-full bg-[color:var(--paper)] px-3 py-1 text-xs font-medium text-[color:var(--ink-soft)]"
          >
            {tag}
          </span>
        ))}
      </div>

      <div className="mt-5 flex items-center justify-between gap-3 border-t border-[color:var(--line-soft)] pt-4 text-sm text-[color:var(--ink-soft)]">
        <span>{dueDate}</span>
        <span className="inline-flex h-9 w-9 items-center justify-center rounded-full bg-[color:var(--accent-teal)]/15 font-mono text-xs font-semibold text-[color:var(--accent-teal-strong)]">
          {owner}
        </span>
      </div>
    </article>
  );
}

function BoardColumnView({ accent, count, name, note, tasks }: BoardColumn) {
  return (
    <section
      className={`w-[320px] shrink-0 rounded-[28px] border p-4 shadow-[0_20px_50px_rgba(61,42,22,0.08)] ${columnStyles[accent].column}`}
    >
      <div className="flex items-start justify-between gap-4">
        <div>
          <div className="flex items-center gap-3">
            <h2 className="text-lg font-semibold tracking-[-0.04em] text-[color:var(--ink)]">
              {name}
            </h2>
            <span
              className={`rounded-full border px-2.5 py-1 text-xs font-semibold ${columnStyles[accent].badge}`}
            >
              {count}
            </span>
          </div>
          <p className="mt-2 text-sm leading-6 text-[color:var(--ink-soft)]">
            {note}
          </p>
        </div>
        <button
          type="button"
          className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-white/70 bg-white/70 text-lg text-[color:var(--ink-soft)]"
          aria-label={`新建${name}任务`}
        >
          +
        </button>
      </div>

      <div className="mt-5 space-y-4">
        {tasks.map((task) => (
          <TaskCardView key={task.title} {...task} />
        ))}
      </div>
    </section>
  );
}

function MobilePreview() {
  return (
    <div className="mx-auto w-full max-w-[280px] rounded-[32px] border border-white/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.94),rgba(247,240,231,0.94))] p-3 shadow-[0_24px_70px_rgba(45,31,19,0.12)]">
      <div className="rounded-[24px] bg-[color:var(--ink)] px-4 py-3 text-white">
        <div className="text-[11px] uppercase tracking-[0.32em] text-white/60">
          移动端预览
        </div>
        <div className="mt-2 flex items-center justify-between">
          <div>
            <div className="text-sm text-white/60">今日进度</div>
            <div className="text-2xl font-semibold tracking-[-0.06em]">4 / 7</div>
          </div>
          <div className="rounded-full bg-white/14 px-3 py-1 text-xs">
            右滑切列
          </div>
        </div>
      </div>

      <div className="mt-3 space-y-3">
        <div className="rounded-[22px] bg-white/90 p-4 shadow-[0_12px_30px_rgba(45,31,19,0.08)]">
          <div className="flex items-start justify-between gap-3">
            <div>
              <div className="text-sm font-semibold text-[color:var(--ink)]">
                任务详情抽屉
              </div>
              <div className="mt-1 text-xs leading-5 text-[color:var(--ink-soft)]">
                在窄屏上从底部弹出，保持卡片浏览的连续性。
              </div>
            </div>
            <span className="rounded-full bg-[color:var(--accent-ember)] px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-white">
              P0
            </span>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div className="rounded-[20px] bg-[color:var(--panel-strong)] px-4 py-3">
            <div className="text-[11px] uppercase tracking-[0.22em] text-[color:var(--ink-soft)]">
              搜索
            </div>
            <div className="mt-2 text-sm font-medium text-[color:var(--ink)]">
              标题 / 标签
            </div>
          </div>
          <div className="rounded-[20px] bg-[color:rgba(226,244,240,0.78)] px-4 py-3">
            <div className="text-[11px] uppercase tracking-[0.22em] text-[color:var(--accent-teal-strong)]">
              筛选
            </div>
            <div className="mt-2 text-sm font-medium text-[color:var(--ink)]">
              高优先级
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default function Home() {
  return (
    <main className="min-h-screen px-4 py-5 sm:px-6 lg:px-8 lg:py-8">
      <div className="mx-auto max-w-[1680px]">
        <div className="overflow-hidden rounded-[36px] border border-white/60 bg-[color:var(--panel)]/82 shadow-[0_35px_120px_rgba(52,35,20,0.16)] backdrop-blur-xl">
          <header className="border-b border-[color:var(--line-soft)] px-5 py-5 sm:px-8">
            <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
              <div className="space-y-4">
                <div className="flex flex-wrap items-center gap-3 text-sm text-[color:var(--ink-soft)]">
                  <span className="rounded-full bg-white/80 px-3 py-1 font-medium text-[color:var(--ink)]">
                    Plan / Project Atlas
                  </span>
                  <span className="rounded-full bg-[color:var(--accent-ember)]/12 px-3 py-1 font-medium text-[color:var(--accent-ember-strong)]">
                    Local-first task system
                  </span>
                </div>

                <div className="max-w-3xl">
                  <p className="text-sm uppercase tracking-[0.32em] text-[color:var(--ink-soft)]">
                    V1 UI Design Draft
                  </p>
                  <h1 className="mt-3 text-4xl font-semibold tracking-[-0.08em] text-[color:var(--ink)] sm:text-5xl">
                    用一块更轻的看板，把任务推进、编辑与检索压进同一视图。
                  </h1>
                  <p className="mt-4 max-w-2xl text-base leading-7 text-[color:var(--ink-soft)] sm:text-lg">
                    这版设计稿围绕“10 秒理解当前状态、3 步创建任务、右侧面板不丢上下文”展开，
                    重点呈现顶部工具栏、横向看板和详情面板之间的层级关系。
                  </p>
                </div>
              </div>

              <div className="flex flex-col gap-3 sm:flex-row lg:flex-col">
                <button
                  type="button"
                  className="rounded-full bg-[color:var(--ink)] px-5 py-3 text-sm font-semibold text-white shadow-[0_14px_30px_rgba(39,28,20,0.22)]"
                >
                  新建任务
                </button>
                <button
                  type="button"
                  className="rounded-full border border-white/70 bg-white/70 px-5 py-3 text-sm font-semibold text-[color:var(--ink)]"
                >
                  共享设计说明
                </button>
              </div>
            </div>

            <div className="mt-6 grid gap-4 xl:grid-cols-[minmax(0,1.25fr)_320px]">
              <section className="grid gap-4 lg:grid-cols-3">
                {metrics.map((metric) => (
                  <MetricCard key={metric.label} {...metric} />
                ))}
              </section>
              <section className="rounded-[28px] border border-[color:var(--line-soft)] bg-[linear-gradient(135deg,rgba(255,244,230,0.9),rgba(249,236,226,0.82))] p-4">
                <div className="text-sm uppercase tracking-[0.28em] text-[color:var(--ink-soft)]">
                  响应式布局
                </div>
                <p className="mt-3 max-w-xs text-sm leading-6 text-[color:var(--ink-soft)]">
                  桌面端保留横向看板与右侧详情面板，移动端改为纵向信息流和抽屉式详情，视觉语义保持一致。
                </p>
                <div className="mt-5">
                  <MobilePreview />
                </div>
              </section>
            </div>
          </header>

          <div className="grid gap-6 px-5 py-5 sm:px-8 xl:grid-cols-[minmax(0,1fr)_360px]">
            <section className="space-y-5">
              <div className="rounded-[28px] border border-[color:var(--line-soft)] bg-white/70 p-4 shadow-[0_18px_40px_rgba(58,41,24,0.06)]">
                <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
                  <div className="flex flex-1 items-center gap-3 rounded-[20px] border border-[color:var(--line-soft)] bg-[color:var(--paper)] px-4 py-3 text-sm text-[color:var(--ink-soft)]">
                    <span className="text-base text-[color:var(--ink)]">⌕</span>
                    搜索任务标题、标签、评论内容
                  </div>

                  <div className="flex flex-wrap gap-2">
                    {filters.map((filter) => (
                      <button
                        key={filter.label}
                        type="button"
                        className={`rounded-full px-4 py-2.5 text-sm font-medium transition-colors ${
                          filter.active
                            ? 'bg-[color:var(--ink)] text-white'
                            : 'bg-[color:var(--panel-strong)] text-[color:var(--ink-soft)]'
                        }`}
                      >
                        {filter.label}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              <div className="overflow-x-auto pb-2">
                <div className="flex min-w-max gap-4">
                  {columns.map((column) => (
                    <BoardColumnView key={column.name} {...column} />
                  ))}
                </div>
              </div>
            </section>

            <aside className="rounded-[30px] border border-[color:var(--line-soft)] bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(249,241,233,0.96))] p-5 shadow-[0_22px_60px_rgba(52,36,22,0.09)] xl:sticky xl:top-6 xl:self-start">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="text-sm uppercase tracking-[0.28em] text-[color:var(--ink-soft)]">
                    任务详情
                  </p>
                  <h2 className="mt-3 text-2xl font-semibold tracking-[-0.06em] text-[color:var(--ink)]">
                    任务详情侧栏高保真设计
                  </h2>
                </div>
                <PriorityBadge priority="P0" />
              </div>

              <p className="mt-4 text-sm leading-7 text-[color:var(--ink-soft)]">
                详情区承担信息补全和状态确认，不抢走主看板。重点字段按照“摘要、属性、评论、活动”分层，避免一次性暴露过长表单。
              </p>

              <div className="mt-5 grid grid-cols-2 gap-3 text-sm">
                <div className="rounded-[22px] bg-[color:var(--paper)] p-4">
                  <div className="text-[11px] uppercase tracking-[0.22em] text-[color:var(--ink-soft)]">
                    状态
                  </div>
                  <div className="mt-2 font-semibold text-[color:var(--accent-teal-strong)]">
                    进行中
                  </div>
                </div>
                <div className="rounded-[22px] bg-[color:var(--paper)] p-4">
                  <div className="text-[11px] uppercase tracking-[0.22em] text-[color:var(--ink-soft)]">
                    截止时间
                  </div>
                  <div className="mt-2 font-semibold text-[color:var(--ink)]">
                    4月19日 18:00
                  </div>
                </div>
              </div>

              <section className="mt-6">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-[color:var(--ink-soft)]">
                    设计要点
                  </h3>
                  <span className="rounded-full bg-[color:var(--accent-teal)]/14 px-3 py-1 text-xs font-semibold text-[color:var(--accent-teal-strong)]">
                    自动保存
                  </span>
                </div>
                <div className="mt-4 space-y-3">
                  {detailChecklist.map((item) => (
                    <div
                      key={item.label}
                      className="flex items-center gap-3 rounded-[20px] bg-white/80 px-4 py-3"
                    >
                      <span
                        className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-xs font-semibold ${
                          item.done
                            ? 'bg-[color:var(--accent-teal)] text-white'
                            : 'bg-[color:var(--panel-strong)] text-[color:var(--ink-soft)]'
                        }`}
                      >
                        {item.done ? '✓' : '·'}
                      </span>
                      <span className="text-sm leading-6 text-[color:var(--ink)]">
                        {item.label}
                      </span>
                    </div>
                  ))}
                </div>
              </section>

              <section className="mt-6">
                <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-[color:var(--ink-soft)]">
                  标签与上下文
                </h3>
                <div className="mt-4 flex flex-wrap gap-2">
                  {['看板', '详情面板', '信息分层', 'V1'].map((tag) => (
                    <span
                      key={tag}
                      className="rounded-full bg-[color:var(--panel-strong)] px-3 py-1.5 text-xs font-medium text-[color:var(--ink-soft)]"
                    >
                      {tag}
                    </span>
                  ))}
                </div>
              </section>

              <section className="mt-6">
                <div className="flex items-center justify-between">
                  <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-[color:var(--ink-soft)]">
                    活动记录
                  </h3>
                  <span className="font-mono text-xs text-[color:var(--ink-soft)]">
                    3 updates
                  </span>
                </div>
                <div className="mt-4 space-y-3">
                  {activityItems.map((item) => (
                    <article
                      key={`${item.time}-${item.author}`}
                      className="rounded-[22px] bg-white/82 px-4 py-3"
                    >
                      <div className="flex items-center justify-between gap-3">
                        <span className="font-mono text-xs text-[color:var(--ink-soft)]">
                          {item.time}
                        </span>
                        <span className="rounded-full bg-[color:var(--paper)] px-2.5 py-1 text-xs font-semibold text-[color:var(--ink)]">
                          {item.author}
                        </span>
                      </div>
                      <p className="mt-2 text-sm leading-6 text-[color:var(--ink)]">
                        {item.text}
                      </p>
                    </article>
                  ))}
                </div>
              </section>
            </aside>
          </div>
        </div>
      </div>
    </main>
  );
}
