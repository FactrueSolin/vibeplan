import type {
  BoardSnapshotDto,
  ReorderTasksInput,
  StatusDto,
  TaskDto,
  TaskListFilters,
} from '@/lib/types';
import { isOverdue } from '@/lib/utils';

export function sortStatuses(statuses: StatusDto[]) {
  return [...statuses].sort((left, right) => left.sortOrder - right.sortOrder);
}

export function sortTasks(tasks: TaskDto[]) {
  return [...tasks].sort((left, right) => left.position - right.position);
}

export function computeBoardSummary(snapshot: Pick<BoardSnapshotDto, 'statuses' | 'tasks'>) {
  const doneStatusIds = new Set(
    snapshot.statuses.filter((status) => status.isDone).map((status) => status.id),
  );

  return snapshot.tasks.reduce(
    (summary, task) => {
      if (task.archivedAt) {
        summary.archivedTaskCount += 1;
      } else if (doneStatusIds.has(task.statusId)) {
        summary.doneTaskCount += 1;
      } else {
        summary.activeTaskCount += 1;
      }

      return summary;
    },
    {
      activeTaskCount: 0,
      doneTaskCount: 0,
      archivedTaskCount: 0,
    },
  );
}

export function applyTaskFilters(
  snapshot: BoardSnapshotDto,
  filters: TaskListFilters,
) {
  const search = filters.q.trim().toLowerCase();

  return snapshot.tasks.filter((task) => {
    if (filters.archived === 'exclude' && task.archivedAt) {
      return false;
    }

    if (filters.archived === 'only' && !task.archivedAt) {
      return false;
    }

    if (filters.statusId !== 'all' && task.statusId !== filters.statusId) {
      return false;
    }

    if (filters.priority !== 'all' && task.priority !== filters.priority) {
      return false;
    }

    if (filters.tagId !== 'all' && !task.tagIds.includes(filters.tagId)) {
      return false;
    }

    if (
      search &&
      !`${task.title} ${task.description ?? ''}`.toLowerCase().includes(search)
    ) {
      return false;
    }

    if (filters.sortBy === 'dueDate') {
      return true;
    }

    return true;
  });
}

export function sortFilteredTasks(tasks: TaskDto[], filters: TaskListFilters) {
  const direction = filters.sortOrder === 'asc' ? 1 : -1;

  return [...tasks].sort((left, right) => {
    if (filters.sortBy === 'position') {
      return (left.position - right.position) * direction;
    }

    if (filters.sortBy === 'createdAt') {
      return (
        (new Date(left.createdAt).getTime() - new Date(right.createdAt).getTime()) *
        direction
      );
    }

    if (filters.sortBy === 'updatedAt') {
      return (
        (new Date(left.updatedAt).getTime() - new Date(right.updatedAt).getTime()) *
        direction
      );
    }

    const leftValue = left.dueDate ? new Date(left.dueDate).getTime() : Number.MAX_SAFE_INTEGER;
    const rightValue = right.dueDate
      ? new Date(right.dueDate).getTime()
      : Number.MAX_SAFE_INTEGER;

    return (leftValue - rightValue) * direction;
  });
}

export function applyTaskReorder(
  snapshot: BoardSnapshotDto,
  input: ReorderTasksInput,
) {
  const now = new Date().toISOString();
  const doneStatusIds = new Set(
    snapshot.statuses.filter((status) => status.isDone).map((status) => status.id),
  );

  const nextTasks = snapshot.tasks.map((task) => {
    if (task.statusId === input.sourceStatusId && task.id !== input.movedTaskId) {
      return task;
    }

    if (task.statusId === input.destinationStatusId || task.id === input.movedTaskId) {
      return task;
    }

    return task;
  });

  const movedTask = nextTasks.find((task) => task.id === input.movedTaskId);

  if (!movedTask) {
    return snapshot;
  }

  const sourceTasks = sortTasks(
    nextTasks.filter(
      (task) =>
        task.statusId === input.sourceStatusId && task.id !== input.movedTaskId,
    ),
  ).map((task, index) => ({
    ...task,
    position: index,
  }));

  const destinationTasks = input.orderedTaskIds
    .map((taskId, index) => {
      if (taskId === input.movedTaskId) {
        const movedToDone = doneStatusIds.has(input.destinationStatusId);

        return {
          ...movedTask,
          statusId: input.destinationStatusId,
          position: index,
          completedAt: movedToDone ? now : null,
          updatedAt: now,
        };
      }

      const existingTask = nextTasks.find((task) => task.id === taskId);

      if (!existingTask) {
        return null;
      }

      return {
        ...existingTask,
        statusId: input.destinationStatusId,
        position: index,
        completedAt: doneStatusIds.has(input.destinationStatusId)
          ? existingTask.completedAt ?? now
          : null,
        updatedAt: now,
      };
    })
    .filter((task): task is TaskDto => Boolean(task));

  const remainingTasks = nextTasks.filter(
    (task) =>
      task.statusId !== input.sourceStatusId &&
      task.statusId !== input.destinationStatusId &&
      task.id !== input.movedTaskId,
  );

  const tasks = [...remainingTasks, ...sourceTasks, ...destinationTasks];

  return {
    ...snapshot,
    tasks,
    summary: computeBoardSummary({
      statuses: snapshot.statuses,
      tasks,
    }),
  };
}

export function countOverdueTasks(tasks: TaskDto[]) {
  return tasks.filter((task) => !task.archivedAt && isOverdue(task.dueDate)).length;
}
