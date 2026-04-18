import { z } from 'zod';

import { fail, ok } from '@/lib/server/http';
import { reorderTasks } from '@/lib/server/mock-db';

type Context = {
  params: Promise<{
    projectId: string;
  }>;
};

export async function POST(request: Request, context: Context) {
  const { projectId } = await context.params;

  try {
    const body = (await request.json()) as Record<string, unknown>;
    const board = await reorderTasks(projectId, body);

    if (!board) {
      return fail(404, 'not_found', '项目不存在');
    }

    return ok(board);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return fail(422, 'validation_error', '拖拽排序参数非法', {
        field: error.issues[0]?.path.join('.') ?? 'orderedTaskIds',
      });
    }

    return fail(500, 'internal_error', '任务排序失败');
  }
}
