## Usage of this tool

```
make run FORMAT=mermaid FEATURES=f1,f2,f3 DEFAULT=n
```

the `FORMAT` can be either **mermaid** or **d2** (default is mermaid), it will out put the result under the deptool directory.

the `TARGET` should be any existed crate or module name, or the path under app directory: eg helloworld, net/httpserver

the `FEATURES` should be the features you want to use, this should be separated by ","

the `DEFAULT` is used to control enable default features or not **n** for no and **y** for yes

the first time you run this tool to analyze a crate/module/app will be slow or blocked for downloading the needed crates for the target

if you think this makefile too naive, you can just run `cargo build`, and then use `./target/debug/deptool -h` to see the available options to use
