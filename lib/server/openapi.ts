export const openApiDocument = {
  openapi: '3.1.0',
  info: {
    title: 'Plan API',
    version: 'v1',
    description: 'Mock contract for the local task management frontend.',
  },
  servers: [
    {
      url: 'http://localhost:3000/api/v1',
    },
  ],
  tags: [
    { name: 'Projects' },
    { name: 'Tasks' },
    { name: 'Comments' },
    { name: 'Tags' },
    { name: 'System' },
  ],
  paths: {
    '/projects': {
      get: {
        operationId: 'listProjects',
        tags: ['Projects'],
      },
      post: {
        operationId: 'createProject',
        tags: ['Projects'],
      },
    },
    '/projects/{projectId}/board': {
      get: {
        operationId: 'getBoardSnapshot',
        tags: ['Projects'],
      },
    },
    '/projects/{projectId}/tasks': {
      post: {
        operationId: 'createProjectTask',
        tags: ['Tasks'],
      },
    },
    '/projects/{projectId}/tasks/reorder': {
      post: {
        operationId: 'reorderProjectTasks',
        tags: ['Tasks'],
      },
    },
    '/projects/{projectId}/tags': {
      get: {
        operationId: 'listProjectTags',
        tags: ['Tags'],
      },
    },
    '/tasks/{taskId}': {
      get: {
        operationId: 'getTask',
        tags: ['Tasks'],
      },
      patch: {
        operationId: 'updateTask',
        tags: ['Tasks'],
      },
    },
    '/tasks/{taskId}/archive': {
      post: {
        operationId: 'archiveTask',
        tags: ['Tasks'],
      },
    },
    '/tasks/{taskId}/restore': {
      post: {
        operationId: 'restoreTask',
        tags: ['Tasks'],
      },
    },
    '/tasks/{taskId}/comments': {
      get: {
        operationId: 'listTaskComments',
        tags: ['Comments'],
      },
      post: {
        operationId: 'createTaskComment',
        tags: ['Comments'],
      },
    },
    '/openapi.json': {
      get: {
        operationId: 'getOpenApiDocument',
        tags: ['System'],
      },
    },
  },
};
