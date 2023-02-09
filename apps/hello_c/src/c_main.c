extern void dummy_syscall(int a0, int a1);

int c_main() {
    dummy_syscall(5, 55);
    return 7;
}