# arceos/modules 發布至 crates.io 順序

依「被依賴者先發布」的拓撲順序，各 crate 僅列出**對同屬 `modules/` 的其它 crate** 的依賴。

## 依賴關係摘要

| Crate | 依賴的 modules |
|-------|----------------|
| axconfig | — |
| axlog | — |
| axalloc | — |
| axhal | axconfig |
| axtask | axhal |
| axsync | axtask |
| axipi | axconfig, axhal |
| axdma | axalloc, axconfig, axhal |
| axdriver | axalloc?, axconfig?, axdma?, axhal?（均 optional） |
| axdisplay | axdriver, axsync |
| axinput | axdriver, axsync |
| axfs | axalloc, axdriver, axhal, axsync |
| axmm | axalloc, axconfig, axhal, axsync, axtask；axfs?（optional） |
| axnet | axconfig, axdriver, axhal, axsync, axtask, axfs |
| axruntime | axconfig, axhal, axlog；其餘多為 optional |

## 建議發布順序

按下列順序執行 `cargo publish -p <name>`（在 arceos 倉庫根目錄）：

1. **axconfig** — 無 module 依賴  
2. **axlog** — 無 module 依賴  
3. **axalloc** — 無 module 依賴  
4. **axhal** — 依賴 axconfig  
5. **axtask** — 依賴 axhal  
6. **axsync** — 依賴 axtask  
7. **axipi** — 依賴 axconfig, axhal  
8. **axdma** — 依賴 axalloc, axconfig, axhal  
9. **axdriver** — 依賴 axalloc, axconfig, axdma, axhal（皆 optional）  
10. **axdisplay** — 依賴 axdriver, axsync  
11. **axinput** — 依賴 axdriver, axsync  
12. **axfs** — 依賴 axalloc, axdriver, axhal, axsync  
13. **axmm** — 依賴 axalloc, axconfig, axhal, axsync, axtask（axfs optional）  
14. **axnet** — 依賴 axconfig, axdriver, axhal, axsync, axtask, axfs  
15. **axruntime** — 依賴其餘 modules（多為 optional）

## 注意

- 上述僅考慮 **modules 內** 的依賴；各 crate 還依賴 workspace 中的 **外部** 依賴（如 `allocator`、`axdriver_*`、`axplat`、`axcpu` 等 git/crates.io）。發布前須確保這些依賴已存在於 crates.io 或改為可發布的 version 依賴。
- 含 **git 依賴** 的 crate 無法直接發布，需先改為 crates.io 版本或移除該依賴。
- 每發布一個 crate 後，再對依賴它的 crate 執行 `cargo publish --dry-run` 以確認可通過。
