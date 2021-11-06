use inkwell::builder::Builder;
use inkwell::values::FloatValue;
use std::collections::HashMap;
use x_lang_ast::shared::{Kind, KindName};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum KindValue<'ctx> {
    Number(FloatValue<'ctx>),
}

impl<'ctx> KindValue<'ctx> {
    pub fn read_number(&self) -> &FloatValue<'ctx> {
        match self {
            KindValue::Number(v) => v,
        }
    }
}

#[derive(Debug)]
pub struct VariableDecl<'ctx> {
    pub kind: Kind,
    pub value: KindValue<'ctx>,
}

#[derive(Debug)]
pub struct FunctionDecl {
    pub return_kind: Kind,
}

#[derive(Debug)]
pub enum ScopeDecl<'ctx> {
    Function(FunctionDecl),
    Variable(VariableDecl<'ctx>),
}

impl<'ctx> ScopeDecl<'ctx> {
    pub fn is_fn(&self) -> bool {
        match self {
            ScopeDecl::Function(_) => true,
            ScopeDecl::Variable(_) => false,
        }
    }

    pub fn is_var(&self) -> bool {
        !self.is_fn()
    }

    pub fn get_fn_decl(&self) -> &FunctionDecl {
        match self {
            ScopeDecl::Function(v) => v,
            ScopeDecl::Variable(_) => panic!("Error"),
        }
    }

    pub fn get_var_decl(&self) -> &VariableDecl<'ctx> {
        match self {
            ScopeDecl::Function(_) => panic!("Error"),
            ScopeDecl::Variable(v) => v,
        }
    }
}

#[derive(Debug)]
pub struct Scope<'ctx> {
    _map: HashMap<String, ScopeDecl<'ctx>>,
}

impl<'ctx> Scope<'ctx> {
    pub fn new() -> Self {
        Scope {
            _map: HashMap::new(),
        }
    }

    // 返回变量或方法声明
    #[inline]
    pub fn get(&self, name: &str) -> Option<&ScopeDecl> {
        self._map.get(name)
    }

    // 返回变量名的值
    #[inline]
    pub fn get_var(&self, name: &str) -> Option<&VariableDecl<'ctx>> {
        match self._map.get(name) {
            Some(v) => {
                if v.is_var() {
                    return Some(v.get_var_decl());
                }
            }
            None => {}
        };
        None
    }

    // 返回变量名的值
    #[inline]
    pub fn get_fn(&self, name: &str) -> Option<&FunctionDecl> {
        match self._map.get(name) {
            Some(v) => {
                if v.is_fn() {
                    return Some(v.get_fn_decl());
                }
            }
            None => {}
        };
        None
    }

    // 是否存在某个变量或方法声明
    #[inline]
    pub fn has(&self, name: &str) -> bool {
        match self.get(name) {
            Some(_) => true,
            None => false,
        }
    }

    // 新增一个变量
    pub fn add_var(&mut self, name: &str, decl: VariableDecl<'ctx>) {
        self._map
            .insert(name.to_string(), ScopeDecl::Variable(decl));
    }

    // 新增一个方法声明
    pub fn add_fn(&mut self, name: &str, decl: FunctionDecl) {
        self._map
            .insert(name.to_string(), ScopeDecl::Function(decl));
    }

    // 移出一个变量
    pub fn remove_var(&mut self, name: &str) {
        self._map.remove(name);
    }

    // 移出一个方法声明
    pub fn remove_fn(&mut self, name: &str) {
        self._map.remove(name);
    }

    // 清除当前作用域所有变量和方法声明
    pub fn clear(&mut self) {
        self._map.clear();
    }
}

#[derive(Debug)]
pub struct BlockScope<'ctx> {
    _scopes: Vec<Scope<'ctx>>,
}

impl<'ctx> BlockScope<'ctx> {
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
    pub fn current(&mut self) -> Option<&mut Scope<'ctx>> {
        self._scopes.last_mut()
    }

    // 将一个变量放置到当前块作用域中
    pub fn put_variable(&mut self, name: &str, decl: VariableDecl<'ctx>) {
        let mut scope = self.current().unwrap();
        scope.add_var(name, decl);
    }

    // 将一个函数声明放置到当前块作用域中
    pub fn put_fn(&mut self, name: &str, decl: FunctionDecl) {
        let mut scope = self.current().unwrap();
        scope.add_fn(name, decl);
    }

    // 作用域范围内搜索变量
    pub fn search_variable(&self, name: &str) -> Option<&ScopeDecl> {
        let mut i = self._scopes.len() - 1;
        while i >= 0 {
            let scope = &self._scopes[i];
            if scope.has(name) {
                return scope.get(name);
            }
            i -= 1;
        }
        None
    }
}
