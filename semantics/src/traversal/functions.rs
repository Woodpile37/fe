use crate::errors::SemanticError;
use crate::namespace::scopes::{
    ContractDef,
    ContractScope,
    FunctionScope,
    Scope,
    Shared,
};
use crate::namespace::types::FixedSize;
use crate::traversal::_utils::spanned_expression;
use crate::traversal::{
    assignments,
    declarations,
    expressions,
    types,
};
use crate::{
    Context,
    FunctionAttributes,
};
use fe_parser::ast as fe;
use fe_parser::span::Spanned;
use std::rc::Rc;

/// Gather context information for a function definition and check for type
/// errors.
pub fn func_def(
    contract_scope: Shared<ContractScope>,
    context: Shared<Context>,
    def: &Spanned<fe::ContractStmt>,
) -> Result<(), SemanticError> {
    if let fe::ContractStmt::FuncDef {
        qual,
        name,
        args,
        return_type,
        body,
    } = &def.node
    {
        let function_scope = FunctionScope::new(def.span, Rc::clone(&contract_scope));

        let name = name.node.to_string();
        let param_types = args
            .iter()
            .map(|arg| func_def_arg(Rc::clone(&function_scope), arg))
            .collect::<Result<Vec<_>, _>>()?;

        let return_type = return_type
            .as_ref()
            .map(|typ| {
                types::type_desc_fixed_size(Scope::Function(Rc::clone(&function_scope)), &typ)
            })
            .transpose()?;

        match return_type {
            Some(_) => validate_all_paths_return_or_revert(&body)?,
            None => validate_no_path_returns(&body)?,
        }

        let is_public = qual.is_some();
        contract_scope.borrow_mut().add_function(
            name.clone(),
            is_public,
            param_types.clone(),
            return_type.clone(),
        );

        let attributes = FunctionAttributes {
            name,
            param_types,
            return_type,
        };

        context.borrow_mut().add_function(def, attributes);

        for stmt in body.iter() {
            func_stmt(Rc::clone(&function_scope), Rc::clone(&context), stmt)?
        }

        return Ok(());
    }

    unreachable!()
}

fn validate_all_paths_return_or_revert(
    body: &[Spanned<fe::FuncStmt>],
) -> Result<(), SemanticError> {
    // This will need to become more sophisticated when we introduce branching logic
    // because we then need to follow different code paths and check that they
    // all return or revert.
    for statement in body {
        if let fe::FuncStmt::Return { .. } = &statement.node {
            return Ok(());
        }

        if let fe::FuncStmt::Revert { .. } = &statement.node {
            return Ok(());
        }
    }

    Err(SemanticError::MissingReturn)
}

fn validate_no_path_returns(body: &[Spanned<fe::FuncStmt>]) -> Result<(), SemanticError> {
    // This will need to become more sophisticated when we introduce branching logic
    // because we then need to follow different code paths and check that none
    // of it returns.
    for statement in body {
        if let fe::FuncStmt::Return { .. } = &statement.node {
            return Err(SemanticError::UnexpectedReturn);
        }
    }

    Ok(())
}

fn func_def_arg(
    scope: Shared<FunctionScope>,
    arg: &Spanned<fe::FuncDefArg>,
) -> Result<FixedSize, SemanticError> {
    let name = arg.node.name.node.to_string();
    let typ = types::type_desc_fixed_size(Scope::Function(Rc::clone(&scope)), &arg.node.typ)?;

    match typ.clone() {
        FixedSize::Base(base) => scope.borrow_mut().add_base(name, base),
        FixedSize::Array(array) => scope.borrow_mut().add_array(name, array),
    }

    Ok(typ)
}

