use llvm_sys::prelude::LLVMValueRef;
use std::collections::HashMap;
use x_lang_ast::shared::Kind;

#[derive(Debug)]
pub enum ScopeType {
    Function {
        return_kind: Kind,
        ptr: LLVMValueRef,
    },
    Variable {
        kind: Kind,
        ptr: LLVMValueRef,
    },
}

impl ScopeType {
    pub fn is_fn(&self) -> bool {
        match self {
            ScopeType::Function { .. } => true,
            ScopeType::Variable { .. } => false,
        }
    }

    pub fn is_var(&self) -> bool {
        !self.is_fn()
    }

    pub fn get_fn(&self) -> (&Kind, &LLVMValueRef) {
        match self {
            ScopeType::Function { return_kind, ptr } => (return_kind, ptr),
            ScopeType::Variable { .. } => panic!("Error"),
        }
    }

    pub fn get_var(&self) -> (&Kind, &LLVMValueRef) {
        match self {
            ScopeType::Function { .. } => panic!("Error"),
            ScopeType::Variable { kind, ptr } => (kind, ptr),
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    _map: HashMap<String, ScopeType>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            _map: HashMap::new(),
        }
    }

    // 根据名称返回 scope 类型
    #[inline]
    pub fn get(&self, name: &str) -> Option<&ScopeType> {
        self._map.get(name)
    }

    // 是否存在某个 scope 名
    #[inline]
    pub fn has(&self, name: &str) -> bool {
        match self.get(name) {
            Some(_) => true,
            None => false,
        }
    }

    // 新增一个 scope 命名空间
    pub fn add(&mut self, name: &str, scope_type: ScopeType) {
        self._map.insert(name.to_string(), scope_type);
    }

    // 移出一个 scope 命名空间
    pub fn remove(&mut self, name: &str) {
        self._map.remove(name);
    }

    // 清除当前作用域所有变量和方法声明
    pub fn clear(&mut self) {
        self._map.clear();
    }
}

#[derive(Debug)]
pub struct BlockScope {
    _scopes: Vec<Scope>,
}

impl BlockScope {
    pub fn new() -> Self {
        BlockScope { _scopes: vec![] }
    }

    // 将一个新的块级作用域压入栈中
    pub fn push(&mut self) {
        let scope = Scope::new();
        self._scopes.push(scope);
    }

    // 当前块级作用域出栈
    pub fn pop(&mut self) {
        self._scopes.pop();
    }

    // 获取当前的块级作用域
    pub fn current(&mut self) -> Option<&mut Scope> {
        self._scopes.last_mut()
    }

    // 将一个变量或函数放置到当前块作用域中
    pub fn put_variable(&mut self, name: &str, scope_type: ScopeType) {
        let mut scope = self.current().unwrap();
        scope.add(name, scope_type);
    }

    // 作用域范围内搜索变量
    pub fn search_variable(&self, name: &str) -> Option<&ScopeType> {
        for scope in self._scopes.iter().rev() {
            if scope.has(name) {
                return scope.get(name);
            }
        }
        None
    }
}