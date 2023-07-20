extern crate alloc;
use core::{
    mem::{align_of, size_of},
    ptr::null,
};

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use axlog::info;

pub const USER_INIT_STACK_SIZE: usize = 0x4000;
/// 规定用户栈初始化时的内容
pub struct UserStack {
    /// 当前的用户栈的栈顶(低地址)
    sp: usize,
    /// 当前的用户栈的栈顶(高地址)
    bottom: usize,
    /// data保存了用户栈上的信息
    pub data: Vec<u8>,
}

impl UserStack {
    pub fn new(sp: usize) -> Self {
        let mut data: Vec<u8> = Vec::with_capacity(USER_INIT_STACK_SIZE);
        unsafe {
            // 规定好初始的用户栈大小，防止越界
            data.set_len(USER_INIT_STACK_SIZE);
        }
        Self {
            sp,
            bottom: sp,
            data,
        }
    }
    pub fn get_data_front_ref(&self) -> &[u8] {
        let offset = self.data.len() - (self.bottom - self.sp);
        &self.data[offset..]
    }
    #[allow(unused)]
    pub fn get_data_front_mut_ref(&mut self) -> &mut [u8] {
        let offset = self.data.len() - (self.bottom - self.sp);
        &mut self.data[offset..]
    }
    /// 插入一段数据到用户栈中
    /// 返回的是插入后的用户栈顶，即这段数据的起始位置
    pub fn push<T: Copy>(&mut self, data: &[T]) {
        self.sp -= data.len() * size_of::<T>();
        self.sp -= self.sp % align_of::<T>();
        let offset = self.data.len() - (self.bottom - self.sp);
        unsafe {
            core::slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(offset) as *mut T,
                data.len(),
            )
        }
        .copy_from_slice(data);
    }
    /// 记得插入后补0
    pub fn push_str(&mut self, str: &str) -> usize {
        self.push(&[b'\0']);
        self.push(str.as_bytes());
        self.sp
    }
    pub fn get_sp(&self) -> usize {
        self.sp
    }
    // 获取真实的栈占用的内容
    pub fn get_len(&self) -> usize {
        self.bottom - self.sp
    }
}

/// 初始化用户栈
pub fn init_stack(
    args: Vec<String>,
    envs: Vec<String>,
    auxv: BTreeMap<u8, usize>,
    sp: usize,
) -> UserStack {
    let mut stack = UserStack::new(sp);
    let random_str: &[usize; 2] = &[1145141919810114514usize, 6316312363615238936usize];
    stack.push(random_str.as_slice());
    let random_str_pos = stack.get_sp();
    // 按照栈的结构，先加入envs和argv的对应实际内容
    let envs: Vec<_> = envs
        .iter()
        .map(|env| stack.push_str(env.as_str()))
        .collect();
    let argv: Vec<_> = args
        .iter()
        .map(|arg| stack.push_str(arg.as_str()))
        .collect();
    // 加入envs和argv的地址
    stack.push(&[null::<u8>(), null::<u8>()]);
    // 再加入auxv
    // 注意若是atrandom，则要指向栈上的一个16字节长度的随机字符串
    for (key, value) in auxv.iter() {
        if (*key) == 25 {
            // AT RANDOM
            stack.push(&[*key as usize, random_str_pos]);
        } else {
            stack.push(&[*key as usize, *value]);
        }
    }
    // 加入envs和argv的地址
    stack.push(&[null::<u8>()]);
    stack.push(envs.as_slice());
    stack.push(&[null::<u8>()]);
    stack.push(argv.as_slice());
    // 加入argc
    info!("args: len: {}", args.len());
    info!("argv: {:x?}", argv);
    stack.push(&[args.len()]);
    stack
}
