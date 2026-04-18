import { fail, ok } from '@/lib/server/http';
import { restoreTask } from '@/lib/server/mock-db';

type Context = {
  params: Promise<{
    taskId: string;
  }>;
};

export async function POST(_request: Request, context: Context) {
  const { taskId } = await context.params;
  const task = await restoreTask(taskId);

  if (!task) {
    return fail(404, 'not_found', '任务不存在');
  }

  return ok(task);
}
