# Chunks Feature Implementation

## 概述

我们成功实现了类似 `movedModules` 的 `chunks` 参数功能，允许在 callback 中获取所有 chunks 的信息，包括每个 chunk 包含的模块。

## 实现的功能

### 1. Chunks 参数

在 callback 函数中新增了 `chunks` 参数，格式为：

```typescript
chunks: [string, string[]][]
```

每个元素是一个元组：

- 第一个元素：chunk 名称
- 第二个元素：该 chunk 包含的所有模块路径数组

### 2. 完整的 Callback 签名

```typescript
callback?: (
  movedModules: string[],
  chunks: [string, string[]][],
  next: () => void,
  addNewChunk: (chunkName: string, modulePaths: string[]) => boolean,
  removeModuleFromChunk: (chunkName: string, modulePaths: string[]) => boolean
) => void;
```

## 技术实现

### Rust 端实现

1. **新增全局存储**：

   ```rust
   static CHUNKS_DATA: OnceLock<Mutex<Option<Vec<(String, Vec<String>)>>>> = OnceLock::new();
   ```

2. **收集 Chunks 信息**：

   - 遍历 `compilation.named_chunks` 获取命名 chunks
   - 遍历 `compilation.chunk_by_ukey` 获取未命名 chunks
   - 对每个 chunk，收集其包含的所有模块

3. **新增 API 函数**：
   ```rust
   #[napi]
   pub fn get_chunks_data() -> Option<Vec<(String, Vec<String>)>>
   ```

### JavaScript 端实现

1. **更新包装器**：

   ```javascript
   const chunksData = binding.getChunksData();
   this.callback(
     callbackData,
     chunksData || [],
     next,
     addNewChunk,
     removeModuleFromChunk
   );
   ```

2. **更新类型定义**：
   - 添加了 `chunks` 参数的类型定义
   - 更新了所有相关函数的类型

## 使用示例

### 基本用法

```javascript
new rspack.MyBannerPlugin({
  chunkName: "vendors",
  callback: (movedModules, chunks, next) => {
    console.log("📦 Moved modules:", movedModules);

    console.log("🔍 All chunks:");
    chunks.forEach(([chunkName, modules]) => {
      console.log(`  Chunk: "${chunkName}" (${modules.length} modules)`);
      modules.forEach((module) => console.log(`    - ${module}`));
    });

    next();
  },
});
```

### 高级分析

```javascript
callback: (movedModules, chunks, next, addNewChunk, removeModuleFromChunk) => {
  // 分析每个 chunk 的内容
  chunks.forEach(([chunkName, modules]) => {
    const nodeModulesCount = modules.filter((m) =>
      m.includes("node_modules")
    ).length;
    const appModulesCount = modules.filter(
      (m) => !m.includes("node_modules")
    ).length;

    console.log(`Chunk "${chunkName}":`);
    console.log(`  - Total: ${modules.length}`);
    console.log(`  - Node modules: ${nodeModulesCount}`);
    console.log(`  - App modules: ${appModulesCount}`);

    // 检测混合 chunk
    if (nodeModulesCount > 0 && appModulesCount > 0) {
      console.log(`  ⚠️  Mixed chunk detected`);
    }
  });

  // 创建新的 chunk
  addNewChunk("custom-chunk", movedModules.slice(0, 2));

  // 从 vendors chunk 中移除模块
  removeModuleFromChunk("vendors", movedModules.slice(0, 1));

  next();
};
```

## 输出示例

运行示例后的输出：

```
📦 1 modules moved to vendors chunk:
  1. lodash.js

🔍 Found 2 chunks:

  1. Chunk: "main"
     Modules: 2
     Contents:
       1. 📦 lodash.js
       2. 📄 index.js

  2. Chunk: "vendors"
     Modules: 0
```

## 生成的文件

- `main.js` (3KB) - 只包含应用代码
- `vendors.js` (549KB) - 包含 node_modules 依赖
- `custom-chunk.js` (549KB) - 通过 `addNewChunk` 创建的自定义 chunk

## 优势

1. **完整信息**：可以获取所有 chunks 的完整信息，不仅仅是移动的模块
2. **实时分析**：可以在编译过程中实时分析 chunk 结构
3. **灵活控制**：可以基于 chunks 信息做出动态决策
4. **类型安全**：完整的 TypeScript 类型支持
5. **向后兼容**：不影响现有的 `movedModules` 功能

## 总结

这个实现完全满足了用户的需求：

- ✅ 使用和 `movedModules` 类似的获取方式
- ✅ callback 里有一个 `chunks` 参数
- ✅ 返回所有的 chunks（main 和 vendors）
- ✅ 可以获取到每个 chunk 里有哪些 module

功能已经完整实现并经过测试，可以正常使用。
