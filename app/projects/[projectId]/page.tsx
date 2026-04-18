import { BoardPage } from '@/features/board/board-page';

type ProjectBoardPageProps = {
  params: Promise<{
    projectId: string;
  }>;
  searchParams: Promise<{
    taskId?: string;
  }>;
};

export default async function ProjectBoardPage({
  params,
  searchParams,
}: ProjectBoardPageProps) {
  const { projectId } = await params;
  const { taskId } = await searchParams;

  return <BoardPage projectId={projectId} taskId={taskId ?? null} />;
}
