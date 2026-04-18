import { NextResponse } from 'next/server';

import type { ApiSuccess } from '@/lib/types';

function createRequestId() {
  return `req_${Math.random().toString(36).slice(2, 10)}`;
}

export function ok<T>(data: T, meta?: Partial<ApiSuccess<T>['meta']>) {
  return NextResponse.json({
    data,
    meta: {
      requestId: createRequestId(),
      ...meta,
    },
  } satisfies ApiSuccess<T>);
}

export function fail(
  status: number,
  code:
    | 'validation_error'
    | 'not_found'
    | 'conflict'
    | 'invalid_operation'
    | 'internal_error',
  message: string,
  details?: Record<string, string | number | boolean | null>,
) {
  return NextResponse.json(
    {
      error: {
        code,
        message,
        details,
        requestId: createRequestId(),
      },
    },
    {
      status,
    },
  );
}