fn func_stmt(
    scope: Shared<FunctionScope>,
    context: Shared<Context>,
    stmt: &Spanned<fe::FuncStmt>,
) -> Result<(), SemanticError> {
    match &stmt.node {
        fe::FuncStmt::Return { .. } => func_return(scope, context, stmt),
        fe::FuncStmt::VarDecl { .. } => declarations::var_decl(scope, context, stmt),
        fe::FuncStmt::Assign { .. } => assignments::assign(scope, context, stmt),
        fe::FuncStmt::Emit { .. } => emit(scope, context, stmt),
        fe::FuncStmt::AugAssign { .. } => unimplemented!(),
        fe::FuncStmt::For { .. } => unimplemented!(),
        fe::FuncStmt::While { .. } => unimplemented!(),
        fe::FuncStmt::If { .. } => unimplemented!(),
        fe::FuncStmt::Assert { .. } => unimplemented!(),
        fe::FuncStmt::Expr { .. } => unimplemented!(),
        fe::FuncStmt::Pass => unimplemented!(),
        fe::FuncStmt::Break => unimplemented!(),
        fe::FuncStmt::Continue => unimplemented!(),
        fe::FuncStmt::Revert => Ok(()),
    }
}

fn emit(
    scope: Shared<FunctionScope>,
    context: Shared<Context>,
    stmt: &Spanned<fe::FuncStmt>,
) -> Result<(), SemanticError> {
    if let fe::FuncStmt::Emit {
        value: Spanned {
            node: fe::Expr::Call { func, args },
            ..
        },
    } = &stmt.node
    {
        let event_name = expressions::expr_name_string(func)?;

        if let Some(ContractDef::Event(event)) = scope.borrow().contract_def(event_name) {
            context.borrow_mut().add_emit(stmt, event);
        }

        for arg in args.node.iter() {
            call_arg(Rc::clone(&scope), Rc::clone(&context), arg)?;
        }

        return Ok(());
    }

    unreachable!()
}

fn call_arg(
    scope: Shared<FunctionScope>,
    context: Shared<Context>,
    arg: &Spanned<fe::CallArg>,
) -> Result<(), SemanticError> {
    match &arg.node {
        fe::CallArg::Arg(value) => {
            let spanned = spanned_expression(&arg.span, value);
            let _attributes = expressions::expr(scope, context, &spanned)?;
            // TODO: Perform type checking
        }
        fe::CallArg::Kwarg(fe::Kwarg { name: _, value }) => {
            let _attributes = expressions::expr(scope, context, value)?;
            // TODO: Perform type checking
        }
    };

    Ok(())
}

fn func_return(
    scope: Shared<FunctionScope>,
    context: Shared<Context>,
    stmt: &Spanned<fe::FuncStmt>,
) -> Result<(), SemanticError> {
    if let fe::FuncStmt::Return { value: Some(value) } = &stmt.node {
        let attributes = expressions::expr(scope.clone(), context.clone(), value)?;

        match context.borrow().get_function(scope.borrow().span) {
            Some(fn_attr) => match fn_attr.return_type.clone() {
                Some(return_type) => {
                    if return_type.into_type() != attributes.typ {
                        return Err(SemanticError::TypeError);
                    }
                }
                None => return Err(SemanticError::UnexpectedReturn),
            },
            None => unreachable!(),
        }

        return Ok(());
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use crate::namespace::scopes::{
        ContractDef,
        ContractScope,
        ModuleScope,
        Shared,
    };
    use crate::namespace::types::{
        Base,
        FixedSize,
    };
    use crate::traversal::functions::func_def;
    use crate::Context;
    use fe_parser as parser;
    use std::rc::Rc;

    fn scope() -> Shared<ContractScope> {
        let module_scope = ModuleScope::new();
        ContractScope::new(module_scope)
    }

    fn analyze(scope: Shared<ContractScope>, src: &str) -> Context {
        let context = Context::new_shared();
        let tokens = parser::get_parse_tokens(src).expect("Couldn't parse expression");
        let def = &parser::parsers::func_def(&tokens[..])
            .expect("Couldn't build func def AST")
            .1;

        func_def(scope, Rc::clone(&context), def).expect("Couldn't map func def AST");
        Rc::try_unwrap(context)
            .map_err(|_| "")
            .unwrap()
            .into_inner()
    }

    #[test]
    fn simple_func_def() {
        let scope = scope();
        let func_def = "\
        def foo(x: u256) -> u256:\
            return x + x\
        ";
        let context = analyze(Rc::clone(&scope), func_def);
        assert_eq!(context.expressions.len(), 3);
        assert_eq!(
            scope.borrow().def("foo".to_string()),
            Some(ContractDef::Function {
                is_public: false,
                params: vec![FixedSize::Base(Base::U256)],
                returns: Some(FixedSize::Base(Base::U256))
            })
        );
    }
}