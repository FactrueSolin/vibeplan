import { ok } from '@/lib/server/http';
import { listTags } from '@/lib/server/mock-db';

type Context = {
  params: Promise<{
    projectId: string;
  }>;
};

export async function GET(_request: Request, context: Context) {
  const { projectId } = await context.params;

  return ok(await listTags(projectId));
}
