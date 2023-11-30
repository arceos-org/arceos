- [ ] bindgen使用的clang编译器不支持target为 'loongarch64-unknown-none'(其实本来也没有这个选项)，产生报错：error: unknown target triple 'loongarch64-unknown-none', please use -triple or -arch
    >通过'clang -print-target-triple' 可以寻找到支持的arch，在对应的build.rs中增加 
    .clang_arg("--target=your_target")'可以解决问题，如下面代码所示。
            let mut builder = bindgen::Builder::default()
            .header(in_file)
            .clang_arg("--target=x86_64-pc-linux-gnu")
            .clang_arg("-I./include")
            .derive_default(true)
            .size_t_is_usize(false)
            .use_core();
    由于我们只需要bindgen库将'cyptes.h'改为rust定义的变量，生成libctypes_gen.rs文件，所以我认为将这里的target修改为其他架构问题应该不大？？如果有问题以后再改（
    >
