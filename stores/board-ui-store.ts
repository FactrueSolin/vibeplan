import { create } from 'zustand';

import type { Priority, TaskListFilters } from '@/lib/types';

type BoardUiState = {
  filters: TaskListFilters;
  isCreateTaskOpen: boolean;
  isCreateProjectOpen: boolean;
  setSearch: (value: string) => void;
  setStatusId: (value: string) => void;
  setPriority: (value: Priority | 'all') => void;
  setTagId: (value: string) => void;
  setArchived: (value: TaskListFilters['archived']) => void;
  clearFilters: () => void;
  setCreateTaskOpen: (value: boolean) => void;
  setCreateProjectOpen: (value: boolean) => void;
  resetForProject: () => void;
};

const initialFilters: TaskListFilters = {
  q: '',
  statusId: 'all',
  priority: 'all',
  tagId: 'all',
  archived: 'exclude',
  sortBy: 'position',
  sortOrder: 'asc',
};

export const useBoardUiStore = create<BoardUiState>((set) => ({
  filters: initialFilters,
  isCreateTaskOpen: false,
  isCreateProjectOpen: false,
  setSearch: (value) =>
    set((state) => ({
      filters: {
        ...state.filters,
        q: value,
      },
    })),
  setStatusId: (value) =>
    set((state) => ({
      filters: {
        ...state.filters,
        statusId: value,
      },
    })),
  setPriority: (value) =>
    set((state) => ({
      filters: {
        ...state.filters,
        priority: value,
      },
    })),
  setTagId: (value) =>
    set((state) => ({
      filters: {
        ...state.filters,
        tagId: value,
      },
    })),
  setArchived: (value) =>
    set((state) => ({
      filters: {
        ...state.filters,
        archived: value,
      },
    })),
  clearFilters: () =>
    set(() => ({
      filters: initialFilters,
    })),
  setCreateTaskOpen: (value) => set(() => ({ isCreateTaskOpen: value })),
  setCreateProjectOpen: (value) => set(() => ({ isCreateProjectOpen: value })),
  resetForProject: () =>
    set(() => ({
      filters: initialFilters,
      isCreateTaskOpen: false,
    })),
}));
