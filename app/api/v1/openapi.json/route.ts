import { ok } from '@/lib/server/http';
import { openApiDocument } from '@/lib/server/openapi';

export async function GET() {
  return ok(openApiDocument);
}
