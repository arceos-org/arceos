# 第十四周汇报

**致理-信计01  佟海轩 2020012709**

## 本周进展

### 构建

#### 链接时的浮点问题

`make A=apps/net/httpclient/ LOG=info NET=y ARCH=x86_64 ACCEL=n run` 在 x86 和 arm 卡住。

编译时禁用浮点解决：

``` diff
match arch {
    "riscv64" => {
        base_config.compiler("riscv64-linux-musl-gcc");
        base_config.flag("-mabi=lp64d");
    }
    "aarch64" => {
        base_config.compiler("aarch64-linux-musl-gcc");
+       base_config.flag("-mgeneral-regs-only");
    }
    "x86_64" => {
        base_config.compiler("x86_64-linux-musl-gcc");
+       base_config.flag("-mno-sse");
    }
    _ => {
        panic!("Unsupported arch: {}", arch);
    }
}
```

#### 移除 `c_libax` 依赖

手动实现如下内容即可：

``` c
typedef long long ssize_t;
int atoi(const char *s);
int strcmp(const char *l, const char *r);
```

#### 在 Makefile 中选择构建工具

export `CC` 和 `ARCH_CFLAGS` （设置浮点相关 flag），编译时使用即可。

``` makefile
CC=$(CC) CFLAGS="$(ARCH_CFLAGS)" cargo rustc $(build_args) $(1) -- $(rustc_flags)
```

#### 优化启用方式

使用 `APP_FEATURES=libax/use-lwip` 启用 lwip 网络栈

### CI

#### 减少 musl 工具链的下载

当仅当 `CARGO_CFG_TARGET_OS` 为 `none` （非生成 doc）和 `CLIPPY_ARGS` 不存在（非 clippy） 时进行 lwip 的编译

对于 unit_test，添加参数 `--exclude lwip_rust` 避免编译

#### 在缺少 musl 工具链时进行 bindgen

CI 中添加 `sudo apt update && sudo apt install -y gcc-multilib`

### 测试

对于所有网络相关测试，添加一份 `APP_FEATURES` 增加 `libax/use-lwip` 的测试

### 性能优化

### 性能测试

性能测试时修改 `./modules/axconfig/src/platform/*.toml` 中的内存大小

### PR

## 下周计划
