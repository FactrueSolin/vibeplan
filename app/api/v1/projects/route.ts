import { z } from 'zod';

import { fail, ok } from '@/lib/server/http';
import { createProject, listProjects } from '@/lib/server/mock-db';

export async function GET() {
  return ok(await listProjects());
}

export async function POST(request: Request) {
  try {
    const body = (await request.json()) as Record<string, unknown>;

    return ok(await createProject(body));
  } catch (error) {
    if (error instanceof z.ZodError) {
      return fail(422, 'validation_error', '项目信息不完整', {
        field: error.issues[0]?.path.join('.') ?? 'name',
      });
    }

    return fail(500, 'internal_error', '创建项目失败');
  }
}
