<!-- BEGIN:nextjs-agent-rules -->
# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` before writing any code. Heed deprecation notices.
<!-- END:nextjs-agent-rules -->
<info>
项目简述
前端基于nextjs，后端基于rust，是一个前后端分离项目。
</info>
<info>
rust代码规范
1. 严格遵守rust的lib和bin最佳实践
2. 拆分不同功能的代码进不同的模块
3. 代码仅需通过cargo check，其他交由用户手动测试
4. 优先使用已有的第三库
</info>
<info>
typescript项目规范
1. 禁止使用any类型
</info>
<info>
项目规范
1. 注意整个代码仓库的可维护性
</info>
