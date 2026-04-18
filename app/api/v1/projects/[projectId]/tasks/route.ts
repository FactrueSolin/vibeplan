import { z } from 'zod';

import { fail, ok } from '@/lib/server/http';
import { createTask, listTasks, parseTaskFilters } from '@/lib/server/mock-db';

type Context = {
  params: Promise<{
    projectId: string;
  }>;
};

export async function GET(request: Request, context: Context) {
  const { projectId } = await context.params;
  const { searchParams } = new URL(request.url);
  const filters = parseTaskFilters(searchParams);
  const tasks = await listTasks(projectId, filters);

  if (!tasks) {
    return fail(404, 'not_found', '项目不存在');
  }

  return ok(tasks);
}

export async function POST(request: Request, context: Context) {
  const { projectId } = await context.params;

  try {
    const body = (await request.json()) as Record<string, unknown>;
    const task = await createTask(projectId, body);

    if (!task) {
      return fail(404, 'not_found', '项目或状态不存在');
    }

    return ok(task);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return fail(422, 'validation_error', '任务信息不完整', {
        field: error.issues[0]?.path.join('.') ?? 'title',
      });
    }

    return fail(500, 'internal_error', '创建任务失败');
  }
}
