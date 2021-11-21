use inkwell::basic_block::BasicBlock;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;
use x_lang_ast::shared::{Kind, KindName};

#[derive(Debug)]
pub struct FunctionScope<'ctx> {
    pub fn_value: FunctionValue<'ctx>,
    pub return_kind: Kind,
    pub arg_kind_names: Vec<KindName>,
    pub arg_variables: Vec<(String, Kind, BasicValueEnum<'ctx>, usize)>, // 形参信息，预编译的时候设置，真实编译的时候再生效
    pub entry_block: Option<BasicBlock<'ctx>>,
}

#[derive(Debug)]
pub enum ScopeType<'ctx> {
    Function(FunctionScope<'ctx>),
    Variable {
        kind: Kind,
        ptr: Option<PointerValue<'ctx>>,
    },
}

impl<'ctx> ScopeType<'ctx> {
    pub fn is_fn(&self) -> bool {
        match self {
            ScopeType::Function { .. } => true,
            ScopeType::Variable { .. } => false,
        }
    }

    pub fn is_var(&self) -> bool {
        !self.is_fn()
    }

    pub fn get_fn(&self) -> &FunctionScope<'ctx> {
        match self {
            ScopeType::Function(v) => v,
            ScopeType::Variable { .. } => panic!("Error"),
        }
    }

    pub fn get_var(&self) -> (&Kind, &Option<PointerValue<'ctx>>) {
        match self {
            ScopeType::Function { .. } => panic!("Error"),
            ScopeType::Variable { kind, ptr } => (kind, ptr),
        }
    }
}

#[derive(Debug)]
pub struct Scope<'ctx> {
    pub basic_block: Option<BasicBlock<'ctx>>,
    map: HashMap<String, ScopeType<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn new(basic_block: Option<BasicBlock<'ctx>>) -> Self {
        Scope {
            basic_block,
            map: HashMap::new(),
        }
    }

    // 根据名称返回 scope 类型
    #[inline]
    pub fn get(&self, name: &str) -> Option<&ScopeType<'ctx>> {
        self.map.get(name)
    }

    // 是否存在某个 scope 名
    #[inline]
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    // 新增一个 scope 命名空间
    pub fn add(&mut self, name: &str, scope_type: ScopeType<'ctx>) {
        self.map.insert(name.to_string(), scope_type);
    }

    // 移出一个 scope 命名空间
    pub fn remove(&mut self, name: &str) {
        self.map.remove(name);
    }

    // 清除当前作用域所有变量和方法声明
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

#[derive(Debug)]
pub struct BlockScope<'ctx> {
    pub external: Scope<'ctx>,      // 外部绑定的作用域
    pub fns: Scope<'ctx>,           // 全局定义的方法（只能在全局定义）
    scope_chains: Vec<Scope<'ctx>>, // 变量作用域链
}

impl<'ctx> BlockScope<'ctx> {
    pub fn new() -> Self {
        BlockScope {
            external: Scope::new(None),
            fns: Scope::new(None),
            scope_chains: vec![],
        }
    }

    // 将一个新的块级作用域压入栈中
    pub fn push(&mut self, basic_block: BasicBlock<'ctx>) {
        let scope = Scope::new(Some(basic_block));
        self.scope_chains.push(scope);
    }

    // 当前块级作用域出栈
    pub fn pop(&mut self) {
        self.scope_chains.pop();
    }

    // 获取当前的块级作用域
    pub fn current(&mut self) -> Option<&mut Scope<'ctx>> {
        self.scope_chains.last_mut()
    }

    // 将一个变量放置到当前块作用域中
    pub fn put_variable(&mut self, name: &str, scope_type: ScopeType<'ctx>) {
        let mut scope = self.current().unwrap();
        scope.add(name, scope_type);
    }

    // 将一个函数放置到全局作用域中
    pub fn put_fn(&mut self, name: &str, scope_type: ScopeType<'ctx>) {
        self.fns.add(name, scope_type);
    }

    // 作用域范围内搜索变量或方法声明
    pub fn search_by_name(
        &self,
        name: &str,
        only_user: bool, // 只搜索用户定义的
    ) -> Option<&ScopeType<'ctx>> {
        // 搜索变量作用域链
        for scope in self.scope_chains.iter().rev() {
            if scope.has(name) {
                return scope.get(name);
            }
        }

        // 搜索全局方法
        if self.fns.has(name) {
            return self.fns.get(name);
        }

        // 搜索 external 作用域
        if !only_user && self.external.has(name) {
            return self.external.get(name);
        }

        None
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Label<'ctx> {
    pub name: Option<String>,
    pub condition_ptr: PointerValue<'ctx>,
    pub after_block: BasicBlock<'ctx>,
}

impl<'ctx> Label<'ctx> {
    pub fn read_name(&self) -> String {
        match &self.name {
            Some(v) => v.to_string(),
            None => String::new(),
        }
    }
}

#[derive(Debug)]
pub struct Labels<'ctx> {
    pub last_break_label: Option<Label<'ctx>>,
    label_chains: Vec<Label<'ctx>>,
}

impl<'ctx> Labels<'ctx> {
    pub fn new() -> Self {
        Labels {
            last_break_label: None,
            label_chains: vec![],
        }
    }

    pub fn push(
        &mut self,
        name: Option<String>,
        condition_ptr: PointerValue<'ctx>,
        loop_block: BasicBlock<'ctx>,
        after_block: BasicBlock<'ctx>,
    ) {
        match name {
            None => {}
            Some(ref v) => {
                if self.has(v) {
                    panic!("Label `{}` is exists", v);
                }
            }
        }
        self.label_chains.push(Label {
            name,
            condition_ptr,
            after_block,
        });
    }

    pub fn pop(&mut self) {
        self.label_chains.pop();
    }

    pub fn has(&mut self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn get(&mut self, name: &str) -> Option<&Label<'ctx>> {
        for i in self.label_chains.iter() {
            if i.read_name() == name {
                return Some(i);
            }
        }
        None
    }

    pub fn current(&self) -> Option<&Label<'ctx>> {
        self.label_chains.last()
    }

    pub fn get_after_all(&self, name: &str) -> Option<Vec<&Label<'ctx>>> {
        let mut res = vec![];
        let mut is_find = false;
        for i in self.label_chains.iter() {
            if i.read_name() == name {
                is_find = true;
            }
            if is_find {
                res.push(i);
            }
        }
        if is_find {
            Some(res)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.label_chains.clear();
    }
}
