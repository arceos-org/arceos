# 获取一个程序的所有系统调用，严格来说存在bug
import re

# 逐行读入debug.S的内容

import sys

assert (len(sys.argv) >= 3)

f = open(sys.argv[1])

g = open(sys.argv[2], "w+")

lines = f.readlines()

syscall_id = 0  # syscall id，记录在 a7 寄存器中

fs_syscall_id = [17, 20, 21, 22, 23, 24, 25, 29, 34, 35, 37, 39, 40, 43, 46, 48, 49, 53, 56, 57, 59, 61, 62, 63, 64,
                 65, 66, 73, 79, 67, 68, 71, 72, 78, 80, 81, 82, 88, 276, 285]

mem_syscall_id = [214, 215, 222, 227, 226, 283, 194, 195, 196]

net_syscall_id = [198, 200, 201, 202, 203,
                  204, 205, 206, 207, 208, 209, 210, 242]

task_syscall_id = [93, 94, 96, 101, 102, 103, 114, 115, 116, 157, 165, 166, 167, 172,
                   173, 174, 175, 176, 177, 178, 179, 199, 220, 221, 233, 260, 278, 124, 113, 137, 153, 160, 169, 261]

futex_id = [98, 99, 100]

schedule_id = [119, 120, 122, 123]

signal_id = [129, 130, 133, 134, 135, 139]

syscall_task = False
syscall_fs = False
syscall_mem = False
syscall_net = False
futex = False
schedule = False
signal = False
for line in lines:
    # 多个空格转化为一个空格
    line_after = re.sub(' +', ' ', line)
    # 制表符转化为一个空格
    line_after = line_after.replace("\t", " ")
    # 去除前面的空格
    line_after = line_after.strip()
    # 前面可能有大量的二进制数据，所以我们要从尾部开始
    line_after = line_after.split(",")

    # 如果以li开头，说明是li指令
    if line_after[0].endswith("li a7"):
        # 以逗号分割
        if (line_after.__len__() != 2):
            continue
        syscall_id = int(line_after[1])
    # 如果以ecall开头，说明是ecall指令
    elif line_after[0].endswith("ecall"):
        # 如果是ecall，那么就输出syscall id
        # print(syscall_id)
        # g.write(str(syscall_id) + "\n")
        if (syscall_id in fs_syscall_id):
            syscall_fs = True
        elif (syscall_id in mem_syscall_id):
            syscall_mem = True
        elif (syscall_id in net_syscall_id):
            syscall_net = True
        elif (syscall_id in task_syscall_id):
            syscall_task = True
        elif (syscall_id in futex_id):
            futex = True
            syscall_task = True
        elif (syscall_id in schedule_id):
            schedule = True
            syscall_task = True
        elif (syscall_id in signal_id):
            signal = True
            syscall_task = True

if (syscall_fs):
    g.write("syscall_fs\n")
if (syscall_mem):
    g.write("syscall_mem\n")
if (syscall_net):
    g.write("syscall_net\n")
if (syscall_task):
    g.write("syscall_task\n")
if (futex):
    g.write("futex\n")
if (schedule):
    g.write("schedule\n")
if (signal):
    g.write("signal\n")
if (len(sys.argv) > 3):
    if (sys.argv[3] == "img"):
        g.write("img\n")

g.close()
f.close()
