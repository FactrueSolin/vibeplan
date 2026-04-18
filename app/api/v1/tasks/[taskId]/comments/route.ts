import { z } from 'zod';

import { createComment, listComments } from '@/lib/server/mock-db';
import { fail, ok } from '@/lib/server/http';

type Context = {
  params: Promise<{
    taskId: string;
  }>;
};

export async function GET(_request: Request, context: Context) {
  const { taskId } = await context.params;

  return ok(await listComments(taskId));
}

export async function POST(request: Request, context: Context) {
  const { taskId } = await context.params;

  try {
    const body = (await request.json()) as Record<string, unknown>;
    const comment = await createComment(taskId, body);

    if (!comment) {
      return fail(404, 'not_found', '任务不存在');
    }

    return ok(comment);
  } catch (error) {
    if (error instanceof z.ZodError) {
      return fail(422, 'validation_error', '评论内容不能为空', {
        field: error.issues[0]?.path.join('.') ?? 'content',
      });
    }

    return fail(500, 'internal_error', '新增评论失败');
  }
}
