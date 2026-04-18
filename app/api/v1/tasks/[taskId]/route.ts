import { z } from 'zod';

import { fail, ok } from '@/lib/server/http';
import { getTask, updateTask } from '@/lib/server/mock-db';

type Context = {
  params: Promise<{
    taskId: string;
  }>;
};

export async function GET(_request: Request, context: Context) {
  const { taskId } = await context.params;
  const task = await getTask(taskId);

  if (!task) {
    return fail(404, 'not_found', '任务不存在');
  }

  return ok(task);
}

export async function PATCH(request: Request, context: Context) {
  const { taskId } = await context.params;

  try {
    const body = (await request.json()) as Record<string, unknown>;
    const task = await updateTask(taskId, body);

    if (!task) {
      return fail(404, 'not_found', '任务或状态不存在');
    }

    return ok(task);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return fail(422, 'validation_error', '任务字段校验失败', {
        field: error.issues[0]?.path.join('.') ?? 'title',
      });
    }

    return fail(500, 'internal_error', '更新任务失败');
  }
}
