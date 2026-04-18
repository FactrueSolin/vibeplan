import { fail, ok } from '@/lib/server/http';
import { getBoard } from '@/lib/server/mock-db';

type Context = {
  params: Promise<{
    projectId: string;
  }>;
};

export async function GET(request: Request, context: Context) {
  const { projectId } = await context.params;
  const { searchParams } = new URL(request.url);
  const includeArchived = searchParams.get('includeArchived') === 'true';
  const board = await getBoard(projectId, includeArchived);

  if (!board) {
    return fail(404, 'not_found', '项目不存在');
  }

  return ok(board);
}
